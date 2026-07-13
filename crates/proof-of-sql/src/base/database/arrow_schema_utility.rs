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
    use super::*;
    use arrow::datatypes::{DataType, Field, Schema};

    fn schema_from_fields(fields: Vec<Field>) -> SchemaRef {
        Arc::new(Schema::new(fields))
    }

    #[test]
    fn test_float_types_converted_to_decimal() {
        let schema = schema_from_fields(vec![
            Field::new("f16", DataType::Float16, false),
            Field::new("f32", DataType::Float32, false),
            Field::new("f64", DataType::Float64, false),
        ]);
        let result = get_posql_compatible_schema(&schema);
        for field in result.fields() {
            assert_eq!(
                field.data_type(),
                &DataType::Decimal256(20, 10),
                "field {} should be Decimal256",
                field.name()
            );
        }
    }

    #[test]
    fn test_non_float_types_unchanged() {
        let schema = schema_from_fields(vec![
            Field::new("i64_col", DataType::Int64, false),
            Field::new("bool_col", DataType::Boolean, false),
            Field::new("str_col", DataType::Utf8, true),
        ]);
        let result = get_posql_compatible_schema(&schema);
        assert_eq!(result.field(0).data_type(), &DataType::Int64);
        assert_eq!(result.field(1).data_type(), &DataType::Boolean);
        assert_eq!(result.field(2).data_type(), &DataType::Utf8);
        assert_eq!(result.field(2).is_nullable(), true);
    }

    #[test]
    fn test_mixed_schema() {
        let schema = schema_from_fields(vec![
            Field::new("a", DataType::Float32, false),
            Field::new("b", DataType::Int32, false),
        ]);
        let result = get_posql_compatible_schema(&schema);
        assert_eq!(result.field(0).data_type(), &DataType::Decimal256(20, 10));
        assert_eq!(result.field(1).data_type(), &DataType::Int32);
    }

    #[test]
    fn test_empty_schema() {
        let schema = schema_from_fields(vec![]);
        let result = get_posql_compatible_schema(&schema);
        assert_eq!(result.fields().len(), 0);
    }

    #[test]
    fn test_field_names_preserved() {
        let schema = schema_from_fields(vec![
            Field::new("my_float", DataType::Float64, false),
            Field::new("my_int", DataType::Int8, false),
        ]);
        let result = get_posql_compatible_schema(&schema);
        assert_eq!(result.field(0).name(), "my_float");
        assert_eq!(result.field(1).name(), "my_int");
    }
}
