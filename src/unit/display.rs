use crate::{
    progress::Step,
    unit::{DisplayValue, Unit},
};
use std::fmt::{self, Write};

/// The location at which [`Throughput`] or [`UnitDisplays`][UnitDisplay] should be placed.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[allow(missing_docs)]
pub enum Location {
    BeforeValue,
    AfterUnit,
}

/// A structure able to display throughput, a value change within a given duration.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Throughput {
    /// The change of value between the current value and the previous one.
    pub value_change_in_timespan: Step,
    /// The amount of time passed between the previous and the current value.
    pub timespan: std::time::Duration,
}

impl Throughput {
    /// A convenience method to create a new ThroughPut from `value_change_in_timespan` and `timespan`.
    pub fn new(value_change_in_timespan: Step, timespan: std::time::Duration) -> Self {
        Throughput {
            value_change_in_timespan,
            timespan,
        }
    }
}

/// A way to display a [Unit].
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

    fn throughput_location(&self) -> Option<Location> {
        if self.throughput {
            Some(self.location)
        } else {
            None
        }
    }
}

/// initialization and modification
impl Mode {
    /// Create a mode instance with percentage only.
    pub fn with_percentage() -> Self {
        Mode {
            percent: true,
            throughput: false,
            location: Location::AfterUnit,
        }
    }
    /// Create a mode instance with throughput only.
    pub fn with_throughput() -> Self {
        Mode {
            percent: false,
            throughput: true,
            location: Location::AfterUnit,
        }
    }
    /// Turn on percentage display on the current instance.
    pub fn and_percentage(mut self) -> Self {
        self.percent = true;
        self
    }
    /// Turn on throughput display on the current instance.
    pub fn and_throughput(mut self) -> Self {
        self.throughput = true;
        self
    }
    /// Change the display location to show up in front of the value.
    pub fn show_before_value(mut self) -> Self {
        self.location = Location::BeforeValue;
        self
    }
}

/// A utility to implement [Display][std::fmt::Display].
pub struct UnitDisplay<'a> {
    pub(crate) current_value: Step,
    pub(crate) upper_bound: Option<Step>,
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
        matches!(self, What::Values | What::ValuesAndUnit)
    }
    fn unit(&self) -> bool {
        matches!(self, What::Unit | What::ValuesAndUnit)
    }
}

impl<'a> UnitDisplay<'a> {
    /// Display everything, values and the unit.
    pub fn all(&mut self) -> &Self {
        self.display = What::ValuesAndUnit;
        self
    }
    /// Display only values.
    pub fn values(&mut self) -> &Self {
        self.display = What::Values;
        self
    }
    /// Display only units.
    pub fn unit(&mut self) -> &Self {
        self.display = What::Unit;
        self
    }
}

impl<'a> fmt::Display for UnitDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}
