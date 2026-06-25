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
    fn overflow_display_contains_message() {
        let e = ScalarConversionError::Overflow {
            error: "value too large".into(),
        };
        let s = alloc::format!("{e}");
        assert!(s.contains("value too large"));
    }

    #[test]
    fn overflow_display_contains_overflow() {
        let e = ScalarConversionError::Overflow {
            error: "abc".into(),
        };
        let s = alloc::format!("{e}");
        assert!(s.contains("Overflow") || s.contains("overflow"));
    }

    #[test]
    fn overflow_debug_contains_overflow() {
        let e = ScalarConversionError::Overflow {
            error: "abc".into(),
        };
        let s = alloc::format!("{e:?}");
        assert!(s.contains("Overflow"));
    }

    #[test]
    fn overflow_error_message_is_preserved() {
        let e = ScalarConversionError::Overflow {
            error: "specific error text".into(),
        };
        let s = alloc::format!("{e}");
        assert!(s.contains("specific error text"));
    }
}
