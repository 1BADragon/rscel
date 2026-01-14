use crate::macros::dispatch;

pub use methods::dispatch as set_month;

#[dispatch]
mod methods {
    use chrono::{DateTime, Datelike, Utc};

    use crate::{
        context::default_funcs::time_funcs::helpers::get_adjusted_datetime, CelError, CelResult,
        CelValue,
    };

    fn set_month(this: DateTime<Utc>, month: i64) -> CelResult<DateTime<Utc>> {
        let month = u32::try_from(month)
            .map_err(|_| CelError::argument("setMonth() month out of range"))?;
        this.with_month0(month)
            .ok_or_else(|| CelError::argument("setMonth() month out of range"))
    }

    fn set_month(this: DateTime<Utc>, month: u64) -> CelResult<DateTime<Utc>> {
        let month = u32::try_from(month)
            .map_err(|_| CelError::argument("setMonth() month out of range"))?;
        this.with_month0(month)
            .ok_or_else(|| CelError::argument("setMonth() month out of range"))
    }

    fn set_month(
        this: DateTime<Utc>,
        month: i64,
        timezone: String,
    ) -> CelResult<DateTime<Utc>> {
        let month = u32::try_from(month)
            .map_err(|_| CelError::argument("setMonth() month out of range"))?;
        let adjusted = get_adjusted_datetime(this, timezone)?;
        let updated = adjusted
            .with_month0(month)
            .ok_or_else(|| CelError::argument("setMonth() month out of range"))?;
        Ok(updated.with_timezone(&Utc))
    }

    fn set_month(
        this: DateTime<Utc>,
        month: u64,
        timezone: String,
    ) -> CelResult<DateTime<Utc>> {
        let month = u32::try_from(month)
            .map_err(|_| CelError::argument("setMonth() month out of range"))?;
        let adjusted = get_adjusted_datetime(this, timezone)?;
        let updated = adjusted
            .with_month0(month)
            .ok_or_else(|| CelError::argument("setMonth() month out of range"))?;
        Ok(updated.with_timezone(&Utc))
    }
}
