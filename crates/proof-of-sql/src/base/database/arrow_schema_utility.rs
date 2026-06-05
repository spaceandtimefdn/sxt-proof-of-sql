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

    fn assert_field(field: &Field, name: &str, data_type: &DataType, nullable: bool) {
        assert_eq!(field.name(), name);
        assert_eq!(field.data_type(), data_type);
        assert_eq!(field.is_nullable(), nullable);
    }

    #[test]
    fn float_fields_are_converted_to_posql_decimal_fields_without_changing_metadata() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("half", DataType::Float16, false),
            Field::new("single", DataType::Float32, true),
            Field::new("double", DataType::Float64, false),
        ]));

        let converted = get_posql_compatible_schema(&schema);

        assert_field(
            &converted.fields()[0],
            "half",
            &DataType::Decimal256(20, 10),
            false,
        );
        assert_field(
            &converted.fields()[1],
            "single",
            &DataType::Decimal256(20, 10),
            true,
        );
        assert_field(
            &converted.fields()[2],
            "double",
            &DataType::Decimal256(20, 10),
            false,
        );
    }

    #[test]
    fn non_float_fields_keep_their_type_name_and_nullability() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::UInt64, false),
            Field::new("label", DataType::Utf8, true),
            Field::new("active", DataType::Boolean, false),
        ]));

        let converted = get_posql_compatible_schema(&schema);

        assert_field(&converted.fields()[0], "id", &DataType::UInt64, false);
        assert_field(&converted.fields()[1], "label", &DataType::Utf8, true);
        assert_field(&converted.fields()[2], "active", &DataType::Boolean, false);
    }

    #[test]
    fn mixed_schema_preserves_field_order_while_converting_only_float_fields() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::UInt64, false),
            Field::new("score", DataType::Float64, true),
            Field::new("label", DataType::Utf8, true),
            Field::new("ratio", DataType::Float32, false),
        ]));

        let converted = get_posql_compatible_schema(&schema);

        assert_field(&converted.fields()[0], "id", &DataType::UInt64, false);
        assert_field(
            &converted.fields()[1],
            "score",
            &DataType::Decimal256(20, 10),
            true,
        );
        assert_field(&converted.fields()[2], "label", &DataType::Utf8, true);
        assert_field(
            &converted.fields()[3],
            "ratio",
            &DataType::Decimal256(20, 10),
            false,
        );
    }
}
