use rscel_macro::dispatch;

pub use methods::dispatch as duration_impl;

#[dispatch]
mod methods {
    use crate::types::cel_value::check_duration_range;
    use crate::{CelError, CelResult, CelValue};
    use chrono::Duration;

    fn duration(arg: String) -> CelResult<Duration> {
        // duration_str does not accept a leading sign, so peel it off and apply it
        // afterwards; CEL durations are signed ("-999999999ns" is valid).
        let (negative, body) = match arg.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, arg.strip_prefix('+').unwrap_or(arg.as_str())),
        };

        let magnitude = duration_str::parse_chrono(body)
            .map_err(|_| CelError::value("Invalid duration format"))?;

        let signed = if negative {
            Duration::zero()
                .checked_sub(&magnitude)
                .ok_or_else(|| CelError::value("Duration out of range"))?
        } else {
            magnitude
        };

        check_duration_range(signed)
    }

    fn duration(arg: i64) -> CelResult<Duration> {
        let d = Duration::new(arg, 0).ok_or_else(|| CelError::value("Invalid argument for duration"))?;
        check_duration_range(d)
    }

    fn duration(arg: Duration) -> CelResult<Duration> {
        check_duration_range(arg)
    }

    fn duration(seconds: i64, nanos: i64) -> CelResult<Duration> {
        let d = Duration::new(seconds, nanos as u32)
            .ok_or_else(|| CelError::value("Invalid argument for duration"))?;
        check_duration_range(d)
    }
}
