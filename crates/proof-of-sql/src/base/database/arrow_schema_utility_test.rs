use super::arrow_schema_utility::get_posql_compatible_schema;
use alloc::sync::Arc;
use arrow::datatypes::{DataType, Field, Schema};

#[test]
fn float_fields_are_converted_to_posql_decimals() {
    let schema = Arc::new(Schema::new(vec![
        Field::new("half", DataType::Float16, false),
        Field::new("single", DataType::Float32, true),
        Field::new("double", DataType::Float64, false),
    ]));

    let converted_schema = get_posql_compatible_schema(&schema);
    let converted_fields = converted_schema.fields();

    assert_eq!(converted_fields.len(), 3);
    assert_eq!(converted_fields[0].name(), "half");
    assert_eq!(
        converted_fields[0].data_type(),
        &DataType::Decimal256(20, 10)
    );
    assert!(!converted_fields[0].is_nullable());
    assert_eq!(converted_fields[1].name(), "single");
    assert_eq!(
        converted_fields[1].data_type(),
        &DataType::Decimal256(20, 10)
    );
    assert!(converted_fields[1].is_nullable());
    assert_eq!(converted_fields[2].name(), "double");
    assert_eq!(
        converted_fields[2].data_type(),
        &DataType::Decimal256(20, 10)
    );
    assert!(!converted_fields[2].is_nullable());
}

#[test]
fn non_float_fields_are_preserved() {
    let schema = Arc::new(Schema::new(vec![
        Field::new("flag", DataType::Boolean, true),
        Field::new("name", DataType::Utf8, false),
        Field::new("amount", DataType::Decimal128(12, 2), false),
        Field::new("count", DataType::Int64, false),
    ]));

    let converted_schema = get_posql_compatible_schema(&schema);
    let converted_fields = converted_schema.fields();

    assert_eq!(converted_fields.len(), schema.fields().len());
    for (converted_field, original_field) in converted_fields.iter().zip(schema.fields().iter()) {
        assert_eq!(converted_field.name(), original_field.name());
        assert_eq!(converted_field.data_type(), original_field.data_type());
        assert_eq!(converted_field.is_nullable(), original_field.is_nullable());
    }
}
