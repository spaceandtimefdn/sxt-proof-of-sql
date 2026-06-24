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
    fn invalid_table_reference_displays_reference() {
        let err = ParseError::InvalidTableReference {
            table_reference: "bad.ref".to_string(),
        };
        assert_eq!(err.to_string(), "Invalid table reference: bad.ref");
    }

    #[test]
    fn invalid_table_reference_empty_string() {
        let err = ParseError::InvalidTableReference {
            table_reference: String::new(),
        };
        assert_eq!(err.to_string(), "Invalid table reference: ");
    }

    #[test]
    fn parse_error_equality() {
        let e1 = ParseError::InvalidTableReference { table_reference: "a.b".to_string() };
        let e2 = ParseError::InvalidTableReference { table_reference: "a.b".to_string() };
        assert_eq!(e1, e2);
    }

    #[test]
    fn parse_error_inequality_different_ref() {
        let e1 = ParseError::InvalidTableReference { table_reference: "a".to_string() };
        let e2 = ParseError::InvalidTableReference { table_reference: "b".to_string() };
        assert_ne!(e1, e2);
    }

    #[test]
    fn parse_error_debug_contains_variant_name() {
        let err = ParseError::InvalidTableReference { table_reference: "foo".to_string() };
        let debug = format!("{err:?}");
        assert!(debug.contains("InvalidTableReference"));
        assert!(debug.contains("foo"));
    }
}
