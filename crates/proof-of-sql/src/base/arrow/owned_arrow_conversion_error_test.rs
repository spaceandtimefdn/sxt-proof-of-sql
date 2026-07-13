use super::OwnedArrowConversionError;

#[test]
fn owned_arrow_conversion_error_display_messages() {
    // UnsupportedType
    use arrow::datatypes::DataType;
    let err = OwnedArrowConversionError::UnsupportedType {
        datatype: DataType::Float32,
    };
    assert!(format!("{}").contains("unsupported type"));
    
    // DuplicateIdents
    let err = OwnedArrowConversionError::DuplicateIdents;
    assert_eq!(format!("{}"), "conversion resulted in duplicate idents");
    
    // NullNotSupportedYet
    let err = OwnedArrowConversionError::NullNotSupportedYet;
    assert_eq!(format!("{}"), "null values are not supported in OwnedColumn yet");
}

#[test]
fn owned_arrow_conversion_error_debug_formatting() {
    let err = OwnedArrowConversionError::DuplicateIdents;
    assert!(format!("{:?}").contains("DuplicateIdents"));
    
    let err2 = OwnedArrowConversionError::NullNotSupportedYet;
    assert!(format!("{:?}").contains("NullNotSupportedYet"));
}

#[test]
fn owned_arrow_conversion_error_equality() {
    use arrow::datatypes::DataType;
    let err1 = OwnedArrowConversionError::UnsupportedType {
        datatype: DataType::Int32,
    };
    let err2 = OwnedArrowConversionError::UnsupportedType {
        datatype: DataType::Int32,
    };
    let err3 = OwnedArrowConversionError::UnsupportedType {
        datatype: DataType::Int64,
    };
    assert_eq!(err1, err2);
    assert_ne!(err1, err3);
}
