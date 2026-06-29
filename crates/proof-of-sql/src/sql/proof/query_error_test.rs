use super::QueryError;
use crate::base::proof::ProofError;

#[test]
fn query_error_display_messages() {
    // Overflow
    let err = QueryError::Overflow;
    assert_eq!(format!("{}"), "Overflow error");
    
    // InvalidString
    let err = QueryError::InvalidString;
    assert_eq!(format!("{}"), "String decode error");
    
    // MiscellaneousDecodingError
    let err = QueryError::MiscellaneousDecodingError;
    assert_eq!(format!("{}"), "Miscellaneous decoding error");
    
    // MiscellaneousEvaluationError
    let err = QueryError::MiscellaneousEvaluationError;
    assert_eq!(format!("{}"), "Miscellaneous evaluation error");
    
    // InvalidColumnCount
    let err = QueryError::InvalidColumnCount;
    assert_eq!(format!("{}"), "Invalid number of columns");
}

#[test]
fn query_error_debug_formatting() {
    let err = QueryError::Overflow;
    assert!(format!("{:?}").contains("Overflow"));
    
    let err2 = QueryError::InvalidColumnCount;
    assert!(format!("{:?}").contains("InvalidColumnCount"));
}

#[test]
fn query_error_equality() {
    let err1 = QueryError::Overflow;
    let err2 = QueryError::Overflow;
    let err3 = QueryError::InvalidString;
    assert_eq!(err1, err2);
    assert_ne!(err1, err3);
}
