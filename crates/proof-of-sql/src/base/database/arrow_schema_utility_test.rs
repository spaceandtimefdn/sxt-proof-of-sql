use super::arrow_schema_utility::get_posql_compatible_schema;
use alloc::sync::Arc;
use arrow::datatypes::{DataType, Field, Schema};

#[test]
fn get_posql_compatible_schema_converts_float_fields_to_decimal256() {
    let schema = Arc::new(Schema::new(vec![
        Field::new("float16", DataType::Float16, false),
        Field::new("float32", DataType::Float32, true),
        Field::new("float64", DataType::Float64, false),
    ]));

    let converted = get_posql_compatible_schema(&schema);

    assert_eq!(
        converted.field(0).data_type(),
        &DataType::Decimal256(20, 10)
    );
    assert_eq!(
        converted.field(1).data_type(),
        &DataType::Decimal256(20, 10)
    );
    assert_eq!(
        converted.field(2).data_type(),
        &DataType::Decimal256(20, 10)
    );
}

#[test]
fn get_posql_compatible_schema_preserves_non_float_field_types_and_nullability() {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, true),
        Field::new("amount", DataType::Decimal128(12, 2), false),
        Field::new("active", DataType::Boolean, true),
    ]));

    let converted = get_posql_compatible_schema(&schema);

    assert_eq!(converted.fields().len(), schema.fields().len());
    for (actual, expected) in converted.fields().iter().zip(schema.fields().iter()) {
        assert_eq!(actual.name(), expected.name());
        assert_eq!(actual.data_type(), expected.data_type());
        assert_eq!(actual.is_nullable(), expected.is_nullable());
    }
}

#[test]
fn get_posql_compatible_schema_preserves_float_field_names_and_nullability() {
    let schema = Arc::new(Schema::new(vec![
        Field::new("nullable_ratio", DataType::Float32, true),
        Field::new("required_score", DataType::Float64, false),
    ]));

    let converted = get_posql_compatible_schema(&schema);

    assert_eq!(converted.field(0).name(), "nullable_ratio");
    assert!(converted.field(0).is_nullable());
    assert_eq!(converted.field(1).name(), "required_score");
    assert!(!converted.field(1).is_nullable());
}
