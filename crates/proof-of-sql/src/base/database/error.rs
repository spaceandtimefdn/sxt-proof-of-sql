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
    fn invalid_table_reference_display_contains_reference() {
        let e = ParseError::InvalidTableReference { table_reference: "a.b.c".into() };
        assert!(alloc::format!("{e}").contains("a.b.c"));
    }

    #[test]
    fn equality_holds_for_same_values() {
        let e1 = ParseError::InvalidTableReference { table_reference: "x".into() };
        let e2 = ParseError::InvalidTableReference { table_reference: "x".into() };
        assert_eq!(e1, e2);
    }

    #[test]
    fn debug_contains_variant_name() {
        let e = ParseError::InvalidTableReference { table_reference: "t".into() };
        assert!(alloc::format!("{e:?}").contains("InvalidTableReference"));
    }
}
