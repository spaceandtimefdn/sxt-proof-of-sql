use super::arrow_schema_utility::get_posql_compatible_schema;
use alloc::sync::Arc;
use arrow::datatypes::{DataType, Field, Schema};

#[test]
fn empty_schema_returns_empty_schema() {
    let schema = Arc::new(Schema::empty());
    let result = get_posql_compatible_schema(&schema);
    assert_eq!(result.fields().len(), 0);
}

#[test]
fn float16_is_converted_to_decimal256() {
    let schema = Arc::new(Schema::new(vec![Field::new("f16", DataType::Float16, true)]));
    let result = get_posql_compatible_schema(&schema);
    assert_eq!(result.field(0).data_type(), &DataType::Decimal256(20, 10));
    assert!(result.field(0).is_nullable());
}

#[test]
fn float32_is_converted_to_decimal256() {
    let schema = Arc::new(Schema::new(vec![Field::new(
        "f32",
        DataType::Float32,
        false,
    )]));
    let result = get_posql_compatible_schema(&schema);
    assert_eq!(result.field(0).data_type(), &DataType::Decimal256(20, 10));
    assert!(!result.field(0).is_nullable());
}

#[test]
fn float64_is_converted_to_decimal256() {
    let schema = Arc::new(Schema::new(vec![Field::new("f64", DataType::Float64, true)]));
    let result = get_posql_compatible_schema(&schema);
    assert_eq!(result.field(0).data_type(), &DataType::Decimal256(20, 10));
}

#[test]
fn non_float_types_are_preserved() {
    let schema = Arc::new(Schema::new(vec![
        Field::new("int32", DataType::Int32, true),
        Field::new("utf8", DataType::Utf8, false),
        Field::new("bool", DataType::Boolean, true),
        Field::new("int64", DataType::Int64, false),
    ]));
    let result = get_posql_compatible_schema(&schema);
    assert_eq!(result.field(0).data_type(), &DataType::Int32);
    assert_eq!(result.field(1).data_type(), &DataType::Utf8);
    assert_eq!(result.field(2).data_type(), &DataType::Boolean);
    assert_eq!(result.field(3).data_type(), &DataType::Int64);
}

#[test]
fn mixed_float_and_non_float_types() {
    let schema = Arc::new(Schema::new(vec![
        Field::new("a", DataType::Float32, true),
        Field::new("b", DataType::Int32, false),
        Field::new("c", DataType::Float64, true),
        Field::new("d", DataType::Utf8, false),
    ]));
    let result = get_posql_compatible_schema(&schema);
    assert_eq!(result.field(0).data_type(), &DataType::Decimal256(20, 10));
    assert_eq!(result.field(1).data_type(), &DataType::Int32);
    assert_eq!(result.field(2).data_type(), &DataType::Decimal256(20, 10));
    assert_eq!(result.field(3).data_type(), &DataType::Utf8);
}

#[test]
fn field_names_are_preserved() {
    let schema = Arc::new(Schema::new(vec![
        Field::new("my_col", DataType::Float32, true),
        Field::new("other_col", DataType::Int64, false),
    ]));
    let result = get_posql_compatible_schema(&schema);
    assert_eq!(result.field(0).name(), "my_col");
    assert_eq!(result.field(1).name(), "other_col");
}

#[test]
fn nullability_is_preserved() {
    let schema = Arc::new(Schema::new(vec![
        Field::new("nullable_float", DataType::Float64, true),
        Field::new("non_nullable_float", DataType::Float32, false),
    ]));
    let result = get_posql_compatible_schema(&schema);
    assert!(result.field(0).is_nullable());
    assert!(!result.field(1).is_nullable());
}
