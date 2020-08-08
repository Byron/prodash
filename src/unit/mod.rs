use crate::tree::ProgressStep;
use std::fmt::Write;
use std::{fmt, ops::Deref};

pub trait DisplayValue {
    fn display_current_value(
        &self,
        f: &mut fmt::Formatter,
        value: ProgressStep,
        _max: Option<ProgressStep>,
    ) -> fmt::Result {
        write!(f, "{}", value)
    }
    fn display_upper_bound(&self, f: &mut fmt::Formatter, value: ProgressStep) -> fmt::Result {
        write!(f, "{}", value)
    }
    fn display_unit(&self, f: &mut fmt::Formatter, value: ProgressStep) -> fmt::Result;
    fn display_percentage(&self, f: &mut fmt::Formatter, fraction: f64) -> fmt::Result {
        write!(f, "[{:.02}]", fraction)
    }
}

impl DisplayValue for &'static str {
    fn display_unit(&self, f: &mut fmt::Formatter, _value: usize) -> fmt::Result {
        write!(f, "{}", self)
    }
}

pub enum Unit {
    Label(&'static str, Option<UnitMode>),
    Dynamic(Box<dyn DisplayValue>, Option<UnitMode>),
}

/// Construction
impl Unit {
    pub fn label(label: &'static str) -> Self {
        Unit::Label(label, None)
    }
}

/// Display and utilities
impl Unit {
    pub fn display(&self, current_value: ProgressStep, upper_bound: Option<ProgressStep>) -> UnitDisplay {
        UnitDisplay {
            current_value,
            upper_bound,
            parent: self,
        }
    }

    pub fn as_display_value(&self) -> (&dyn DisplayValue, Option<UnitMode>) {
        match self {
            Unit::Label(unit, mode) => (unit, *mode),
            Unit::Dynamic(unit, mode) => (unit.deref(), *mode),
        }
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum UnitMode {
    PercentageBeforeValue,
    PercentageAfterUnit,
}

pub struct UnitDisplay<'a> {
    current_value: ProgressStep,
    upper_bound: Option<ProgressStep>,
    parent: &'a Unit,
}

impl<'a> fmt::Display for UnitDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (unit, mode): (&dyn DisplayValue, _) = self.parent.as_display_value();
        unit.display_current_value(f, self.current_value, self.upper_bound)?;
        if let Some(upper) = self.upper_bound {
            f.write_char('/')?;
            unit.display_upper_bound(f, upper)?;
        }
        f.write_char(' ')?;
        unit.display_unit(f, self.current_value)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
