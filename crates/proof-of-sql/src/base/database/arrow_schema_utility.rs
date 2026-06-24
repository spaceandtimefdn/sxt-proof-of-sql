//! This module provides utility functions for working with Arrow schemas in the context of Proof of SQL.
//! It includes functionality to convert Arrow schemas to PoSQL-compatible formats.

use alloc::sync::Arc;
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};

/// Converts an Arrow schema to a PoSQL-compatible schema.
///
/// This function takes an Arrow `SchemaRef` and returns a new `SchemaRef` where
/// floating-point data types (Float16, Float32, Float64) are converted to Decimal256(75, 30).
/// Other data types remain unchanged.
///
/// # Arguments
///
/// * `schema` - The input Arrow schema to convert.
///
/// # Returns
///
/// A new `SchemaRef` with PoSQL-compatible data types.
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

    fn make_schema(fields: Vec<Field>) -> Arc<Schema> {
        Arc::new(Schema::new(fields))
    }

    #[test]
    fn float32_is_converted_to_decimal256() {
        let schema = make_schema(vec![Field::new("f", DataType::Float32, false)]);
        let result = get_posql_compatible_schema(&schema);
        assert_eq!(result.field(0).data_type(), &DataType::Decimal256(20, 10));
    }

    #[test]
    fn float64_is_converted_to_decimal256() {
        let schema = make_schema(vec![Field::new("f", DataType::Float64, false)]);
        let result = get_posql_compatible_schema(&schema);
        assert_eq!(result.field(0).data_type(), &DataType::Decimal256(20, 10));
    }

    #[test]
    fn float16_is_converted_to_decimal256() {
        let schema = make_schema(vec![Field::new("f", DataType::Float16, false)]);
        let result = get_posql_compatible_schema(&schema);
        assert_eq!(result.field(0).data_type(), &DataType::Decimal256(20, 10));
    }

    #[test]
    fn int64_is_unchanged() {
        let schema = make_schema(vec![Field::new("i", DataType::Int64, false)]);
        let result = get_posql_compatible_schema(&schema);
        assert_eq!(result.field(0).data_type(), &DataType::Int64);
    }

    #[test]
    fn boolean_is_unchanged() {
        let schema = make_schema(vec![Field::new("b", DataType::Boolean, false)]);
        let result = get_posql_compatible_schema(&schema);
        assert_eq!(result.field(0).data_type(), &DataType::Boolean);
    }

    #[test]
    fn mixed_schema_converts_only_floats() {
        let schema = make_schema(vec![
            Field::new("a", DataType::Float32, false),
            Field::new("b", DataType::Int32, false),
            Field::new("c", DataType::Float64, true),
        ]);
        let result = get_posql_compatible_schema(&schema);
        assert_eq!(result.field(0).data_type(), &DataType::Decimal256(20, 10));
        assert_eq!(result.field(1).data_type(), &DataType::Int32);
        assert_eq!(result.field(2).data_type(), &DataType::Decimal256(20, 10));
    }

    #[test]
    fn empty_schema_returns_empty_schema() {
        let schema = make_schema(vec![]);
        let result = get_posql_compatible_schema(&schema);
        assert_eq!(result.fields().len(), 0);
    }

    #[test]
    fn field_nullability_is_preserved() {
        let schema = make_schema(vec![Field::new("f", DataType::Float32, true)]);
        let result = get_posql_compatible_schema(&schema);
        assert!(result.field(0).is_nullable());
    }

    #[test]
    fn field_name_is_preserved() {
        let schema = make_schema(vec![Field::new("my_float", DataType::Float64, false)]);
        let result = get_posql_compatible_schema(&schema);
        assert_eq!(result.field(0).name(), "my_float");
    }
}
