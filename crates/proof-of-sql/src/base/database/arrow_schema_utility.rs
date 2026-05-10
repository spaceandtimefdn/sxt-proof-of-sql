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

    #[test]
    fn we_convert_arrow_float_fields_to_posql_decimal_fields() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("float16_col", DataType::Float16, false),
            Field::new("float32_col", DataType::Float32, true),
            Field::new("float64_col", DataType::Float64, false),
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
    fn we_keep_non_float_arrow_fields_unchanged() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("payload", DataType::Utf8, true),
            Field::new("amount", DataType::Decimal128(12, 2), false),
        ]));

        let converted = get_posql_compatible_schema(&schema);

        assert_eq!(converted.field(0).data_type(), &DataType::Int64);
        assert_eq!(converted.field(1).data_type(), &DataType::Utf8);
        assert_eq!(converted.field(2).data_type(), &DataType::Decimal128(12, 2));
        assert_eq!(converted.field(0).name(), "id");
        assert_eq!(converted.field(1).name(), "payload");
        assert_eq!(converted.field(2).name(), "amount");
    }
}
