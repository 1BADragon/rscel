use rscel_macro::dispatch;

pub use methods::dispatch as get_day_of_week;

#[dispatch]
mod methods {
    use crate::{CelResult, CelValue};
    use chrono::{DateTime, Datelike, Utc};

    use crate::context::default_funcs::time_funcs::helpers::get_adjusted_datetime;

    fn get_day_of_week(this: DateTime<Utc>) -> i64 {
        this.weekday().num_days_from_sunday() as i64
    }

    fn get_day_of_week(this: DateTime<Utc>, timezone: String) -> CelResult<i64> {
        Ok(get_adjusted_datetime(this, timezone)?
            .weekday()
            .number_from_sunday() as i64)
    }
}
