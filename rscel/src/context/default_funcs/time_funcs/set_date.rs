use crate::macros::dispatch;

pub use methods::dispatch as set_date;

#[dispatch]
mod methods {
    use chrono::{DateTime, Datelike, Utc};

    use crate::{
        context::default_funcs::time_funcs::helpers::get_adjusted_datetime, CelError, CelResult,
        CelValue,
    };

    fn set_date(this: DateTime<Utc>, day: i64) -> CelResult<DateTime<Utc>> {
        let day = u32::try_from(day)
            .map_err(|_| CelError::argument("setDate() day out of range"))?;
        this.with_day(day)
            .ok_or_else(|| CelError::argument("setDate() day out of range"))
    }

    fn set_date(this: DateTime<Utc>, day: u64) -> CelResult<DateTime<Utc>> {
        let day = u32::try_from(day)
            .map_err(|_| CelError::argument("setDate() day out of range"))?;
        this.with_day(day)
            .ok_or_else(|| CelError::argument("setDate() day out of range"))
    }

    fn set_date(
        this: DateTime<Utc>,
        day: i64,
        timezone: String,
    ) -> CelResult<DateTime<Utc>> {
        let day = u32::try_from(day)
            .map_err(|_| CelError::argument("setDate() day out of range"))?;
        let adjusted = get_adjusted_datetime(this, timezone)?;
        let updated = adjusted
            .with_day(day)
            .ok_or_else(|| CelError::argument("setDate() day out of range"))?;
        Ok(updated.with_timezone(&Utc))
    }

    fn set_date(
        this: DateTime<Utc>,
        day: u64,
        timezone: String,
    ) -> CelResult<DateTime<Utc>> {
        let day = u32::try_from(day)
            .map_err(|_| CelError::argument("setDate() day out of range"))?;
        let adjusted = get_adjusted_datetime(this, timezone)?;
        let updated = adjusted
            .with_day(day)
            .ok_or_else(|| CelError::argument("setDate() day out of range"))?;
        Ok(updated.with_timezone(&Utc))
    }
}
