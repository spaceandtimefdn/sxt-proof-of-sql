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
    use alloc::string::ToString;

    fn assert_json_round_trip(error: PoSQLTimestampError) {
        let serialized = serde_json::to_string(&error).unwrap();
        let deserialized: PoSQLTimestampError = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, error);
    }

    #[test]
    fn timestamp_errors_display_expected_messages() {
        assert_eq!(
            PoSQLTimestampError::InvalidTimezone {
                timezone: "mars".into()
            }
            .to_string(),
            "invalid timezone string: mars"
        );
        assert_eq!(
            PoSQLTimestampError::InvalidTimezoneOffset.to_string(),
            "invalid timezone offset"
        );
        assert_eq!(
            PoSQLTimestampError::InvalidTimeUnit {
                error: "fortnight".into()
            }
            .to_string(),
            "Invalid time unit"
        );
        assert_eq!(
            PoSQLTimestampError::UnsupportedPrecision { error: "7".into() }.to_string(),
            "Unsupported precision for timestamp: 7"
        );
    }

    #[test]
    fn timestamp_errors_convert_into_strings() {
        let converted: String =
            PoSQLTimestampError::UnsupportedPrecision { error: "2".into() }.into();
        assert_eq!(converted, "Unsupported precision for timestamp: 2");
    }

    #[test]
    fn timestamp_errors_serde_round_trip() {
        assert_json_round_trip(PoSQLTimestampError::InvalidTimezone {
            timezone: "UTC+99".into(),
        });
        assert_json_round_trip(PoSQLTimestampError::InvalidTimezoneOffset);
        assert_json_round_trip(PoSQLTimestampError::InvalidTimeUnit {
            error: "minutes".into(),
        });
        assert_json_round_trip(PoSQLTimestampError::UnsupportedPrecision { error: "10".into() });
    }
}
