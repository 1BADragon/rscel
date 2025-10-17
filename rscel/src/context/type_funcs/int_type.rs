use rscel_macro::dispatch;

pub use methods::dispatch as int_impl;

#[dispatch]
mod methods {
    use crate::{CelError, CelResult, CelValue};
    use chrono::{DateTime, Utc};

    fn int(arg: i64) -> i64 {
        arg
    }

    fn int(arg: u64) -> i64 {
        arg as i64
    }

    fn int(arg: f64) -> i64 {
        arg as i64
    }

    fn int(arg: bool) -> i64 {
        if arg {
            1
        } else {
            0
        }
    }

    fn int(arg: String) -> CelResult<i64> {
        arg.parse::<i64>()
            .map_err(|_| CelError::value(&format!("int conversion invalid for \"{}\"", arg)))
    }

    fn int(arg: DateTime<Utc>) -> i64 {
        arg.timestamp()
    }
}
