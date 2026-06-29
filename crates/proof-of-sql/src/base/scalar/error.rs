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
    use super::*;

    #[test]
    fn scalar_conversion_error_displays_overflow_message() {
        let error = ScalarConversionError::Overflow {
            error: "value exceeds i64".into(),
        };

        assert!(matches!(
            error,
            ScalarConversionError::Overflow { ref error } if error == "value exceeds i64"
        ));
        assert_eq!(error.to_string(), "Overflow error: value exceeds i64");
    }
}
