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

    #[test]
    fn we_can_convert_float16_to_decimal256() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "float16_col",
            DataType::Float16,
            false,
        )]));

        let result = get_posql_compatible_schema(&schema);

        assert_eq!(result.fields().len(), 1);
        assert_eq!(result.fields()[0].name(), "float16_col");
        assert_eq!(
            result.fields()[0].data_type(),
            &DataType::Decimal256(20, 10)
        );
        assert!(!result.fields()[0].is_nullable());
    }

    #[test]
    fn we_can_convert_float32_to_decimal256() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "float32_col",
            DataType::Float32,
            true,
        )]));

        let result = get_posql_compatible_schema(&schema);

        assert_eq!(result.fields().len(), 1);
        assert_eq!(result.fields()[0].name(), "float32_col");
        assert_eq!(
            result.fields()[0].data_type(),
            &DataType::Decimal256(20, 10)
        );
        assert!(result.fields()[0].is_nullable());
    }

    #[test]
    fn we_can_convert_float64_to_decimal256() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "float64_col",
            DataType::Float64,
            false,
        )]));

        let result = get_posql_compatible_schema(&schema);

        assert_eq!(result.fields().len(), 1);
        assert_eq!(result.fields()[0].name(), "float64_col");
        assert_eq!(
            result.fields()[0].data_type(),
            &DataType::Decimal256(20, 10)
        );
        assert!(!result.fields()[0].is_nullable());
    }

    #[test]
    fn we_can_preserve_non_float_types() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("int32_col", DataType::Int32, false),
            Field::new("int64_col", DataType::Int64, true),
            Field::new("utf8_col", DataType::Utf8, false),
            Field::new("bool_col", DataType::Boolean, true),
        ]));

        let result = get_posql_compatible_schema(&schema);

        assert_eq!(result.fields().len(), 4);
        assert_eq!(result.fields()[0].data_type(), &DataType::Int32);
        assert_eq!(result.fields()[1].data_type(), &DataType::Int64);
        assert_eq!(result.fields()[2].data_type(), &DataType::Utf8);
        assert_eq!(result.fields()[3].data_type(), &DataType::Boolean);
    }

    #[test]
    fn we_can_handle_mixed_float_and_non_float_types() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("int_col", DataType::Int32, false),
            Field::new("float32_col", DataType::Float32, false),
            Field::new("string_col", DataType::Utf8, true),
            Field::new("float64_col", DataType::Float64, true),
            Field::new("bool_col", DataType::Boolean, false),
            Field::new("float16_col", DataType::Float16, false),
        ]));

        let result = get_posql_compatible_schema(&schema);

        assert_eq!(result.fields().len(), 6);
        assert_eq!(result.fields()[0].data_type(), &DataType::Int32);
        assert_eq!(
            result.fields()[1].data_type(),
            &DataType::Decimal256(20, 10)
        );
        assert_eq!(result.fields()[2].data_type(), &DataType::Utf8);
        assert_eq!(
            result.fields()[3].data_type(),
            &DataType::Decimal256(20, 10)
        );
        assert_eq!(result.fields()[4].data_type(), &DataType::Boolean);
        assert_eq!(
            result.fields()[5].data_type(),
            &DataType::Decimal256(20, 10)
        );
    }

    #[test]
    fn we_can_preserve_field_names_during_conversion() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("my_float", DataType::Float32, false),
            Field::new("another_field", DataType::Int64, true),
        ]));

        let result = get_posql_compatible_schema(&schema);

        assert_eq!(result.fields()[0].name(), "my_float");
        assert_eq!(result.fields()[1].name(), "another_field");
    }

    #[test]
    fn we_can_preserve_nullability_during_conversion() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("nullable_float", DataType::Float32, true),
            Field::new("non_nullable_float", DataType::Float64, false),
            Field::new("nullable_int", DataType::Int32, true),
            Field::new("non_nullable_int", DataType::Int64, false),
        ]));

        let result = get_posql_compatible_schema(&schema);

        assert!(result.fields()[0].is_nullable());
        assert!(!result.fields()[1].is_nullable());
        assert!(result.fields()[2].is_nullable());
        assert!(!result.fields()[3].is_nullable());
    }

    #[test]
    fn we_can_handle_empty_schema() {
        let schema = Arc::new(Schema::new(vec![] as Vec<Field>));

        let result = get_posql_compatible_schema(&schema);

        assert_eq!(result.fields().len(), 0);
    }

    #[test]
    fn we_can_preserve_existing_decimal_types() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("decimal128_col", DataType::Decimal128(10, 2), false),
            Field::new("decimal256_col", DataType::Decimal256(30, 5), true),
        ]));

        let result = get_posql_compatible_schema(&schema);

        assert_eq!(result.fields().len(), 2);
        assert_eq!(result.fields()[0].data_type(), &DataType::Decimal128(10, 2));
        assert_eq!(result.fields()[1].data_type(), &DataType::Decimal256(30, 5));
    }
}
