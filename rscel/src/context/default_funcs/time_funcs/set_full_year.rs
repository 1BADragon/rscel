use crate::macros::dispatch;

pub use methods::dispatch as set_full_year;

#[dispatch]
mod methods {
    use chrono::{DateTime, Datelike, Utc};

    use crate::{
        context::default_funcs::time_funcs::helpers::get_adjusted_datetime, CelError, CelResult,
        CelValue,
    };

    fn set_full_year(this: DateTime<Utc>, year: i64) -> CelResult<DateTime<Utc>> {
        let year = i32::try_from(year)
            .map_err(|_| CelError::argument("setFullYear() year out of range"))?;
        this.with_year(year)
            .ok_or_else(|| CelError::argument("setFullYear() year out of range"))
    }

    fn set_full_year(this: DateTime<Utc>, year: u64) -> CelResult<DateTime<Utc>> {
        let year = i32::try_from(year)
            .map_err(|_| CelError::argument("setFullYear() year out of range"))?;
        this.with_year(year)
            .ok_or_else(|| CelError::argument("setFullYear() year out of range"))
    }

    fn set_full_year(
        this: DateTime<Utc>,
        year: i64,
        timezone: String,
    ) -> CelResult<DateTime<Utc>> {
        let year = i32::try_from(year)
            .map_err(|_| CelError::argument("setFullYear() year out of range"))?;
        let adjusted = get_adjusted_datetime(this, timezone)?;
        let updated = adjusted
            .with_year(year)
            .ok_or_else(|| CelError::argument("setFullYear() year out of range"))?;
        Ok(updated.with_timezone(&Utc))
    }

    fn set_full_year(
        this: DateTime<Utc>,
        year: u64,
        timezone: String,
    ) -> CelResult<DateTime<Utc>> {
        let year = i32::try_from(year)
            .map_err(|_| CelError::argument("setFullYear() year out of range"))?;
        let adjusted = get_adjusted_datetime(this, timezone)?;
        let updated = adjusted
            .with_year(year)
            .ok_or_else(|| CelError::argument("setFullYear() year out of range"))?;
        Ok(updated.with_timezone(&Utc))
    }
}
