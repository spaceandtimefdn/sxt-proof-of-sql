use super::{AnalyzeError, AnalyzeResult};
use crate::base::database::ColumnType;
use crate::base::proof::PlaceholderError;
use crate::base::math::decimal::DecimalError;

#[test]
fn analyze_error_display_messages() {
    // Test InvalidDataType
    let err = AnalyzeError::InvalidDataType {
        expr_type: ColumnType::Boolean,
    };
    assert_eq!(format!("{}"), "Expression has datatype Boolean, which was not valid");
    
    // Test DataTypeMismatch
    let err = AnalyzeError::DataTypeMismatch {
        left_type: "Int64".to_string(),
        right_type: "Boolean".to_string(),
    };
    assert_eq!(format!("{}"), "Left side has 'Int64' type but right side has 'Boolean' type");
    
    // Test DifferentColumnLength
    let err = AnalyzeError::DifferentColumnLength {
        len_a: 10,
        len_b: 20,
    };
    assert_eq!(format!("{}"), "Columns have different lengths: 10 != 20");
    
    // Test NotEnoughInputPlans
    let err = AnalyzeError::NotEnoughInputPlans;
    assert_eq!(format!("{}"), "Not enough input plans");
}

#[test]
fn analyze_error_debug_formatting() {
    let err = AnalyzeError::InvalidDataType {
        expr_type: ColumnType::SmallInt,
    };
    assert!(format!("{:?}").contains("InvalidDataType"));
    
    let err2 = AnalyzeError::NotEnoughInputPlans;
    assert!(format!("{:?}").contains("NotEnoughInputPlans"));
}

#[test]
fn analyze_error_equality() {
    let err1 = AnalyzeError::InvalidDataType {
        expr_type: ColumnType::Int,
    };
    let err2 = AnalyzeError::InvalidDataType {
        expr_type: ColumnType::Int,
    };
    let err3 = AnalyzeError::InvalidDataType {
        expr_type: ColumnType::BigInt,
    };
    assert_eq!(err1, err2);
    assert_ne!(err1, err3);
    
    // Different variants are not equal
    let err4 = AnalyzeError::NotEnoughInputPlans;
    assert_ne!(err1, err4);
}

#[test]
fn analyze_error_to_string_conversion() {
    let err = AnalyzeError::DataTypeMismatch {
        left_type: "A".to_string(),
        right_type: "B".to_string(),
    };
    let s: String = err.into();
    assert_eq!(s, "Left side has 'A' type but right side has 'B' type");
}

#[test]
fn analyze_result_error_propagation() -> AnalyzeResult<i32> {
    Err(AnalyzeError::InvalidDataType {
        expr_type: ColumnType::Varchar,
    })?;
    Ok(42)
}

#[test]
fn analyze_result_success() -> AnalyzeResult<i32> {
    Ok(42)
}
