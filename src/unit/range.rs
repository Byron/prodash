use crate::tree::ProgressStep;
use crate::unit::DisplayValue;
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
    fn separator(&self, w: &mut dyn fmt::Write, _value: ProgressStep, _upper: Option<ProgressStep>) -> fmt::Result {
        w.write_str(" of ")
    }
    fn display_current_value(
        &self,
        w: &mut dyn fmt::Write,
        value: ProgressStep,
        _upper: Option<ProgressStep>,
    ) -> fmt::Result {
        fmt::write(w, format_args!("{} {}", self.name, value + 1))
    }
    fn display_unit(&self, _w: &mut dyn fmt::Write, _value: usize) -> fmt::Result {
        Ok(())
    }
}
