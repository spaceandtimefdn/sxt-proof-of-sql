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
    fn timestamp_errors_render_human_readable_messages() {
        assert_eq!(
            PoSQLTimestampError::InvalidTimezone {
                timezone: "MARS".into(),
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
                error: "fortnight".into(),
            }
            .to_string(),
            "Invalid time unit"
        );
        assert_eq!(
            PoSQLTimestampError::UnsupportedPrecision { error: "12".into() }.to_string(),
            "Unsupported precision for timestamp: 12"
        );
    }

    #[test]
    fn timestamp_errors_convert_into_strings_without_losing_context() {
        let invalid_timezone: String = PoSQLTimestampError::InvalidTimezone {
            timezone: "UTC+25".into(),
        }
        .into();
        let unsupported_precision: String =
            PoSQLTimestampError::UnsupportedPrecision { error: "2".into() }.into();

        assert_eq!(invalid_timezone, "invalid timezone string: UTC+25");
        assert_eq!(
            unsupported_precision,
            "Unsupported precision for timestamp: 2"
        );
    }
}
