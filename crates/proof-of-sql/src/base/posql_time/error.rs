use alloc::string::{String, ToString};
use serde::{Deserialize, Serialize};
use snafu::Snafu;

/// Errors related to time operations, including timezone and timestamp conversions.
#[derive(Snafu, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PoSQLTimestampError {
    /// Error when the timezone string provided cannot be parsed into a valid timezone.
    #[snafu(display("invalid timezone string: {timezone}"))]
    InvalidTimezone {
        /// The invalid timezone
        timezone: String,
    },

    /// Error indicating an invalid timezone offset was provided.
    #[snafu(display("invalid timezone offset"))]
    InvalidTimezoneOffset,

    /// Indicates a failure to convert between different representations of time units.
    #[snafu(display("Invalid time unit"))]
    InvalidTimeUnit {
        /// The underlying error
        error: String,
    },

    /// Represents a failure to parse a provided time unit precision value, `PoSQL` supports
    /// Seconds, Milliseconds, Microseconds, and Nanoseconds
    #[snafu(display("Unsupported precision for timestamp: {error}"))]
    UnsupportedPrecision {
        /// The underlying error
        error: String,
    },
}

// This exists because TryFrom<DataType> for ColumnType error is String
impl From<PoSQLTimestampError> for String {
    fn from(error: PoSQLTimestampError) -> Self {
        error.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn we_can_display_invalid_timezone_error() {
        let error = PoSQLTimestampError::InvalidTimezone {
            timezone: "Mars/Olympus".to_string(),
        };
        let msg = error.to_string();
        assert!(msg.contains("Mars/Olympus"));
    }

    #[test]
    fn we_can_display_invalid_timezone_offset_error() {
        let error = PoSQLTimestampError::InvalidTimezoneOffset;
        let msg = error.to_string();
        assert!(msg.contains("invalid timezone offset"));
    }

    #[test]
    fn we_can_display_invalid_time_unit_error() {
        let error = PoSQLTimestampError::InvalidTimeUnit {
            error: "bad unit".to_string(),
        };
        let msg = error.to_string();
        assert!(msg.contains("Invalid time unit"));
    }

    #[test]
    fn we_can_display_unsupported_precision_error() {
        let error = PoSQLTimestampError::UnsupportedPrecision {
            error: "12".to_string(),
        };
        let msg = error.to_string();
        assert!(msg.contains("12"));
    }

    #[test]
    fn we_can_convert_posql_timestamp_error_to_string() {
        let error = PoSQLTimestampError::InvalidTimezoneOffset;
        let s: String = error.into();
        assert!(s.contains("invalid timezone offset"));
    }

    #[test]
    fn posql_timestamp_errors_with_same_data_are_equal() {
        let a = PoSQLTimestampError::InvalidTimezone {
            timezone: "foo".to_string(),
        };
        let b = PoSQLTimestampError::InvalidTimezone {
            timezone: "foo".to_string(),
        };
        assert_eq!(a, b);
    }

    #[test]
    fn posql_timestamp_errors_with_different_data_are_not_equal() {
        let a = PoSQLTimestampError::InvalidTimezone {
            timezone: "foo".to_string(),
        };
        let b = PoSQLTimestampError::InvalidTimezone {
            timezone: "bar".to_string(),
        };
        assert_ne!(a, b);
    }
}
