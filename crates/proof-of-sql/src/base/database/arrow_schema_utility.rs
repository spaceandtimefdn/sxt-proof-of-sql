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

    #[test]
    fn we_convert_float_fields_to_posql_decimal_fields() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("float16", DataType::Float16, false),
            Field::new("float32", DataType::Float32, true),
            Field::new("float64", DataType::Float64, false),
        ]));

        let converted = get_posql_compatible_schema(&schema);

        assert_eq!(
            converted
                .fields()
                .iter()
                .map(|field| field.data_type())
                .collect::<Vec<_>>(),
            vec![
                &DataType::Decimal256(20, 10),
                &DataType::Decimal256(20, 10),
                &DataType::Decimal256(20, 10),
            ]
        );
        assert!(!converted.field(0).is_nullable());
        assert!(converted.field(1).is_nullable());
        assert!(!converted.field(2).is_nullable());
    }

    #[test]
    fn we_preserve_non_float_field_types() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("boolean", DataType::Boolean, false),
            Field::new("int64", DataType::Int64, false),
            Field::new("utf8", DataType::Utf8, true),
            Field::new("decimal", DataType::Decimal256(75, 30), true),
        ]));

        let converted = get_posql_compatible_schema(&schema);

        assert_eq!(
            converted
                .fields()
                .iter()
                .map(|field| field.data_type())
                .collect::<Vec<_>>(),
            vec![
                &DataType::Boolean,
                &DataType::Int64,
                &DataType::Utf8,
                &DataType::Decimal256(75, 30),
            ]
        );
        assert_eq!(
            converted
                .fields()
                .iter()
                .map(|field| field.name().as_str())
                .collect::<Vec<_>>(),
            vec!["boolean", "int64", "utf8", "decimal"]
        );
    }
}
