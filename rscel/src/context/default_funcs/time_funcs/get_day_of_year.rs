use rscel_macro::dispatch;

pub use methods::dispatch as get_day_of_year;

#[dispatch]
mod methods {
    use chrono::{DateTime, Datelike, Utc};

    use crate::{
        context::default_funcs::time_funcs::helpers::get_adjusted_datetime, CelResult, CelValue,
    };

    fn get_day_of_year(this: DateTime<Utc>) -> i64 {
        this.ordinal0() as i64
    }

    fn get_day_of_year(this: DateTime<Utc>, timezone: String) -> CelResult<i64> {
        Ok(get_adjusted_datetime(this, timezone)?.ordinal0() as i64)
    }
}
