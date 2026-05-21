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
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn we_can_display_parse_errors() {
        let error = ParseError::InvalidTableReference {
            table_reference: "too.many.parts".to_string(),
        };

        assert_eq!(error.to_string(), "Invalid table reference: too.many.parts");
    }
}
