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
    fn we_can_convert_timestamp_errors_to_strings() {
        assert_eq!(
            String::from(PoSQLTimestampError::InvalidTimezone {
                timezone: "Mars/Base".to_string()
            }),
            "invalid timezone string: Mars/Base"
        );
        assert_eq!(
            String::from(PoSQLTimestampError::InvalidTimezoneOffset),
            "invalid timezone offset"
        );
        assert_eq!(
            String::from(PoSQLTimestampError::InvalidTimeUnit {
                error: "fortnight".to_string()
            }),
            "Invalid time unit"
        );
        assert_eq!(
            String::from(PoSQLTimestampError::UnsupportedPrecision {
                error: "2".to_string()
            }),
            "Unsupported precision for timestamp: 2"
        );
    }
}
