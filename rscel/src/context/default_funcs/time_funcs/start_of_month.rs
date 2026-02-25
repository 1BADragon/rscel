use crate::macros::dispatch;

pub use methods::dispatch as start_of_month;

#[dispatch]
mod methods {
    use chrono::{DateTime, Datelike, NaiveDate, NaiveTime, TimeZone, Utc};

    use crate::{
        context::default_funcs::time_funcs::helpers::get_adjusted_datetime, CelError, CelResult,
        CelValue,
    };

    fn start_of_month(this: DateTime<Utc>) -> CelResult<DateTime<Utc>> {
        let d = NaiveDate::from_ymd_opt(this.year(), this.month(), 1)
            .ok_or_else(|| CelError::value("startOfMonth() could not compute start of month"))?;
        Ok(d.and_time(NaiveTime::MIN).and_utc())
    }

    fn start_of_month(this: DateTime<Utc>, timezone: String) -> CelResult<DateTime<Utc>> {
        let adjusted = get_adjusted_datetime(this, timezone)?;
        let d = NaiveDate::from_ymd_opt(adjusted.year(), adjusted.month(), 1)
            .ok_or_else(|| CelError::value("startOfMonth() could not compute start of month"))?;
        let midnight = d.and_time(NaiveTime::MIN);
        let tz = adjusted.timezone();
        tz.from_local_datetime(&midnight)
            .single()
            .map(|dt| dt.with_timezone(&Utc))
            .ok_or_else(|| CelError::value("startOfMonth() ambiguous local time"))
    }
}
