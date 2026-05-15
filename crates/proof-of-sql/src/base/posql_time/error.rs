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
    fn timestamp_errors_render_stable_messages() {
        assert_eq!(
            PoSQLTimestampError::InvalidTimezone {
                timezone: "MARS".to_string()
            }
            .to_string(),
            "invalid timezone string: MARS"
        );
        assert_eq!(
            PoSQLTimestampError::InvalidTimezoneOffset.to_string(),
            "invalid timezone offset"
        );
        assert_eq!(
            PoSQLTimestampError::InvalidTimeUnit {
                error: "fortnight".to_string()
            }
            .to_string(),
            "Invalid time unit"
        );
        assert_eq!(
            PoSQLTimestampError::UnsupportedPrecision {
                error: "4".to_string()
            }
            .to_string(),
            "Unsupported precision for timestamp: 4"
        );
    }

    #[test]
    fn timestamp_errors_convert_into_strings() {
        let message: String = PoSQLTimestampError::UnsupportedPrecision {
            error: "12".to_string(),
        }
        .into();

        assert_eq!(message, "Unsupported precision for timestamp: 12");
    }

    #[test]
    fn timestamp_errors_serde_round_trip() {
        let error = PoSQLTimestampError::InvalidTimezone {
            timezone: "NOT_A_ZONE".to_string(),
        };

        let serialized = serde_json::to_string(&error).unwrap();
        let deserialized: PoSQLTimestampError = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, error);
    }
}
