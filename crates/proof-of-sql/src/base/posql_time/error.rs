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
    fn invalid_timezone_displays_timezone_string() {
        let err = PoSQLTimestampError::InvalidTimezone { timezone: "Bad/Zone".to_string() };
        assert_eq!(err.to_string(), "invalid timezone string: Bad/Zone");
    }

    #[test]
    fn invalid_timezone_offset_displays_correctly() {
        assert_eq!(
            PoSQLTimestampError::InvalidTimezoneOffset.to_string(),
            "invalid timezone offset"
        );
    }

    #[test]
    fn invalid_time_unit_displays_error_string() {
        let err = PoSQLTimestampError::InvalidTimeUnit { error: "bad unit".to_string() };
        assert_eq!(err.to_string(), "Invalid time unit");
    }

    #[test]
    fn unsupported_precision_includes_error_in_message() {
        let err = PoSQLTimestampError::UnsupportedPrecision { error: "femtoseconds".to_string() };
        let msg = err.to_string();
        assert!(msg.contains("femtoseconds"));
        assert!(msg.contains("Unsupported precision"));
    }

    #[test]
    fn from_impl_converts_invalid_timezone_offset_to_string() {
        let err = PoSQLTimestampError::InvalidTimezoneOffset;
        let s: String = err.into();
        assert_eq!(s, "invalid timezone offset");
    }

    #[test]
    fn from_impl_converts_invalid_time_unit_to_string() {
        let err = PoSQLTimestampError::InvalidTimeUnit { error: "x".to_string() };
        let s: String = err.into();
        assert_eq!(s, "Invalid time unit");
    }

    #[test]
    fn errors_implement_partial_eq_equal() {
        assert_eq!(
            PoSQLTimestampError::InvalidTimezoneOffset,
            PoSQLTimestampError::InvalidTimezoneOffset
        );
        let e1 = PoSQLTimestampError::InvalidTimezone { timezone: "X".to_string() };
        let e2 = PoSQLTimestampError::InvalidTimezone { timezone: "X".to_string() };
        assert_eq!(e1, e2);
    }

    #[test]
    fn errors_with_different_messages_are_not_equal() {
        let e1 = PoSQLTimestampError::InvalidTimezone { timezone: "A".to_string() };
        let e2 = PoSQLTimestampError::InvalidTimezone { timezone: "B".to_string() };
        assert_ne!(e1, e2);
    }

    #[test]
    fn different_variants_are_not_equal() {
        let e1 = PoSQLTimestampError::InvalidTimezoneOffset;
        let e2 = PoSQLTimestampError::InvalidTimeUnit { error: "x".to_string() };
        assert_ne!(e1, e2);
    }

    #[test]
    fn error_debug_contains_variant_name() {
        let debug = alloc::format!("{:?}", PoSQLTimestampError::InvalidTimezoneOffset);
        assert!(debug.contains("InvalidTimezoneOffset"));
    }

    #[test]
    fn invalid_timezone_error_contains_timezone_in_message() {
        let err = PoSQLTimestampError::InvalidTimezone { timezone: "Europe/Paris".to_string() };
        assert!(err.to_string().contains("Europe/Paris"));
    }

    #[test]
    fn unsupported_precision_with_empty_error_string() {
        let err = PoSQLTimestampError::UnsupportedPrecision { error: String::new() };
        assert!(err.to_string().contains("Unsupported precision"));
    }
}

