#[cfg(feature = "localtime")]
mod localtime {
    use std::time::SystemTime;

    pub fn format_now_datetime_seconds() -> String {
        time::OffsetDateTime::try_now_local()
            .expect("time with local time offset")
            .format("%F %T")
    }
    pub fn format_time_for_messages(time: SystemTime) -> String {
        time::OffsetDateTime::from(time)
            .to_offset(time::UtcOffset::try_current_local_offset().expect("UTC with local offset to always work"))
            .format("%T")
    }
}

pub const DATE_TIME_HMS: usize = "00:51:45".len();

#[cfg(not(feature = "localtime"))]
mod utc {
    use super::DATE_TIME_HMS;
    use std::time::SystemTime;
    const DATE_TIME_YMD: usize = "2020-02-13T".len();

    pub fn format_time_for_messages(time: SystemTime) -> String {
        String::from_utf8_lossy(
            &humantime::format_rfc3339_seconds(time).to_string().as_bytes()
                [DATE_TIME_YMD..DATE_TIME_YMD + DATE_TIME_HMS],
        )
        .into_owned()
    }

    pub fn format_now_datetime_seconds() -> String {
        String::from_utf8_lossy(
            &humantime::format_rfc3339_seconds(std::time::SystemTime::now())
                .to_string()
                .as_bytes()[.."2020-02-13T00:51:45".len()],
        )
        .into_owned()
    }
}

#[cfg(feature = "localtime")]
pub use localtime::*;

#[cfg(not(feature = "localtime"))]
pub use utc::*;
