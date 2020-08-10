use crate::{tree::progress::Step, unit::DisplayValue};
use std::fmt;

#[derive(Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Bytes;

impl Bytes {
    fn format_bytes(w: &mut dyn fmt::Write, value: Step) -> fmt::Result {
        let string = bytesize::to_string(value as u64, false);
        for token in string.split(' ') {
            w.write_str(token)?;
        }
        Ok(())
    }
}

impl DisplayValue for Bytes {
    fn display_current_value(&self, w: &mut dyn fmt::Write, value: Step, _upper: Option<Step>) -> fmt::Result {
        Self::format_bytes(w, value)
    }
    fn display_upper_bound(&self, w: &mut dyn fmt::Write, upper_bound: Step, _value: Step) -> fmt::Result {
        Self::format_bytes(w, upper_bound)
    }
    fn display_unit(&self, _w: &mut dyn fmt::Write, _value: Step) -> fmt::Result {
        Ok(())
    }
}
