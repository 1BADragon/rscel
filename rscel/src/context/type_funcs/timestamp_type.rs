use rscel_macro::dispatch;

pub use methods::dispatch as timestamp_impl;

#[dispatch]
mod methods {
    use crate::types::cel_value::check_timestamp_range;
    use crate::{CelError, CelResult, CelValue};
    use chrono::{DateTime, TimeZone, Utc};

    fn timestamp() -> DateTime<Utc> {
        Utc::now()
    }

    fn timestamp(arg: String) -> CelResult<DateTime<Utc>> {
        let parsed = if let Ok(val) = arg.parse::<DateTime<Utc>>() {
            val
        } else if let Ok(val) = DateTime::parse_from_rfc2822(&arg) {
            val.to_utc()
        } else if let Ok(val) = DateTime::parse_from_rfc3339(&arg) {
            val.to_utc()
        } else {
            return Err(CelError::value("Invalid timestamp format"));
        };

        check_timestamp_range(parsed)
    }

    fn timestamp(arg: i64) -> CelResult<DateTime<Utc>> {
        use chrono::MappedLocalTime;
        match Utc.timestamp_opt(arg, 0) {
            MappedLocalTime::Single(s) => check_timestamp_range(s),
            _ => Err(CelError::value("Invalid timestamp value")),
        }
    }

    fn timestamp(arg: u64) -> CelResult<DateTime<Utc>> {
        use chrono::MappedLocalTime;
        match Utc.timestamp_opt(arg as i64, 0) {
            MappedLocalTime::Single(s) => check_timestamp_range(s),
            _ => Err(CelError::value("Invalid timestamp value")),
        }
    }

    fn timestamp(arg: DateTime<Utc>) -> DateTime<Utc> {
        arg
    }
}
