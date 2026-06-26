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
    fn we_can_convert_float_fields_to_decimal256() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("f16", DataType::Float16, false),
            Field::new("f32", DataType::Float32, true),
            Field::new("f64", DataType::Float64, false),
        ]));

        let converted = get_posql_compatible_schema(&schema);
        let fields = converted.fields();

        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].name(), "f16");
        assert_eq!(fields[1].name(), "f32");
        assert_eq!(fields[2].name(), "f64");
        assert_eq!(fields[0].data_type(), &DataType::Decimal256(20, 10));
        assert_eq!(fields[1].data_type(), &DataType::Decimal256(20, 10));
        assert_eq!(fields[2].data_type(), &DataType::Decimal256(20, 10));
        assert!(!fields[0].is_nullable());
        assert!(fields[1].is_nullable());
        assert!(!fields[2].is_nullable());
    }

    #[test]
    fn we_preserve_non_float_fields_and_nullability() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("name", DataType::Utf8, true),
            Field::new("amount", DataType::Decimal256(75, 30), false),
        ]));

        let converted = get_posql_compatible_schema(&schema);
        let fields = converted.fields();

        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].name(), "id");
        assert_eq!(fields[1].name(), "name");
        assert_eq!(fields[2].name(), "amount");
        assert_eq!(fields[0].data_type(), &DataType::Int64);
        assert_eq!(fields[1].data_type(), &DataType::Utf8);
        assert_eq!(fields[2].data_type(), &DataType::Decimal256(75, 30));
        assert!(!fields[0].is_nullable());
        assert!(fields[1].is_nullable());
        assert!(!fields[2].is_nullable());
    }
}
