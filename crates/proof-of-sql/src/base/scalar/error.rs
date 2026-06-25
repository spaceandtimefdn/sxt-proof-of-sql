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
    fn overflow_error_display_contains_message() {
        let err = ScalarConversionError::Overflow {
            error: "value too large".to_string(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("Overflow error:"));
        assert!(msg.contains("value too large"));
    }

    #[test]
    fn overflow_error_is_debug_formattable() {
        let err = ScalarConversionError::Overflow {
            error: "test".to_string(),
        };
        let dbg = format!("{err:?}");
        assert!(dbg.contains("Overflow"));
    }

    #[test]
    fn overflow_error_with_empty_message() {
        let err = ScalarConversionError::Overflow {
            error: "".to_string(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("Overflow error:"));
    }
}
