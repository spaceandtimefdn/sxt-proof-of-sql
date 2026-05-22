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
    use super::PoSQLTimestampError;
    use alloc::string::{String, ToString};

    #[test]
    fn we_can_display_timestamp_errors() {
        assert_eq!(
            PoSQLTimestampError::InvalidTimezone {
                timezone: "Mars/Phobos".to_string()
            }
            .to_string(),
            "invalid timezone string: Mars/Phobos"
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
                error: "picoseconds".to_string()
            }
            .to_string(),
            "Unsupported precision for timestamp: picoseconds"
        );
    }

    #[test]
    fn we_can_convert_timestamp_errors_into_strings() {
        let error = PoSQLTimestampError::UnsupportedPrecision {
            error: "weeks".to_string(),
        };
        let message: String = error.into();

        assert_eq!(message, "Unsupported precision for timestamp: weeks");
    }

    #[test]
    fn timestamp_errors_have_stable_equality_and_serde_roundtrip() {
        let error = PoSQLTimestampError::InvalidTimeUnit {
            error: "century".to_string(),
        };
        let bytes = bincode::serde::encode_to_vec(&error, bincode::config::legacy()).unwrap();
        let (roundtripped, consumed): (PoSQLTimestampError, usize) =
            bincode::serde::decode_from_slice(&bytes, bincode::config::legacy()).unwrap();

        assert_eq!(consumed, bytes.len());
        assert_eq!(roundtripped, error);
    }
}
