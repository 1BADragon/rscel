use crate::macros::dispatch;

pub use methods::dispatch as get_hours;

#[dispatch]
mod methods {
    use chrono::{DateTime, Duration, Timelike, Utc};

    use crate::{
        context::default_funcs::time_funcs::helpers::get_adjusted_datetime, CelResult, CelValue,
    };

    fn get_hours(this: DateTime<Utc>) -> i64 {
        this.time().hour() as i64
    }

    fn get_hours(this: DateTime<Utc>, timezone: String) -> CelResult<i64> {
        Ok(get_adjusted_datetime(this, timezone)?.time().hour() as i64)
    }

    fn get_hours(this: Duration) -> i64 {
        this.num_hours()
    }
}
