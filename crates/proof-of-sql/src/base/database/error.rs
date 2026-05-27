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
    fn invalid_table_reference_display_includes_the_rejected_reference() {
        let error = ParseError::InvalidTableReference {
            table_reference: "catalog.schema.table".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "Invalid table reference: catalog.schema.table"
        );
    }

    #[test]
    fn invalid_table_reference_equality_tracks_the_original_reference() {
        let first = ParseError::InvalidTableReference {
            table_reference: "schema.table".to_string(),
        };
        let second = ParseError::InvalidTableReference {
            table_reference: "schema.table".to_string(),
        };
        let different = ParseError::InvalidTableReference {
            table_reference: "catalog.schema.table".to_string(),
        };

        assert_eq!(first, second);
        assert_ne!(first, different);
    }
}
