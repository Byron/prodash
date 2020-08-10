use crate::tree::progress::Step;
use std::{fmt, ops::Deref, sync::Arc};

#[cfg(feature = "unit-bytes")]
mod bytes;
#[cfg(feature = "unit-bytes")]
pub use bytes::Bytes;

#[cfg(feature = "unit-duration")]
mod duration;
#[cfg(feature = "unit-duration")]
pub use duration::Duration;

#[cfg(feature = "unit-human")]
pub mod human;
#[cfg(feature = "unit-human")]
#[doc(inline)]
pub use human::Human;

mod range;
pub use range::Range;

mod traits;
pub use traits::DisplayValue;

pub mod display;

#[derive(Debug, Clone)]
pub struct Unit {
    kind: Kind,
    mode: Option<display::Mode>,
}

#[derive(Clone)]
pub enum Kind {
    Label(&'static str),
    Dynamic(Arc<dyn DisplayValue + Send + Sync>),
}

impl fmt::Debug for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Label(name) => f.write_fmt(format_args!("Unit::Label({:?})", name)),
            Kind::Dynamic(_) => f.write_fmt(format_args!("Unit::Dynamic(..)")),
        }
    }
}

impl From<&'static str> for Unit {
    fn from(v: &'static str) -> Self {
        label(v)
    }
}

pub fn label(label: &'static str) -> Unit {
    Unit {
        kind: Kind::Label(label),
        mode: None,
    }
}
pub fn label_and_mode(label: &'static str, mode: display::Mode) -> Unit {
    Unit {
        kind: Kind::Label(label),
        mode: Some(mode),
    }
}
pub fn dynamic(label: impl DisplayValue + Send + Sync + 'static) -> Unit {
    Unit {
        kind: Kind::Dynamic(Arc::new(label)),
        mode: None,
    }
}
pub fn dynamic_and_mode(label: impl DisplayValue + Send + Sync + 'static, mode: display::Mode) -> Unit {
    Unit {
        kind: Kind::Dynamic(Arc::new(label)),
        mode: Some(mode),
    }
}

/// Display and utilities
impl Unit {
    pub fn display(
        &self,
        current_value: Step,
        upper_bound: Option<Step>,
        elapsed: impl Into<Option<display::Throughput>>,
    ) -> display::UnitDisplay {
        display::UnitDisplay {
            current_value,
            upper_bound,
            throughput: elapsed.into(),
            parent: self,
            display: display::What::ValuesAndUnit,
        }
    }

    fn as_display_value(&self) -> &dyn DisplayValue {
        match self.kind {
            Kind::Label(ref unit) => unit,
            Kind::Dynamic(ref unit) => unit.deref(),
        }
    }
}

#[cfg(test)]
mod tests;
