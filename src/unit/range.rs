use crate::{progress::Step, unit::DisplayValue};
use std::fmt;

#[derive(Copy, Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Range {
    pub name: &'static str,
}

impl Range {
    pub fn new(name: &'static str) -> Self {
        Range { name }
    }
}

impl DisplayValue for Range {
    fn display_current_value(&self, w: &mut dyn fmt::Write, value: Step, _upper: Option<Step>) -> fmt::Result {
        w.write_fmt(format_args!("{}", value + 1))
    }
    fn separator(&self, w: &mut dyn fmt::Write, _value: Step, _upper: Option<Step>) -> fmt::Result {
        w.write_str(" of ")
    }
    fn display_unit(&self, w: &mut dyn fmt::Write, _value: Step) -> fmt::Result {
        w.write_str(self.name)
    }
}
