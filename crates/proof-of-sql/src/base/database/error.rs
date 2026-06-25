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
    fn parse_error_displays_invalid_table_reference() {
        assert_eq!(
            ParseError::InvalidTableReference {
                table_reference: "schema.table.extra".to_string(),
            }
            .to_string(),
            "Invalid table reference: schema.table.extra"
        );
    }
}
