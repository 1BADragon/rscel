use std::str::FromStr;

use chrono::{DateTime, Utc};
use chrono_tz::Tz;

use crate::{CelError, CelResult};

pub fn get_adjusted_datetime(this: DateTime<Utc>, timezone: String) -> CelResult<DateTime<Tz>> {
    if let Ok(tz) = Tz::from_str(&timezone) {
        Ok(this.with_timezone(&tz))
    } else {
        Err(CelError::argument("Failed to parse timezone"))
    }
}
