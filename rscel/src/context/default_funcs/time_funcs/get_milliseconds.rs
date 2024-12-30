use crate::macros::dispatch;

pub use methods::dispatch as get_milliseconds;

#[dispatch]
mod methods {
    use chrono::{DateTime, Duration, Utc};

    use crate::{
        context::default_funcs::time_funcs::helpers::get_adjusted_datetime, CelResult, CelValue,
    };

    fn get_milliseconds(this: DateTime<Utc>) -> i64 {
        this.timestamp_subsec_millis() as i64
    }

    fn get_milliseconds(this: DateTime<Utc>, timezone: String) -> CelResult<i64> {
        Ok(get_adjusted_datetime(this, timezone)?.timestamp_subsec_millis() as i64)
    }

    fn get_milliseconds(this: Duration) -> i64 {
        this.subsec_nanos() as i64 / 1000000i64
    }
}
