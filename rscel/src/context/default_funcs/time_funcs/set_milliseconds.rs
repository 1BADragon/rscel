use crate::macros::dispatch;

pub use methods::dispatch as set_milliseconds;

#[dispatch]
mod methods {
    use chrono::{DateTime, Timelike, Utc};

    use crate::{
        context::default_funcs::time_funcs::helpers::get_adjusted_datetime, CelError, CelResult,
        CelValue,
    };

    fn set_milliseconds(this: DateTime<Utc>, millisecond: i64) -> CelResult<DateTime<Utc>> {
        let millisecond = u32::try_from(millisecond)
            .map_err(|_| CelError::argument("setMilliseconds() millisecond out of range"))?;
        if millisecond > 999 {
            return Err(CelError::argument(
                "setMilliseconds() millisecond out of range",
            ));
        }
        let nanos = millisecond * 1_000_000;
        this.with_nanosecond(nanos)
            .ok_or_else(|| CelError::argument("setMilliseconds() millisecond out of range"))
    }

    fn set_milliseconds(this: DateTime<Utc>, millisecond: u64) -> CelResult<DateTime<Utc>> {
        let millisecond = u32::try_from(millisecond)
            .map_err(|_| CelError::argument("setMilliseconds() millisecond out of range"))?;
        if millisecond > 999 {
            return Err(CelError::argument(
                "setMilliseconds() millisecond out of range",
            ));
        }
        let nanos = millisecond * 1_000_000;
        this.with_nanosecond(nanos)
            .ok_or_else(|| CelError::argument("setMilliseconds() millisecond out of range"))
    }

    fn set_milliseconds(
        this: DateTime<Utc>,
        millisecond: i64,
        timezone: String,
    ) -> CelResult<DateTime<Utc>> {
        let millisecond = u32::try_from(millisecond)
            .map_err(|_| CelError::argument("setMilliseconds() millisecond out of range"))?;
        if millisecond > 999 {
            return Err(CelError::argument(
                "setMilliseconds() millisecond out of range",
            ));
        }
        let nanos = millisecond * 1_000_000;
        let adjusted = get_adjusted_datetime(this, timezone)?;
        let updated = adjusted
            .with_nanosecond(nanos)
            .ok_or_else(|| CelError::argument("setMilliseconds() millisecond out of range"))?;
        Ok(updated.with_timezone(&Utc))
    }

    fn set_milliseconds(
        this: DateTime<Utc>,
        millisecond: u64,
        timezone: String,
    ) -> CelResult<DateTime<Utc>> {
        let millisecond = u32::try_from(millisecond)
            .map_err(|_| CelError::argument("setMilliseconds() millisecond out of range"))?;
        if millisecond > 999 {
            return Err(CelError::argument(
                "setMilliseconds() millisecond out of range",
            ));
        }
        let nanos = millisecond * 1_000_000;
        let adjusted = get_adjusted_datetime(this, timezone)?;
        let updated = adjusted
            .with_nanosecond(nanos)
            .ok_or_else(|| CelError::argument("setMilliseconds() millisecond out of range"))?;
        Ok(updated.with_timezone(&Utc))
    }
}
