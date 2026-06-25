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
    fn invalid_timezone_display() {
        let e = PoSQLTimestampError::InvalidTimezone {
            timezone: "Nowhere/Land".into(),
        };
        let s = alloc::format!("{e}");
        assert!(s.contains("Nowhere/Land"));
    }

    #[test]
    fn invalid_timezone_offset_display() {
        let e = PoSQLTimestampError::InvalidTimezoneOffset;
        let s = alloc::format!("{e}");
        assert!(s.contains("offset") || s.contains("timezone"));
    }

    #[test]
    fn invalid_time_unit_display() {
        let e = PoSQLTimestampError::InvalidTimeUnit {
            error: "bad unit".into(),
        };
        let s = alloc::format!("{e}");
        assert!(s.contains("time unit") || s.contains("Invalid"));
    }

    #[test]
    fn unsupported_precision_display() {
        let e = PoSQLTimestampError::UnsupportedPrecision {
            error: "femtoseconds".into(),
        };
        let s = alloc::format!("{e}");
        assert!(s.contains("femtoseconds"));
    }

    #[test]
    fn invalid_timezone_equality() {
        let a = PoSQLTimestampError::InvalidTimezone { timezone: "X".into() };
        let b = PoSQLTimestampError::InvalidTimezone { timezone: "X".into() };
        assert_eq!(a, b);
    }

    #[test]
    fn invalid_timezone_offset_equality() {
        assert_eq!(
            PoSQLTimestampError::InvalidTimezoneOffset,
            PoSQLTimestampError::InvalidTimezoneOffset
        );
    }

    #[test]
    fn posql_timestamp_error_is_debug_formattable() {
        let e = PoSQLTimestampError::InvalidTimezoneOffset;
        let _ = alloc::format!("{e:?}");
    }

    #[test]
    fn from_posql_timestamp_error_to_string() {
        let e = PoSQLTimestampError::InvalidTimezoneOffset;
        let s: alloc::string::String = e.into();
        assert!(!s.is_empty());
    }
}
