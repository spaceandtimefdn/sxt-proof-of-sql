use super::arrow_schema_utility::get_posql_compatible_schema;
use alloc::sync::Arc;
use arrow::datatypes::{DataType, Field, Schema};

#[test]
fn floating_point_fields_are_converted_to_decimals() {
    let schema = Arc::new(Schema::new(vec![
        Field::new("float16", DataType::Float16, false),
        Field::new("float32", DataType::Float32, true),
        Field::new("float64", DataType::Float64, false),
    ]));

    let compatible_schema = get_posql_compatible_schema(&schema);

    assert_eq!(compatible_schema.fields().len(), 3);
    for (index, name) in ["float16", "float32", "float64"].iter().enumerate() {
        let field = compatible_schema.field(index);
        assert_eq!(field.name(), *name);
        assert_eq!(field.data_type(), &DataType::Decimal256(20, 10));
    }
    assert!(!compatible_schema.field(0).is_nullable());
    assert!(compatible_schema.field(1).is_nullable());
    assert!(!compatible_schema.field(2).is_nullable());
}

#[test]
fn compatible_fields_are_left_unchanged() {
    let fields = vec![
        Field::new("boolean", DataType::Boolean, false),
        Field::new("integer", DataType::Int64, true),
        Field::new("text", DataType::Utf8, false),
        Field::new("decimal", DataType::Decimal256(12, 4), true),
    ];
    let schema = Arc::new(Schema::new(fields.clone()));

    let compatible_schema = get_posql_compatible_schema(&schema);

    assert_eq!(compatible_schema.fields().len(), fields.len());
    for (actual, expected) in compatible_schema.fields().iter().zip(fields.iter()) {
        assert_eq!(actual.name(), expected.name());
        assert_eq!(actual.data_type(), expected.data_type());
        assert_eq!(actual.is_nullable(), expected.is_nullable());
    }
}

#[test]
fn empty_schema_remains_empty() {
    let schema = Arc::new(Schema::empty());

    let compatible_schema = get_posql_compatible_schema(&schema);

    assert!(compatible_schema.fields().is_empty());
}
