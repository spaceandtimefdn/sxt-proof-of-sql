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

    #[test]
    fn we_can_display_parse_errors() {
        assert_eq!(
            ParseError::InvalidTableReference {
                table_reference: "bad table".into(),
            }
            .to_string(),
            "Invalid table reference: bad table"
        );
    }
}
