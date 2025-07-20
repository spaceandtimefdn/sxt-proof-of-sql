use alloc::string::String;
use snafu::Snafu;

/// Errors encountered during the parsing process
#[derive(Debug, Snafu, Eq, PartialEq, Clone)]
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
    fn test_parse_error_invalid_table_reference() {
        let table_ref = "invalid.table.reference.with.too.many.parts".to_string();
        let error = ParseError::InvalidTableReference {
            table_reference: table_ref.clone(),
        };
        
        assert_eq!(
            error.to_string(),
            format!("Invalid table reference: {}", table_ref)
        );
    }

    #[test]
    fn test_parse_error_equality() {
        let error1 = ParseError::InvalidTableReference {
            table_reference: "test.table".to_string(),
        };
        let error2 = ParseError::InvalidTableReference {
            table_reference: "test.table".to_string(),
        };
        let error3 = ParseError::InvalidTableReference {
            table_reference: "different.table".to_string(),
        };
        
        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    #[test]
    fn test_parse_error_debug() {
        let error = ParseError::InvalidTableReference {
            table_reference: "test.table".to_string(),
        };
        
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("InvalidTableReference"));
        assert!(debug_str.contains("test.table"));
    }

    #[test]
    fn test_parse_error_display() {
        let error = ParseError::InvalidTableReference {
            table_reference: "my.invalid.reference".to_string(),
        };
        
        let display_str = error.to_string();
        assert_eq!(display_str, "Invalid table reference: my.invalid.reference");
    }

    #[test]
    fn test_parse_error_with_empty_string() {
        let error = ParseError::InvalidTableReference {
            table_reference: String::new(),
        };
        
        assert_eq!(error.to_string(), "Invalid table reference: ");
    }

    #[test]
    fn test_parse_error_with_special_characters() {
        let table_ref = "table with spaces and symbols !@#$%".to_string();
        let error = ParseError::InvalidTableReference {
            table_reference: table_ref.clone(),
        };
        
        assert_eq!(
            error.to_string(),
            format!("Invalid table reference: {}", table_ref)
        );
    }

    #[test]
    fn test_parse_error_clone() {
        let original = ParseError::InvalidTableReference {
            table_reference: "test.table".to_string(),
        };
        let cloned = original.clone();
        
        assert_eq!(original, cloned);
    }

    // Test that the error implements std::error::Error trait (if std feature is enabled)
    #[test]
    #[cfg(feature = "std")]
    fn test_parse_error_is_std_error() {
        use std::error::Error;
        
        let error = ParseError::InvalidTableReference {
            table_reference: "test.table".to_string(),
        };
        
        // Should compile if ParseError implements Error
        let _: &dyn Error = &error;
    }
}
