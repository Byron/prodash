use crate::{
    tree::ProgressStep,
    unit::{DisplayValue, Unit},
};
use std::fmt::{self, Write};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Location {
    BeforeValue,
    AfterUnit,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
struct ThroughputState {
    desired: std::time::Duration,
    observed: std::time::Duration,
    aggregate_value_for_observed_duration: ProgressStep,
    last_value: Option<ProgressStep>,
}

impl Default for ThroughputState {
    fn default() -> Self {
        ThroughputState {
            desired: std::time::Duration::from_secs(1),
            observed: Default::default(),
            aggregate_value_for_observed_duration: 0,
            last_value: None,
        }
    }
}

pub struct Throughput {
    pub value_change_in_timespan: ProgressStep,
    pub timespan: std::time::Duration,
}

impl Throughput {
    pub fn new(value_change_in_timespan: ProgressStep, timespan: std::time::Duration) -> Self {
        Throughput {
            value_change_in_timespan,
            timespan,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Mode {
    location: Location,
    percent: bool,
    throughput: bool,
}

impl Mode {
    fn percent_location(&self) -> Option<Location> {
        if self.percent {
            Some(self.location)
        } else {
            None
        }
    }
}

/// initialization and modification
impl Mode {
    pub fn with_percentage() -> Self {
        Mode {
            percent: true,
            throughput: false,
            location: Location::AfterUnit,
        }
    }
    pub fn with_throughput_per_second() -> Self {
        Mode {
            percent: false,
            throughput: true,
            location: Location::AfterUnit,
        }
    }
    pub fn and_percentage(mut self) -> Self {
        self.percent = true;
        self
    }
    pub fn and_throughput(mut self) -> Self {
        self.throughput = true;
        self
    }
    pub fn show_before_value(mut self) -> Self {
        self.location = Location::BeforeValue;
        self
    }
}

pub struct UnitDisplay<'a> {
    pub(crate) current_value: ProgressStep,
    pub(crate) upper_bound: Option<ProgressStep>,
    pub(crate) throughput: Option<Throughput>,
    pub(crate) parent: &'a Unit,
    pub(crate) display: What,
}

pub(crate) enum What {
    ValuesAndUnit,
    Unit,
    Values,
}

impl What {
    fn values(&self) -> bool {
        match self {
            What::Values | What::ValuesAndUnit => true,
            _ => false,
        }
    }
    fn unit(&self) -> bool {
        match self {
            What::Unit | What::ValuesAndUnit => true,
            _ => false,
        }
    }
}

impl<'a> UnitDisplay<'a> {
    pub fn all(&mut self) -> &Self {
        self.display = What::ValuesAndUnit;
        self
    }
    pub fn values(&mut self) -> &Self {
        self.display = What::Values;
        self
    }
    pub fn unit(&mut self) -> &Self {
        self.display = What::Unit;
        self
    }
}

impl<'a> fmt::Display for UnitDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let unit: &dyn DisplayValue = self.parent.as_display_value();
        let mode = self.parent.mode;

        let percent_location_and_fraction = mode.and_then(|m| m.percent_location()).and_then(|location| {
            self.upper_bound
                .map(|upper| (location, ((self.current_value as f64 / upper as f64) * 100.0).floor()))
        });
        if self.display.values() {
            if let Some((Location::BeforeValue, fraction)) = percent_location_and_fraction {
                unit.display_percentage(f, fraction)?;
                f.write_char(' ')?;
            }
            unit.display_current_value(f, self.current_value, self.upper_bound)?;
            if let Some(upper) = self.upper_bound {
                unit.separator(f, self.current_value, self.upper_bound)?;
                unit.display_upper_bound(f, upper, self.current_value)?;
            }
        }
        if self.display.unit() {
            let mut buf = String::with_capacity(10);
            if self.display.values() {
                buf.write_char(' ')?;
            }
            unit.display_unit(&mut buf, self.current_value)?;
            if buf.len() > 1 {
                // did they actually write a unit?
                f.write_str(&buf)?;
            }

            if let Some((Location::AfterUnit, fraction)) = percent_location_and_fraction {
                f.write_char(' ')?;
                unit.display_percentage(f, fraction)?;
            }
        }
        Ok(())
    }
}
