use std::fmt;
use std::str::FromStr;

use chrono::{
    DateTime, FixedOffset, MappedLocalTime, NaiveDate, NaiveDateTime, Offset, TimeZone, Utc,
};
use chrono_tz::{Tz, TzOffset};

use crate::{CelError, CelResult};

/// A CEL timezone argument: either an IANA name ("America/Los_Angeles") or a fixed
/// numeric UTC offset ("+11:00").
///
/// This exists so named zones keep their DST rules. Collapsing them to a fixed offset
/// would be fine for reading calendar fields, but the set*/startOf* functions rebuild a
/// local time through the zone, and a January offset must not be reused for a June date.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CelTz {
    Named(Tz),
    Fixed(FixedOffset),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CelTzOffset {
    Named(TzOffset),
    Fixed(FixedOffset),
}

impl fmt::Display for CelTzOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CelTzOffset::Named(offset) => offset.fmt(f),
            CelTzOffset::Fixed(offset) => offset.fmt(f),
        }
    }
}

impl Offset for CelTzOffset {
    fn fix(&self) -> FixedOffset {
        match self {
            CelTzOffset::Named(offset) => offset.fix(),
            CelTzOffset::Fixed(offset) => offset.fix(),
        }
    }
}

impl TimeZone for CelTz {
    type Offset = CelTzOffset;

    fn from_offset(offset: &Self::Offset) -> Self {
        match offset {
            CelTzOffset::Named(offset) => CelTz::Named(Tz::from_offset(offset)),
            CelTzOffset::Fixed(offset) => CelTz::Fixed(*offset),
        }
    }

    #[allow(deprecated)]
    fn offset_from_local_date(&self, local: &NaiveDate) -> MappedLocalTime<Self::Offset> {
        match self {
            CelTz::Named(tz) => tz.offset_from_local_date(local).map(CelTzOffset::Named),
            CelTz::Fixed(offset) => offset.offset_from_local_date(local).map(CelTzOffset::Fixed),
        }
    }

    fn offset_from_local_datetime(&self, local: &NaiveDateTime) -> MappedLocalTime<Self::Offset> {
        match self {
            CelTz::Named(tz) => tz.offset_from_local_datetime(local).map(CelTzOffset::Named),
            CelTz::Fixed(offset) => offset
                .offset_from_local_datetime(local)
                .map(CelTzOffset::Fixed),
        }
    }

    #[allow(deprecated)]
    fn offset_from_utc_date(&self, utc: &NaiveDate) -> Self::Offset {
        match self {
            CelTz::Named(tz) => CelTzOffset::Named(tz.offset_from_utc_date(utc)),
            CelTz::Fixed(offset) => CelTzOffset::Fixed(offset.offset_from_utc_date(utc)),
        }
    }

    fn offset_from_utc_datetime(&self, utc: &NaiveDateTime) -> Self::Offset {
        match self {
            CelTz::Named(tz) => CelTzOffset::Named(tz.offset_from_utc_datetime(utc)),
            CelTz::Fixed(offset) => CelTzOffset::Fixed(offset.offset_from_utc_datetime(utc)),
        }
    }
}

pub fn get_adjusted_datetime(this: DateTime<Utc>, timezone: String) -> CelResult<DateTime<CelTz>> {
    if let Ok(tz) = Tz::from_str(&timezone) {
        return Ok(this.with_timezone(&CelTz::Named(tz)));
    }

    if let Some(offset) = parse_fixed_offset(&timezone) {
        return Ok(this.with_timezone(&CelTz::Fixed(offset)));
    }

    Err(CelError::Argument(format!(
        "Failed to parse timezone: '{}'",
        timezone
    )))
}

/// Parse a `±HH:MM` UTC offset. The sign is optional and defaults to east, so both
/// "+02:00" and "02:00" are accepted; "-00:00" is UTC.
fn parse_fixed_offset(timezone: &str) -> Option<FixedOffset> {
    let (east, rest) = match timezone.strip_prefix('-') {
        Some(rest) => (false, rest),
        None => (true, timezone.strip_prefix('+').unwrap_or(timezone)),
    };

    let (hours, minutes) = rest.split_once(':')?;
    if hours.len() != 2 || minutes.len() != 2 {
        return None;
    }
    if !hours.bytes().chain(minutes.bytes()).all(|b| b.is_ascii_digit()) {
        return None;
    }

    let hours: i32 = hours.parse().ok()?;
    let minutes: i32 = minutes.parse().ok()?;
    if hours > 23 || minutes > 59 {
        return None;
    }

    let seconds = hours * 3600 + minutes * 60;
    if east {
        FixedOffset::east_opt(seconds)
    } else {
        FixedOffset::west_opt(seconds)
    }
}
