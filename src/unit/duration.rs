use crate::{progress::Step, unit::DisplayValue};
use std::fmt;

#[derive(Copy, Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Duration;

impl DisplayValue for Duration {
    fn display_current_value(&self, w: &mut dyn fmt::Write, value: Step, _upper: Option<Step>) -> fmt::Result {
        w.write_str(&compound_duration::format_dhms(value))
    }
    fn separator(&self, w: &mut dyn fmt::Write, _value: Step, _upper: Option<Step>) -> fmt::Result {
        w.write_str(" of ")
    }
    fn display_upper_bound(&self, w: &mut dyn fmt::Write, upper_bound: Step, _value: Step) -> fmt::Result {
        w.write_str(&compound_duration::format_dhms(upper_bound))
    }
    fn display_unit(&self, _w: &mut dyn fmt::Write, _value: Step) -> fmt::Result {
        Ok(())
    }
}
