use crate::macros::dispatch;

pub use methods::dispatch as set_hours;

#[dispatch]
mod methods {
    use chrono::{DateTime, Timelike, Utc};

    use crate::{
        context::default_funcs::time_funcs::helpers::get_adjusted_datetime, CelError, CelResult,
        CelValue,
    };

    fn set_hours(this: DateTime<Utc>, hour: i64) -> CelResult<DateTime<Utc>> {
        let hour = u32::try_from(hour)
            .map_err(|_| CelError::argument("setHours() hour out of range"))?;
        this.with_hour(hour)
            .ok_or_else(|| CelError::argument("setHours() hour out of range"))
    }

    fn set_hours(this: DateTime<Utc>, hour: u64) -> CelResult<DateTime<Utc>> {
        let hour = u32::try_from(hour)
            .map_err(|_| CelError::argument("setHours() hour out of range"))?;
        this.with_hour(hour)
            .ok_or_else(|| CelError::argument("setHours() hour out of range"))
    }

    fn set_hours(
        this: DateTime<Utc>,
        hour: i64,
        timezone: String,
    ) -> CelResult<DateTime<Utc>> {
        let hour = u32::try_from(hour)
            .map_err(|_| CelError::argument("setHours() hour out of range"))?;
        let adjusted = get_adjusted_datetime(this, timezone)?;
        let updated = adjusted
            .with_hour(hour)
            .ok_or_else(|| CelError::argument("setHours() hour out of range"))?;
        Ok(updated.with_timezone(&Utc))
    }

    fn set_hours(
        this: DateTime<Utc>,
        hour: u64,
        timezone: String,
    ) -> CelResult<DateTime<Utc>> {
        let hour = u32::try_from(hour)
            .map_err(|_| CelError::argument("setHours() hour out of range"))?;
        let adjusted = get_adjusted_datetime(this, timezone)?;
        let updated = adjusted
            .with_hour(hour)
            .ok_or_else(|| CelError::argument("setHours() hour out of range"))?;
        Ok(updated.with_timezone(&Utc))
    }
}
