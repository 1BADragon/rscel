use crate::macros::dispatch;

pub use methods::dispatch as set_seconds;

#[dispatch]
mod methods {
    use chrono::{DateTime, Timelike, Utc};

    use crate::{
        context::default_funcs::time_funcs::helpers::get_adjusted_datetime, CelError, CelResult,
        CelValue,
    };

    fn set_seconds(this: DateTime<Utc>, second: i64) -> CelResult<DateTime<Utc>> {
        let second = u32::try_from(second)
            .map_err(|_| CelError::argument("setSeconds() second out of range"))?;
        this.with_second(second)
            .ok_or_else(|| CelError::argument("setSeconds() second out of range"))
    }

    fn set_seconds(this: DateTime<Utc>, second: u64) -> CelResult<DateTime<Utc>> {
        let second = u32::try_from(second)
            .map_err(|_| CelError::argument("setSeconds() second out of range"))?;
        this.with_second(second)
            .ok_or_else(|| CelError::argument("setSeconds() second out of range"))
    }

    fn set_seconds(
        this: DateTime<Utc>,
        second: i64,
        timezone: String,
    ) -> CelResult<DateTime<Utc>> {
        let second = u32::try_from(second)
            .map_err(|_| CelError::argument("setSeconds() second out of range"))?;
        let adjusted = get_adjusted_datetime(this, timezone)?;
        let updated = adjusted
            .with_second(second)
            .ok_or_else(|| CelError::argument("setSeconds() second out of range"))?;
        Ok(updated.with_timezone(&Utc))
    }

    fn set_seconds(
        this: DateTime<Utc>,
        second: u64,
        timezone: String,
    ) -> CelResult<DateTime<Utc>> {
        let second = u32::try_from(second)
            .map_err(|_| CelError::argument("setSeconds() second out of range"))?;
        let adjusted = get_adjusted_datetime(this, timezone)?;
        let updated = adjusted
            .with_second(second)
            .ok_or_else(|| CelError::argument("setSeconds() second out of range"))?;
        Ok(updated.with_timezone(&Utc))
    }
}
