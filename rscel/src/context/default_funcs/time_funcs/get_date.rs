use rscel_macro::dispatch;

pub use methods::dispatch as get_date;

#[dispatch]
mod methods {
    use super::super::helpers::get_adjusted_datetime;
    use crate::{CelResult, CelValue};
    use chrono::{DateTime, Datelike, Utc};

    fn get_date(this: DateTime<Utc>) -> i64 {
        this.day() as i64
    }

    fn get_date(this: DateTime<Utc>, timezone: String) -> CelResult<i64> {
        Ok(get_adjusted_datetime(this, timezone)?.day() as i64)
    }
}
