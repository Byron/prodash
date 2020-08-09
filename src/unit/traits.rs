use crate::tree::ProgressStep;
use std::fmt;

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
