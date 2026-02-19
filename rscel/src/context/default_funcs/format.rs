use crate::macros::dispatch;

pub use methods::dispatch as format;

#[dispatch]
mod methods {
    use chrono::{DateTime, Utc};

    use crate::{
        context::default_funcs::time_funcs::helpers::get_adjusted_datetime, CelResult, CelValue,
    };

    fn format(timestamp: DateTime<Utc>, opts: String) -> String {
        return timestamp.format(&opts).to_string();
    }

    fn format(timestamp: DateTime<Utc>, timezone: String, opts: String) -> CelResult<String> {
        return Ok(get_adjusted_datetime(timestamp, timezone)?
            .format(&opts)
            .to_string());
    }

    fn format(this: DateTime<Utc>, opts: String) -> String {
        return this.format(&opts).to_string();
    }

    fn format(this: DateTime<Utc>, timezone: String, opts: String) -> CelResult<String> {
        return Ok(get_adjusted_datetime(this, timezone)?
            .format(&opts)
            .to_string());
    }
}
