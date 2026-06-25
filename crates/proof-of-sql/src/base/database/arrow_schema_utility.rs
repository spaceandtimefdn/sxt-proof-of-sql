//! This module provides utility functions for working with Arrow schemas in the context of Proof of SQL.
//! It includes functionality to convert Arrow schemas to PoSQL-compatible formats.

use alloc::sync::Arc;
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};

/// Converts an Arrow schema to a PoSQL-compatible schema.
#[must_use]
pub fn get_posql_compatible_schema(schema: &SchemaRef) -> SchemaRef {
    let new_fields: Vec<Field> = schema
        .fields()
        .iter()
        .map(|field| {
            let new_data_type = match field.data_type() {
                DataType::Float16 | DataType::Float32 | DataType::Float64 => {
                    DataType::Decimal256(20, 10)
                }
                _ => field.data_type().clone(),
            };
            Field::new(field.name(), new_data_type, field.is_nullable())
        })
        .collect();

    Arc::new(Schema::new(new_fields))
}

#[cfg(test)]
mod tests {
    use super::get_posql_compatible_schema;
    use alloc::sync::Arc;
    use arrow::datatypes::{DataType, Field, Schema};

    #[test]
    fn float32_is_converted_to_decimal256() {
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::Float32, false)]));
        let converted = get_posql_compatible_schema(&schema);
        assert_eq!(converted.field(0).data_type(), &DataType::Decimal256(20, 10));
    }

    #[test]
    fn float64_is_converted_to_decimal256() {
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::Float64, false)]));
        let converted = get_posql_compatible_schema(&schema);
        assert_eq!(converted.field(0).data_type(), &DataType::Decimal256(20, 10));
    }

    #[test]
    fn float16_is_converted_to_decimal256() {
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::Float16, false)]));
        let converted = get_posql_compatible_schema(&schema);
        assert_eq!(converted.field(0).data_type(), &DataType::Decimal256(20, 10));
    }

    #[test]
    fn int64_type_is_unchanged() {
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::Int64, false)]));
        let converted = get_posql_compatible_schema(&schema);
        assert_eq!(converted.field(0).data_type(), &DataType::Int64);
    }

    #[test]
    fn boolean_type_is_unchanged() {
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::Boolean, false)]));
        let converted = get_posql_compatible_schema(&schema);
        assert_eq!(converted.field(0).data_type(), &DataType::Boolean);
    }

    #[test]
    fn field_name_is_preserved_after_conversion() {
        let schema = Arc::new(Schema::new(vec![Field::new("myfield", DataType::Float32, false)]));
        let converted = get_posql_compatible_schema(&schema);
        assert_eq!(converted.field(0).name(), "myfield");
    }

    #[test]
    fn empty_schema_converts_to_empty_schema() {
        let schema = Arc::new(Schema::new(Vec::<Field>::new()));
        let converted = get_posql_compatible_schema(&schema);
        assert_eq!(converted.fields().len(), 0);
    }

    #[test]
    fn mixed_schema_converts_only_float_fields() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("a", DataType::Float32, false),
            Field::new("b", DataType::Int32, false),
        ]));
        let converted = get_posql_compatible_schema(&schema);
        assert_eq!(converted.field(0).data_type(), &DataType::Decimal256(20, 10));
        assert_eq!(converted.field(1).data_type(), &DataType::Int32);
    }
}
