use crate::tree::ProgressStep;
use std::fmt;

pub trait DisplayValue {
    fn display_current_value(
        &self,
        f: &mut fmt::Formatter,
        value: ProgressStep,
        max: Option<ProgressStep>,
    ) -> fmt::Result;
    fn display_upper_bound(&self, f: &mut fmt::Formatter, value: ProgressStep) -> fmt::Result;
    fn display_unit(&self, f: &mut fmt::Formatter, value: ProgressStep) -> fmt::Result;
    fn display_percentage(&self, f: &mut fmt::Formatter, fraction: f64) -> fmt::Result;
}

pub enum Unit {
    Static(&'static str, Option<UnitMode>),
    Dynamic(Box<dyn DisplayValue>, Option<UnitMode>),
}

pub enum UnitMode {
    PercentageBeforeValue,
    PercentageAfterUnit,
}

pub(crate) struct UnitDisplay<'a> {
    current_value: ProgressStep,
    upper_bound: Option<ProgressStep>,
    parent: &'a Unit,
}

impl Unit {
    pub(crate) fn display(&self, current_value: ProgressStep, upper_bound: Option<ProgressStep>) -> UnitDisplay {
        UnitDisplay {
            current_value,
            upper_bound,
            parent: self,
        }
    }
}

impl<'a> fmt::Display for UnitDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.parent {
            Unit::Static(unit, mode) => unimplemented!("static mode"),
            Unit::Dynamic(unit, mode) => unimplemented!("dynamic mode"),
        }
    }
}
