use super::ParseError;

#[test]
fn parse_error_display_messages() {
    let err = ParseError::InvalidTableReference {
        table_reference: "users".to_string(),
    };
    assert_eq!(format!("{}"), "Invalid table reference: users");
    
    let err2 = ParseError::InvalidTableReference {
        table_reference: "orders".to_string(),
    };
    assert_eq!(format!("{}"), "Invalid table reference: orders");
}

#[test]
fn parse_error_debug_formatting() {
    let err = ParseError::InvalidTableReference {
        table_reference: "test".to_string(),
    };
    assert_eq!(format!("{:?}"), "InvalidTableReference { table_reference: "test" }");
}

#[test]
fn parse_error_equality() {
    let err1 = ParseError::InvalidTableReference {
        table_reference: "foo".to_string(),
    };
    let err2 = ParseError::InvalidTableReference {
        table_reference: "foo".to_string(),
    };
    let err3 = ParseError::InvalidTableReference {
        table_reference: "bar".to_string(),
    };
    assert_eq!(err1, err2);
    assert_ne!(err1, err3);
}
