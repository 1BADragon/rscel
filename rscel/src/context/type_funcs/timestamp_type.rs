use rscel_macro::dispatch;

pub use methods::dispatch as timestamp_impl;

#[dispatch]
mod methods {
    use crate::{CelError, CelResult, CelValue};
    use chrono::{DateTime, TimeZone, Utc};

    fn timestamp() -> DateTime<Utc> {
        Utc::now()
    }

    fn timestamp(arg: String) -> CelResult<DateTime<Utc>> {
        if let Ok(val) = arg.parse::<DateTime<Utc>>() {
            Ok(val)
        } else if let Ok(val) = DateTime::parse_from_rfc2822(&arg) {
            Ok(val.to_utc())
        } else if let Ok(val) = DateTime::parse_from_rfc3339(&arg) {
            Ok(val.to_utc())
        } else {
            Err(CelError::value("Invalid timestamp format"))
        }
    }

    fn timestamp(arg: i64) -> CelResult<DateTime<Utc>> {
        use chrono::MappedLocalTime;
        match Utc.timestamp_opt(arg, 0) {
            MappedLocalTime::Single(s) => Ok(s),
            _ => Err(CelError::value("Invalid timestamp value")),
        }
    }

    fn timestamp(arg: u64) -> CelResult<DateTime<Utc>> {
        use chrono::MappedLocalTime;
        match Utc.timestamp_opt(arg as i64, 0) {
            MappedLocalTime::Single(s) => Ok(s),
            _ => Err(CelError::value("Invalid timestamp value")),
        }
    }

    fn timestamp(arg: DateTime<Utc>) -> DateTime<Utc> {
        arg
    }
}
