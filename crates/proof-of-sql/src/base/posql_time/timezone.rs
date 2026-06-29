use super::PoSQLTimestampError;
use alloc::sync::Arc;
use core::fmt;
use serde::{Deserialize, Serialize};

/// Captures a timezone from a timestamp query
#[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize, PartialEq, Eq)]
pub struct PoSQLTimeZone {
    offset: i32,
}

impl PoSQLTimeZone {
    /// Create a timezone from a count of seconds
    #[must_use]
    pub const fn new(offset: i32) -> Self {
        PoSQLTimeZone { offset }
    }
    #[must_use]
    /// The UTC timezone
    pub const fn utc() -> Self {
        PoSQLTimeZone::new(0)
    }
    /// Get the underlying offset in seconds
    #[must_use]
    pub const fn offset(self) -> i32 {
        self.offset
    }
}

impl TryFrom<&Option<Arc<str>>> for PoSQLTimeZone {
    type Error = PoSQLTimestampError;

    fn try_from(value: &Option<Arc<str>>) -> Result<Self, Self::Error> {
        match value {
            Some(tz_str) => {
                let tz = Arc::as_ref(tz_str).to_uppercase();
                match tz.as_str() {
                    "Z" | "UTC" | "00:00" | "+00:00" | "0:00" | "+0:00" => Ok(PoSQLTimeZone::utc()),
                    tz if tz.chars().count() == 6
                        && (tz.starts_with('+') || tz.starts_with('-')) =>
                    {
                        let sign = if tz.starts_with('-') { -1 } else { 1 };
                        let hours = tz[1..3]
                            .parse::<i32>()
                            .map_err(|_| PoSQLTimestampError::InvalidTimezoneOffset)?;
                        let minutes = tz[4..6]
                            .parse::<i32>()
                            .map_err(|_| PoSQLTimestampError::InvalidTimezoneOffset)?;
                        let total_seconds = sign * ((hours * 3600) + (minutes * 60));
                        Ok(PoSQLTimeZone::new(total_seconds))
                    }
                    _ => Err(PoSQLTimestampError::InvalidTimezone {
                        timezone: tz.clone(),
                    }),
                }
            }
            None => Ok(PoSQLTimeZone::utc()),
        }
    }
}

impl fmt::Display for PoSQLTimeZone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let seconds = self.offset();
        let hours = seconds / 3600;
        let minutes = (seconds.abs() % 3600) / 60;
        if seconds < 0 {
            write!(f, "-{:02}:{:02}", hours.abs(), minutes)
        } else {
            write!(f, "+{hours:02}:{minutes:02}")
        }
    }
}

#[cfg(test)]
mod timezone_parsing_tests {
    use super::*;
    use alloc::format;

    #[test]
    fn test_display_fixed_offset_positive() {
        let timezone = PoSQLTimeZone::new(4500); // +01:15
        assert_eq!(format!("{timezone}"), "+01:15");
    }

    #[test]
    fn test_display_fixed_offset_negative() {
        let timezone = PoSQLTimeZone::new(-3780); // -01:03
        assert_eq!(format!("{timezone}"), "-01:03");
    }

    #[test]
    fn test_display_utc() {
        let timezone = PoSQLTimeZone::utc();
        assert_eq!(format!("{timezone}"), "+00:00");
    }

    #[test]
    fn we_can_parse_time_zone() {
        let timezone_as_str: Option<Arc<str>> = Some("-01:03".into());
        let timezone = PoSQLTimeZone::try_from(&timezone_as_str).unwrap();
        let expected_time_zone = PoSQLTimeZone::new(-3780);
        assert_eq!(timezone, expected_time_zone);
    }

    #[test]
    fn we_can_parse_none_time_zone() {
        let timezone = PoSQLTimeZone::try_from(&None).unwrap();
        let expected_time_zone = PoSQLTimeZone::utc();
        assert_eq!(timezone, expected_time_zone);
    }

    #[test]
    fn we_cannot_parse_invalid_time_zone() {
        let timezone_as_str: Option<Arc<str>> = Some("111111111".into());
        let timezone_err = PoSQLTimeZone::try_from(&timezone_as_str).unwrap_err();
        assert!(matches!(
            timezone_err,
            PoSQLTimestampError::InvalidTimezone { timezone: _ }
        ));
    }

    #[test]
    fn we_can_parse_utc_aliases() {
        for alias in ["Z", "UTC", "utc", "z", "00:00", "+00:00", "0:00", "+0:00"] {
            let tz_str: Option<Arc<str>> = Some(alias.into());
            assert_eq!(
                PoSQLTimeZone::try_from(&tz_str).unwrap(),
                PoSQLTimeZone::utc(),
                "`{alias}` should parse as UTC"
            );
        }
    }

    #[test]
    fn we_can_parse_positive_fixed_offset() {
        let tz_str: Option<Arc<str>> = Some("+02:30".into());
        let timezone = PoSQLTimeZone::try_from(&tz_str).unwrap();
        // +02:30 == 2 * 3600 + 30 * 60 == 9000 seconds east of UTC
        assert_eq!(timezone, PoSQLTimeZone::new(9000));
    }

    #[test]
    fn parsed_offsets_round_trip_through_display() {
        for offset in ["+02:30", "-05:45", "+00:00"] {
            let tz_str: Option<Arc<str>> = Some(offset.into());
            let timezone = PoSQLTimeZone::try_from(&tz_str).unwrap();
            assert_eq!(format!("{timezone}"), offset);
        }
    }

    #[test]
    fn we_cannot_parse_offset_with_non_numeric_hours() {
        let tz_str: Option<Arc<str>> = Some("+ab:00".into());
        assert!(matches!(
            PoSQLTimeZone::try_from(&tz_str).unwrap_err(),
            PoSQLTimestampError::InvalidTimezoneOffset
        ));
    }

    #[test]
    fn we_cannot_parse_offset_with_non_numeric_minutes() {
        let tz_str: Option<Arc<str>> = Some("-01:zz".into());
        assert!(matches!(
            PoSQLTimeZone::try_from(&tz_str).unwrap_err(),
            PoSQLTimestampError::InvalidTimezoneOffset
        ));
    }
}
