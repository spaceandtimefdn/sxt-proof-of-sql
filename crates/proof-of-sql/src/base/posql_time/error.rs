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
    fn timestamp_errors_preserve_payloads() {
        let invalid_timezone = PoSQLTimestampError::InvalidTimezone {
            timezone: "BAD".to_string(),
        };
        assert!(matches!(
            invalid_timezone,
            PoSQLTimestampError::InvalidTimezone { ref timezone } if timezone == "BAD"
        ));

        let invalid_time_unit = PoSQLTimestampError::InvalidTimeUnit {
            error: "hours".to_string(),
        };
        assert!(matches!(
            invalid_time_unit,
            PoSQLTimestampError::InvalidTimeUnit { ref error } if error == "hours"
        ));

        let unsupported_precision = PoSQLTimestampError::UnsupportedPrecision {
            error: "10".to_string(),
        };
        assert!(matches!(
            unsupported_precision,
            PoSQLTimestampError::UnsupportedPrecision { ref error } if error == "10"
        ));
    }

    #[test]
    fn timestamp_errors_display_readable_messages() {
        assert_eq!(
            PoSQLTimestampError::InvalidTimezoneOffset.to_string(),
            "invalid timezone offset"
        );
        assert_eq!(
            PoSQLTimestampError::InvalidTimeUnit {
                error: "hours".to_string()
            }
            .to_string(),
            "Invalid time unit"
        );
        assert_eq!(
            PoSQLTimestampError::UnsupportedPrecision {
                error: "10".to_string()
            }
            .to_string(),
            "Unsupported precision for timestamp: 10"
        );
    }

    #[test]
    fn timestamp_errors_convert_into_strings() {
        let error_string: String = PoSQLTimestampError::UnsupportedPrecision {
            error: "5".to_string(),
        }
        .into();

        assert_eq!(error_string, "Unsupported precision for timestamp: 5");
    }
}
