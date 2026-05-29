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
    fn test_invalid_timezone_display_includes_timezone() {
        let error = PoSQLTimestampError::InvalidTimezone {
            timezone: "Etc/NotAZone".to_string(),
        };

        assert_eq!(error.to_string(), "invalid timezone string: Etc/NotAZone");
    }

    #[test]
    fn test_invalid_timezone_offset_display() {
        let error = PoSQLTimestampError::InvalidTimezoneOffset;

        assert_eq!(error.to_string(), "invalid timezone offset");
    }

    #[test]
    fn test_invalid_time_unit_display() {
        let error = PoSQLTimestampError::InvalidTimeUnit {
            error: "weeks".to_string(),
        };

        assert_eq!(error.to_string(), "Invalid time unit");
    }

    #[test]
    fn test_unsupported_precision_display_includes_error() {
        let error = PoSQLTimestampError::UnsupportedPrecision {
            error: "12".to_string(),
        };

        assert_eq!(error.to_string(), "Unsupported precision for timestamp: 12");
    }

    #[test]
    fn test_timestamp_error_converts_to_string() {
        let error = PoSQLTimestampError::UnsupportedPrecision {
            error: "4".to_string(),
        };

        let message = String::from(error);

        assert_eq!(message, "Unsupported precision for timestamp: 4");
    }
}
