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
    use alloc::string::ToString;

    #[test]
    fn we_can_display_scalar_conversion_errors() {
        let error = ScalarConversionError::Overflow {
            error: "too many bits".to_string(),
        };

        assert_eq!(error.to_string(), "Overflow error: too many bits");
    }
}
