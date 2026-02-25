use crate::macros::dispatch;

pub use methods::dispatch as start_of_day;

#[dispatch]
mod methods {
    use chrono::{DateTime, NaiveTime, Utc};

    use crate::{
        context::default_funcs::time_funcs::helpers::get_adjusted_datetime, CelError, CelResult,
        CelValue,
    };

    fn start_of_day(this: DateTime<Utc>) -> CelResult<DateTime<Utc>> {
        this.with_time(NaiveTime::MIN)
            .single()
            .ok_or_else(|| CelError::value("startOfDay() could not compute start of day"))
    }

    fn start_of_day(this: DateTime<Utc>, timezone: String) -> CelResult<DateTime<Utc>> {
        let adjusted = get_adjusted_datetime(this, timezone)?;
        let midnight = adjusted
            .with_time(NaiveTime::MIN)
            .single()
            .ok_or_else(|| CelError::value("startOfDay() could not compute start of day"))?;
        Ok(midnight.with_timezone(&Utc))
    }
}
