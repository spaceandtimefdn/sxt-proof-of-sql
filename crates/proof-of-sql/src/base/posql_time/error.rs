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
    use alloc::string::{String, ToString};

    #[test]
    fn timestamp_errors_display_descriptive_messages() {
        let cases = [
            (
                PoSQLTimestampError::InvalidTimezone {
                    timezone: "Mars/Phobos".to_string(),
                },
                "invalid timezone string: Mars/Phobos",
            ),
            (
                PoSQLTimestampError::InvalidTimezoneOffset,
                "invalid timezone offset",
            ),
            (
                PoSQLTimestampError::InvalidTimeUnit {
                    error: "fortnight".to_string(),
                },
                "Invalid time unit",
            ),
            (
                PoSQLTimestampError::UnsupportedPrecision {
                    error: "picoseconds".to_string(),
                },
                "Unsupported precision for timestamp: picoseconds",
            ),
        ];

        for (error, expected) in cases {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn timestamp_error_converts_into_string_via_display() {
        let message: String = PoSQLTimestampError::InvalidTimezone {
            timezone: "Moon/Base".to_string(),
        }
        .into();

        assert_eq!(message, "invalid timezone string: Moon/Base");
    }
}
