use crate::macros::dispatch;

pub use methods::dispatch as get_minutes;

#[dispatch]
mod methods {
    use chrono::{DateTime, Duration, Timelike, Utc};

    use crate::{
        context::default_funcs::time_funcs::helpers::get_adjusted_datetime, CelResult, CelValue,
    };

    fn get_minutes(this: DateTime<Utc>) -> i64 {
        this.time().minute() as i64
    }

    fn get_minutes(this: DateTime<Utc>, timezone: String) -> CelResult<i64> {
        Ok(get_adjusted_datetime(this, timezone)?.time().minute() as i64)
    }

    fn get_minutes(this: Duration) -> i64 {
        this.num_minutes() as i64
    }
}
