use crate::{tree::ProgressStep, unit::DisplayValue};
use std::fmt;

#[derive(Copy, Clone, Default, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Bytes;

impl Bytes {
    fn format_bytes(f: &mut fmt::Formatter, value: ProgressStep) -> fmt::Result {
        let string = bytesize::to_string(value as u64, false);
        for token in string.split(' ') {
            write!(f, "{}", token)?;
        }
        Ok(())
    }
}

impl DisplayValue for Bytes {
    fn display_current_value(
        &self,
        f: &mut fmt::Formatter,
        value: ProgressStep,
        _upper: Option<ProgressStep>,
    ) -> fmt::Result {
        Self::format_bytes(f, value)
    }
    fn display_upper_bound(
        &self,
        f: &mut fmt::Formatter,
        upper_bound: ProgressStep,
        _value: ProgressStep,
    ) -> fmt::Result {
        Self::format_bytes(f, upper_bound)
    }
    fn display_unit(&self, _f: &mut fmt::Formatter, _value: usize) -> fmt::Result {
        Ok(())
    }
}
