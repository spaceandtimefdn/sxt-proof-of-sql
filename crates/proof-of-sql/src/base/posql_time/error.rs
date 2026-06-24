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

    #[test]
    fn invalid_timezone_displays_timezone_string() {
        let err = PoSQLTimestampError::InvalidTimezone {
            timezone: "Mars/UTC".to_string(),
        };
        assert_eq!(err.to_string(), "invalid timezone string: Mars/UTC");
    }

    #[test]
    fn invalid_timezone_offset_displays_correct_message() {
        assert_eq!(
            PoSQLTimestampError::InvalidTimezoneOffset.to_string(),
            "invalid timezone offset"
        );
    }

    #[test]
    fn invalid_time_unit_displays_message() {
        let err = PoSQLTimestampError::InvalidTimeUnit {
            error: "unknown unit".to_string(),
        };
        assert_eq!(err.to_string(), "Invalid time unit");
    }

    #[test]
    fn unsupported_precision_displays_error() {
        let err = PoSQLTimestampError::UnsupportedPrecision {
            error: "picoseconds".to_string(),
        };
        assert_eq!(err.to_string(), "Unsupported precision for timestamp: picoseconds");
    }

    #[test]
    fn from_impl_converts_to_string_via_display() {
        let err = PoSQLTimestampError::InvalidTimezoneOffset;
        let s: String = err.into();
        assert_eq!(s, "invalid timezone offset");
    }

    #[test]
    fn timestamp_errors_implement_partial_eq() {
        assert_eq!(PoSQLTimestampError::InvalidTimezoneOffset, PoSQLTimestampError::InvalidTimezoneOffset);
        let e1 = PoSQLTimestampError::InvalidTimezone { timezone: "x".to_string() };
        let e2 = PoSQLTimestampError::InvalidTimezone { timezone: "x".to_string() };
        assert_eq!(e1, e2);
    }

    #[test]
    fn timestamp_error_debug_contains_variant_name() {
        let debug = format!("{:?}", PoSQLTimestampError::InvalidTimezoneOffset);
        assert!(debug.contains("InvalidTimezoneOffset"));
    }
}
