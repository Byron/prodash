use crate::{tree::ProgressStep, unit::display};
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
    fn display_throughput(&self, w: &mut dyn fmt::Write, throughput: display::Throughput) -> fmt::Result {
        let (fraction, unit) = self.fraction_and_time_unit(throughput.timespan);
        w.write_char('|')?;
        self.display_current_value(w, throughput.value_change_in_timespan, None)?;
        w.write_char('/')?;
        match fraction {
            Some(fraction) => w.write_fmt(format_args!("{}", fraction)),
            None => Ok(()),
        }?;
        w.write_fmt(format_args!("{}|", unit))
    }
    fn fraction_and_time_unit(&self, timespan: std::time::Duration) -> (Option<f64>, &'static str) {
        fn skip_one(v: f64) -> Option<f64> {
            if (v - 1.0).abs() < f64::EPSILON {
                None
            } else {
                Some(v)
            }
        }
        const HOUR_IN_SECS: u64 = 60 * 60;
        let secs = timespan.as_secs();
        let h = secs / HOUR_IN_SECS;
        if h > 0 {
            return (skip_one(secs as f64 / HOUR_IN_SECS as f64), "h");
        }
        const MINUTES_IN_SECS: u64 = 60;
        let m = secs / MINUTES_IN_SECS;
        if m > 0 {
            return (skip_one(secs as f64 / MINUTES_IN_SECS as f64), "m");
        }
        if secs > 0 {
            return (skip_one(secs as f64), "s");
        }

        (skip_one(timespan.as_millis() as f64), "ms")
    }
}

impl DisplayValue for &'static str {
    fn display_unit(&self, w: &mut dyn fmt::Write, _value: usize) -> fmt::Result {
        w.write_fmt(format_args!("{}", self))
    }
}
