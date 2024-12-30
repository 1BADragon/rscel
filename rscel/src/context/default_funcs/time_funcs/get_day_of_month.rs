use rscel_macro::dispatch;

pub use methods::dispatch as get_day_of_month;

#[dispatch]
mod methods {
    use crate::{
        context::default_funcs::time_funcs::helpers::get_adjusted_datetime, CelResult, CelValue,
    };
    use chrono::{DateTime, Datelike, Utc};

    fn get_day_of_month(this: DateTime<Utc>) -> i64 {
        this.day() as i64 - 1
    }

    fn get_day_of_month(this: DateTime<Utc>, timezone: String) -> CelResult<i64> {
        Ok(get_adjusted_datetime(this, timezone)?.day() as i64 - 1)
    }
}
