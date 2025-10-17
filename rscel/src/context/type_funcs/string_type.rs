use rscel_macro::dispatch;

pub use methods::dispatch as string_impl;

#[dispatch]
mod methods {
    use crate::types::CelBytes;
    use crate::{CelError, CelResult, CelValue};
    use chrono::{DateTime, Duration, Utc};

    fn string(arg: i64) -> String {
        arg.to_string()
    }

    fn string(arg: u64) -> String {
        arg.to_string()
    }

    fn string(arg: f64) -> String {
        arg.to_string()
    }

    fn string(arg: String) -> String {
        arg
    }

    fn string(arg: CelBytes) -> CelResult<String> {
        String::from_utf8(arg.into()).map_err(|_| CelError::value("Bad bytes in utf8 convertion"))
    }

    fn string(arg: DateTime<Utc>) -> String {
        arg.to_rfc3339()
    }

    fn string(arg: Duration) -> String {
        format!(
            "{}s",
            arg.num_nanoseconds().unwrap() as f64 / 1_000_000_000.0
        )
    }

    fn string(arg: CelValue) -> CelResult<String> {
        Err(CelError::value(&format!(
            "string() invalid for value {:?}",
            arg
        )))
    }
}
