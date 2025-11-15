use rscel_macro::dispatch;

pub use methods::dispatch as duration_impl;

#[dispatch]
mod methods {
    use crate::{CelError, CelResult, CelValue};
    use chrono::Duration;

    fn duration(arg: String) -> CelResult<Duration> {
        duration_str::parse_chrono(&arg).map_err(|_| CelError::value("Invalid duration format"))
    }

    fn duration(arg: i64) -> CelResult<Duration> {
        Duration::new(arg, 0).ok_or_else(|| CelError::value("Invalid argument for duration"))
    }

    fn duration(arg: Duration) -> Duration {
        arg
    }

    fn duration(seconds: i64, nanos: i64) -> CelResult<Duration> {
        Duration::new(seconds, nanos as u32)
            .ok_or_else(|| CelError::value("Invalid argument for duration"))
    }
}
