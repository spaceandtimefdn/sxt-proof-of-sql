use alloc::string::String;
use snafu::Snafu;

#[derive(Snafu, Debug)]
/// These errors occur when a scalar conversion fails.
pub enum ScalarConversionError {
    #[snafu(display("Overflow error: {error}"))]
    /// This error occurs when a scalar is too large to be converted.
    Overflow {
        /// The underlying error
        error: String,
    },
}

#[cfg(test)]
mod tests {
    use super::ScalarConversionError;

    #[test]
    fn overflow_displays_error_message() {
        let err = ScalarConversionError::Overflow {
            error: "value too large".to_string(),
        };
        assert_eq!(err.to_string(), "Overflow error: value too large");
    }

    #[test]
    fn overflow_with_empty_error_message() {
        let err = ScalarConversionError::Overflow {
            error: String::new(),
        };
        assert_eq!(err.to_string(), "Overflow error: ");
    }

    #[test]
    fn overflow_debug_contains_variant_name() {
        let err = ScalarConversionError::Overflow {
            error: "x".to_string(),
        };
        let debug = format!("{err:?}");
        assert!(debug.contains("Overflow"));
    }

    #[test]
    fn overflow_debug_contains_error_content() {
        let err = ScalarConversionError::Overflow {
            error: "sentinel_value".to_string(),
        };
        let debug = format!("{err:?}");
        assert!(debug.contains("sentinel_value"));
    }
}
