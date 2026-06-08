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
    fn we_display_scalar_conversion_overflow_errors() {
        let error = ScalarConversionError::Overflow {
            error: "BigInt too large for Scalar".into(),
        };

        assert_eq!(
            error.to_string(),
            "Overflow error: BigInt too large for Scalar"
        );
    }
}
