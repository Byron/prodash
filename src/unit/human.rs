use crate::{tree::progress::Step, unit::DisplayValue};
pub use human_format::{Formatter, Scales};
use std::fmt;

#[derive(Debug)]
pub struct Human {
    pub name: &'static str,
    pub formatter: Formatter,
}

impl Human {
    pub fn new(formatter: Formatter, name: &'static str) -> Self {
        Human { formatter, name }
    }
    fn format_bytes(&self, w: &mut dyn fmt::Write, value: Step) -> fmt::Result {
        let string = self.formatter.format(value as f64);
        for token in string.split(' ') {
            w.write_str(token)?;
        }
        Ok(())
    }
}

impl DisplayValue for Human {
    fn display_current_value(&self, w: &mut dyn fmt::Write, value: Step, _upper: Option<Step>) -> fmt::Result {
        self.format_bytes(w, value)
    }

    fn display_upper_bound(&self, w: &mut dyn fmt::Write, upper_bound: Step, _value: Step) -> fmt::Result {
        self.format_bytes(w, upper_bound)
    }

    fn display_unit(&self, w: &mut dyn fmt::Write, _value: Step) -> fmt::Result {
        w.write_str(self.name)
    }
}
