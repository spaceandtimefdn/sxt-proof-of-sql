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
    fn timestamp_errors_convert_to_display_strings() {
        let timezone_error: String = PoSQLTimestampError::InvalidTimezone {
            timezone: "MARS".into(),
        }
        .into();
        assert_eq!(timezone_error, "invalid timezone string: MARS");

        let unit_error: String = PoSQLTimestampError::InvalidTimeUnit {
            error: "fortnight".into(),
        }
        .into();
        assert_eq!(unit_error, "Invalid time unit");

        let precision_error: String =
            PoSQLTimestampError::UnsupportedPrecision { error: "2".into() }.into();
        assert_eq!(precision_error, "Unsupported precision for timestamp: 2");
    }
}
