use crate::macros::dispatch;

pub use methods::dispatch as to_rfc3339;

#[dispatch]
mod methods {
    use chrono::{DateTime, Utc};

    use crate::{
        context::default_funcs::time_funcs::helpers::get_adjusted_datetime, CelResult, CelValue,
    };

    fn to_rfc3339(this: DateTime<Utc>) -> String {
        this.to_rfc3339()
    }

    fn to_rfc3339(this: DateTime<Utc>, timezone: String) -> CelResult<String> {
        Ok(get_adjusted_datetime(this, timezone)?.to_rfc3339())
    }
}
