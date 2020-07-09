#[cfg(feature = "localtime")]
mod format_localtime {
    use std::time::SystemTime;

    pub fn now_datetime_seconds() -> String {
        time::OffsetDateTime::now_local().format("%F %T")
    }
    pub fn time(time: SystemTime) -> String {
        time::OffsetDateTime::from(time)
            .to_offset(time::UtcOffset::current_local_offset())
            .format("%T")
    }
}

#[cfg(all(feature = "humantime", not(feature = "localtime")))]
mod format_utc {
    const DATE_TIME_HMS: usize = "00:51:45".len();

    use std::time::SystemTime;
    const DATE_TIME_YMD: usize = "2020-02-13T".len();

    pub fn now_datetime_seconds() -> String {
        String::from_utf8_lossy(
            &humantime::format_rfc3339_seconds(std::time::SystemTime::now())
                .to_string()
                .as_bytes()[.."2020-02-13T00:51:45".len()],
        )
        .into_owned()
    }

    pub fn time(time: SystemTime) -> String {
        String::from_utf8_lossy(
            &humantime::format_rfc3339_seconds(time).to_string().as_bytes()
                [DATE_TIME_YMD..DATE_TIME_YMD + DATE_TIME_HMS],
        )
        .into_owned()
    }
}

/// Various utilities for formatting time in either **utc** with the `utctime` feature or **localtime**
/// with the `localtime` feature.
pub mod format {
    #[cfg(feature = "localtime")]
    pub use super::format_localtime::*;

    #[cfg(all(feature = "humantime", not(feature = "localtime")))]
    pub use super::format_utc::*;
}
