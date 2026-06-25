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
    fn invalid_timezone_display_contains_timezone() {
        let e = PoSQLTimestampError::InvalidTimezone { timezone: "badzone".into() };
        assert!(alloc::format!("{e}").contains("badzone"));
    }

    #[test]
    fn invalid_timezone_offset_display() {
        let e = PoSQLTimestampError::InvalidTimezoneOffset;
        assert_eq!(alloc::format!("{e}"), "invalid timezone offset");
    }

    #[test]
    fn invalid_time_unit_display_contains_error() {
        let e = PoSQLTimestampError::InvalidTimeUnit { error: "bad unit".into() };
        assert!(alloc::format!("{e}").contains("time unit") || alloc::format!("{e}").contains("Invalid"));
    }

    #[test]
    fn unsupported_precision_display_contains_error() {
        let e = PoSQLTimestampError::UnsupportedPrecision { error: "nanoseconds".into() };
        assert!(alloc::format!("{e}").contains("nanoseconds") || alloc::format!("{e}").contains("precision"));
    }

    #[test]
    fn error_can_be_converted_to_string() {
        let e = PoSQLTimestampError::InvalidTimezoneOffset;
        let s: alloc::string::String = e.into();
        assert!(!s.is_empty());
    }

    #[test]
    fn debug_contains_variant_name() {
        let e = PoSQLTimestampError::InvalidTimezoneOffset;
        assert!(alloc::format!("{e:?}").contains("InvalidTimezoneOffset"));
    }

    #[test]
    fn equality_holds_for_same_variant() {
        assert_eq!(PoSQLTimestampError::InvalidTimezoneOffset, PoSQLTimestampError::InvalidTimezoneOffset);
    }
}
