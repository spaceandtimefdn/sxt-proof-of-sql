use super::PoSQLTimestampError;
use alloc::{string::ToString, sync::Arc};
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
                        timezone: tz.to_string(),
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
}

#[cfg(test)]
mod timezone_arc_str_parsing {
    use super::*;
    use crate::base::posql_time::PoSQLTimestampError::InvalidTimezoneOffset;
    use alloc::format;

    #[test]
    fn test_parsing_from_arc_str_fixed_offset() {
        let ss = "00:00";
        let timezone_arc: Arc<str> = Arc::from(ss);
        let timezone = PoSQLTimeZone::try_from(&Some(timezone_arc)).unwrap();
        assert_eq!(format!("{timezone}"), "+00:00");
    }

    #[test]
    fn test_parsing_from_arc_str_fixed_offset_positive() {
        let input_timezone = "+01:15";
        let timezone_arc: Arc<str> = Arc::from(input_timezone);
        let timezone = PoSQLTimeZone::try_from(&Some(timezone_arc)).unwrap();
        assert_eq!(format!("{timezone}"), "+01:15");
    }

    #[test]
    fn test_parsing_from_arc_str_fixed_offset_negative() {
        let input_timezone = "-01:03";
        let timezone_arc: Arc<str> = Arc::from(input_timezone);
        let timezone = PoSQLTimeZone::try_from(&Some(timezone_arc)).unwrap();
        assert_eq!(format!("{timezone}"), "-01:03");
    }

    #[test]
    fn check_for_invalid_timezone_hour_offset() {
        // Test invalid hour format (non-numeric)
        let input_timezone = "-0A:03";
        let timezone_arc: Arc<str> = Arc::from(input_timezone);
        let offset_error = PoSQLTimeZone::try_from(&Some(timezone_arc));
        assert_eq!(offset_error, Err(InvalidTimezoneOffset));

        // Test invalid hour format (non-numeric in hours)
        let input_timezone = "-AB:03";
        let timezone_arc: Arc<str> = Arc::from(input_timezone);
        let offset_error = PoSQLTimeZone::try_from(&Some(timezone_arc));
        assert_eq!(offset_error, Err(InvalidTimezoneOffset));

        // Test that hours > 12 are accepted
        let input_timezone = "-13:03";
        let timezone_arc: Arc<str> = Arc::from(input_timezone);
        let result = PoSQLTimeZone::try_from(&Some(timezone_arc));
        
        assert!(result.is_ok());

        // Test invalid minutes format (non-numeric)
        let input_timezone = "-11:AB";
        let timezone_arc: Arc<str> = Arc::from(input_timezone);
        let offset_error = PoSQLTimeZone::try_from(&Some(timezone_arc));
        assert_eq!(offset_error, Err(InvalidTimezoneOffset));

        // Test that minutes >= 60 are accepted 
        
        let input_timezone = "-11:60";
        let timezone_arc: Arc<str> = Arc::from(input_timezone);
        let result = PoSQLTimeZone::try_from(&Some(timezone_arc));
        
        assert!(result.is_ok());
    }

    #[test]
    fn check_for_invalid_timezone_minute_offset() {
        // Test invalid minute format (non-numeric)
        let input_timezone = "-00:B3";
        let timezone_arc: Arc<str> = Arc::from(input_timezone);
        let offset_error = PoSQLTimeZone::try_from(&Some(timezone_arc));
        assert_eq!(offset_error, Err(InvalidTimezoneOffset));

        // Test invalid minute format (non-numeric in minutes)
        let input_timezone = "-00:AB";
        let timezone_arc: Arc<str> = Arc::from(input_timezone);
        let offset_error = PoSQLTimeZone::try_from(&Some(timezone_arc));
        assert_eq!(offset_error, Err(InvalidTimezoneOffset));

        // Test that minutes >= 60 are accepted 
        let input_timezone = "-00:83";
        let timezone_arc: Arc<str> = Arc::from(input_timezone);
        let result = PoSQLTimeZone::try_from(&Some(timezone_arc));
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_timezone() {
        let expected = PoSQLTimestampError::InvalidTimezone {
            timezone: "WRONG".to_string(),
        };
        let timezone_input = "WRONG";
        let timezone_arc: Arc<str> = Arc::from(timezone_input);
        let timezone_err = PoSQLTimeZone::try_from(&Some(timezone_arc));
        assert_eq!(expected, timezone_err.err().unwrap());
    }

    #[test]
    fn test_when_none() {
        let timezone = PoSQLTimeZone::try_from(&None).unwrap();
        assert_eq!(format!("{timezone}"), "+00:00");
    }
}
