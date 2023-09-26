use std::{fmt, hash::Hasher};

pub use humansize::{format_size_i, FormatSizeOptions, ISizeFormatter};
#[cfg(doc)]
use humansize::{BINARY, DECIMAL};

use crate::{progress::Step, unit::DisplayValue};

/// A helper for formatting numbers in a format easily read by humans in
/// renderers, as in `2.54 million objects`
pub struct Human {
    /// The name of the represented unit, like 'items' or 'objects'.
    pub name: &'static str,
    /// Formatting options of [`humansize`].
    pub format_options: FormatSizeOptions,
}

impl Human {
    /// A convenience method to create a new instance with a set of
    /// [`humansize`] format options, like [`BINARY`] or [`DECIMAL`],
    /// and a name for the unit.
    pub fn new(format_options: FormatSizeOptions, name: &'static str) -> Self {
        Self { name, format_options }
    }
    fn format_bytes(&self, w: &mut dyn fmt::Write, value: Step) -> fmt::Result {
        let string = format_size_i(value as f64, self.format_options);
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

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        state.write(self.name.as_bytes());
        state.write_u8(0);
    }

    fn display_unit(&self, w: &mut dyn fmt::Write, _value: Step) -> fmt::Result {
        w.write_str(self.name)
    }
}
