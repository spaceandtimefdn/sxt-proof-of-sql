use alloc::string::String;
use snafu::Snafu;

/// Errors encountered during the parsing process
#[derive(Debug, Snafu, Eq, PartialEq)]
pub enum ParseError {
    #[snafu(display("Invalid table reference: {}", table_reference))]
    /// Cannot parse the `TableRef`
    InvalidTableReference {
        /// The underlying error
        table_reference: String,
    },
}

#[cfg(test)]
mod tests {
    use super::ParseError;

    #[test]
    fn invalid_table_reference_display() {
        let e = ParseError::InvalidTableReference {
            table_reference: "a.b.c.d".into(),
        };
        let s = alloc::format!("{e}");
        assert!(s.contains("a.b.c.d"));
    }

    #[test]
    fn invalid_table_reference_equality() {
        let a = ParseError::InvalidTableReference {
            table_reference: "bad".into(),
        };
        let b = ParseError::InvalidTableReference {
            table_reference: "bad".into(),
        };
        assert_eq!(a, b);
    }

    #[test]
    fn invalid_table_reference_inequality() {
        let a = ParseError::InvalidTableReference {
            table_reference: "x".into(),
        };
        let b = ParseError::InvalidTableReference {
            table_reference: "y".into(),
        };
        assert_ne!(a, b);
    }

    #[test]
    fn parse_error_is_debug_formattable() {
        let e = ParseError::InvalidTableReference {
            table_reference: "z".into(),
        };
        let s = alloc::format!("{e:?}");
        assert!(s.contains("InvalidTableReference"));
    }
}
