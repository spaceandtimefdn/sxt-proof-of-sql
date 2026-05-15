//! This module provides utility functions for working with Arrow schemas in the context of Proof of SQL.
//! It includes functionality to convert Arrow schemas to PoSQL-compatible formats.

use alloc::sync::Arc;
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};

/// Converts an Arrow schema to a PoSQL-compatible schema.
///
/// This function takes an Arrow `SchemaRef` and returns a new `SchemaRef` where
/// floating-point data types (Float16, Float32, Float64) are converted to Decimal256(20, 10).
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

    fn field_type(schema: &SchemaRef, index: usize) -> &DataType {
        schema.field(index).data_type()
    }

    #[test]
    fn float_fields_are_converted_to_decimal256() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("half", DataType::Float16, false),
            Field::new("single", DataType::Float32, true),
            Field::new("double", DataType::Float64, false),
        ]));

        let converted = get_posql_compatible_schema(&schema);

        assert_eq!(field_type(&converted, 0), &DataType::Decimal256(20, 10));
        assert_eq!(field_type(&converted, 1), &DataType::Decimal256(20, 10));
        assert_eq!(field_type(&converted, 2), &DataType::Decimal256(20, 10));
        assert_eq!(converted.field(0).name(), "half");
        assert!(converted.field(1).is_nullable());
        assert!(!converted.field(2).is_nullable());
    }

    #[test]
    fn non_float_fields_are_preserved() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("amount", DataType::Decimal256(75, 30), true),
            Field::new("label", DataType::Utf8, true),
        ]));

        let converted = get_posql_compatible_schema(&schema);

        assert_eq!(field_type(&converted, 0), &DataType::Int64);
        assert_eq!(field_type(&converted, 1), &DataType::Decimal256(75, 30));
        assert_eq!(field_type(&converted, 2), &DataType::Utf8);
        assert_eq!(converted.field(0).name(), "id");
        assert!(!converted.field(0).is_nullable());
        assert!(converted.field(1).is_nullable());
    }

    #[test]
    fn empty_schema_stays_empty() {
        let schema = Arc::new(Schema::empty());

        let converted = get_posql_compatible_schema(&schema);

        assert!(converted.fields().is_empty());
    }
}
