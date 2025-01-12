use crate::macros::dispatch;

pub use methods::dispatch as get_month;

#[dispatch]
mod methods {
    use chrono::{DateTime, Datelike, Utc};

    use crate::{
        context::default_funcs::time_funcs::helpers::get_adjusted_datetime, CelResult, CelValue,
    };

    fn get_month(this: DateTime<Utc>) -> i64 {
        this.month0() as i64
    }

    fn get_month(this: DateTime<Utc>, timezone: String) -> CelResult<i64> {
        Ok(get_adjusted_datetime(this, timezone)?.month0() as i64)
    }
}
