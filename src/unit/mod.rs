use crate::tree::ProgressStep;
use std::{fmt, fmt::Write, ops::Deref, sync::Arc};

#[cfg(feature = "unit-bytes")]
mod bytes;
#[cfg(feature = "unit-bytes")]
pub use bytes::Bytes;

#[cfg(feature = "unit-duration")]
mod duration;
#[cfg(feature = "unit-duration")]
pub use duration::Duration;

#[cfg(feature = "unit-human")]
pub mod human;
#[cfg(feature = "unit-human")]
#[doc(inline)]
pub use human::Human;

mod range;
pub use range::Range;

pub trait DisplayValue {
    fn display_current_value(
        &self,
        w: &mut dyn fmt::Write,
        value: ProgressStep,
        _upper: Option<ProgressStep>,
    ) -> fmt::Result {
        fmt::write(w, format_args!("{}", value))
    }
    fn separator(&self, w: &mut dyn fmt::Write, _value: ProgressStep, _upper: Option<ProgressStep>) -> fmt::Result {
        w.write_str("/")
    }
    fn display_upper_bound(
        &self,
        w: &mut dyn fmt::Write,
        upper_bound: ProgressStep,
        _value: ProgressStep,
    ) -> fmt::Result {
        fmt::write(w, format_args!("{}", upper_bound))
    }
    fn display_unit(&self, w: &mut dyn fmt::Write, value: ProgressStep) -> fmt::Result;
    fn display_percentage(&self, w: &mut dyn fmt::Write, percentage: f64) -> fmt::Result {
        w.write_fmt(format_args!("[{}%]", percentage as usize))
    }
}

impl DisplayValue for &'static str {
    fn display_unit(&self, w: &mut dyn fmt::Write, _value: usize) -> fmt::Result {
        w.write_fmt(format_args!("{}", self))
    }
}

#[derive(Clone)]
pub enum Unit {
    Label(&'static str, Option<Mode>),
    Dynamic(Arc<dyn DisplayValue + Send + Sync>, Option<Mode>),
}

impl fmt::Debug for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unit::Label(name, mode) => f.write_fmt(format_args!("Unit::Label({:?}, {:?})", name, mode)),
            Unit::Dynamic(_, mode) => f.write_fmt(format_args!("Unit::Dynamic(.., {:?})", mode)),
        }
    }
}

impl From<&'static str> for Unit {
    fn from(v: &'static str) -> Self {
        Unit::label(v)
    }
}

/// Construction
impl Unit {
    pub fn label(label: &'static str) -> Self {
        Unit::Label(label, None)
    }
    pub fn label_and_mode(label: &'static str, mode: Mode) -> Self {
        Unit::Label(label, Some(mode))
    }
    pub fn dynamic(label: impl DisplayValue + Send + Sync + 'static) -> Self {
        Unit::Dynamic(Arc::new(label), None)
    }
    pub fn dynamic_and_mode(label: impl DisplayValue + Send + Sync + 'static, mode: Mode) -> Self {
        Unit::Dynamic(Arc::new(label), Some(mode))
    }
}

/// Display and utilities
impl Unit {
    pub fn display(&self, current_value: ProgressStep, upper_bound: Option<ProgressStep>) -> UnitDisplay {
        UnitDisplay {
            current_value,
            upper_bound,
            parent: self,
            display: WhatToDisplay::ValuesAndUnit,
        }
    }

    pub fn as_display_value(&self) -> (&dyn DisplayValue, Option<Mode>) {
        match self {
            Unit::Label(unit, mode) => (unit, *mode),
            Unit::Dynamic(unit, mode) => (unit.deref(), *mode),
        }
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum Mode {
    PercentageBeforeValue,
    PercentageAfterUnit,
}

pub struct UnitDisplay<'a> {
    current_value: ProgressStep,
    upper_bound: Option<ProgressStep>,
    parent: &'a Unit,
    display: WhatToDisplay,
}

enum WhatToDisplay {
    ValuesAndUnit,
    Unit,
    Values,
}

impl WhatToDisplay {
    fn values(&self) -> bool {
        match self {
            WhatToDisplay::Values | WhatToDisplay::ValuesAndUnit => true,
            _ => false,
        }
    }
    fn unit(&self) -> bool {
        match self {
            WhatToDisplay::Unit | WhatToDisplay::ValuesAndUnit => true,
            _ => false,
        }
    }
}

impl<'a> UnitDisplay<'a> {
    pub fn all(&mut self) -> &Self {
        self.display = WhatToDisplay::ValuesAndUnit;
        self
    }
    pub fn values(&mut self) -> &Self {
        self.display = WhatToDisplay::Values;
        self
    }
    pub fn unit(&mut self) -> &Self {
        self.display = WhatToDisplay::Unit;
        self
    }
}

impl<'a> fmt::Display for UnitDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (unit, mode): (&dyn DisplayValue, _) = self.parent.as_display_value();

        let mode_and_fraction = mode.and_then(|mode| {
            self.upper_bound
                .map(|upper| (mode, ((self.current_value as f64 / upper as f64) * 100.0).floor()))
        });
        if self.display.values() {
            if let Some((Mode::PercentageBeforeValue, fraction)) = mode_and_fraction {
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

            if let Some((Mode::PercentageAfterUnit, fraction)) = mode_and_fraction {
                f.write_char(' ')?;
                unit.display_percentage(f, fraction)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
