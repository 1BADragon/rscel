use crate::macros::dispatch;

pub use methods::dispatch as set_minutes;

#[dispatch]
mod methods {
    use chrono::{DateTime, Timelike, Utc};

    use crate::{
        context::default_funcs::time_funcs::helpers::get_adjusted_datetime, CelError, CelResult,
        CelValue,
    };

    fn set_minutes(this: DateTime<Utc>, minute: i64) -> CelResult<DateTime<Utc>> {
        let minute = u32::try_from(minute)
            .map_err(|_| CelError::argument("setMinutes() minute out of range"))?;
        this.with_minute(minute)
            .ok_or_else(|| CelError::argument("setMinutes() minute out of range"))
    }

    fn set_minutes(this: DateTime<Utc>, minute: u64) -> CelResult<DateTime<Utc>> {
        let minute = u32::try_from(minute)
            .map_err(|_| CelError::argument("setMinutes() minute out of range"))?;
        this.with_minute(minute)
            .ok_or_else(|| CelError::argument("setMinutes() minute out of range"))
    }

    fn set_minutes(
        this: DateTime<Utc>,
        minute: i64,
        timezone: String,
    ) -> CelResult<DateTime<Utc>> {
        let minute = u32::try_from(minute)
            .map_err(|_| CelError::argument("setMinutes() minute out of range"))?;
        let adjusted = get_adjusted_datetime(this, timezone)?;
        let updated = adjusted
            .with_minute(minute)
            .ok_or_else(|| CelError::argument("setMinutes() minute out of range"))?;
        Ok(updated.with_timezone(&Utc))
    }

    fn set_minutes(
        this: DateTime<Utc>,
        minute: u64,
        timezone: String,
    ) -> CelResult<DateTime<Utc>> {
        let minute = u32::try_from(minute)
            .map_err(|_| CelError::argument("setMinutes() minute out of range"))?;
        let adjusted = get_adjusted_datetime(this, timezone)?;
        let updated = adjusted
            .with_minute(minute)
            .ok_or_else(|| CelError::argument("setMinutes() minute out of range"))?;
        Ok(updated.with_timezone(&Utc))
    }
}
