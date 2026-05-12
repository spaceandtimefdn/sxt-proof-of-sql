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
    use alloc::{sync::Arc, vec};
    use arrow::datatypes::{DataType, Field, Schema};

    #[test]
    fn we_can_convert_floating_point_fields_to_decimal256() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("half", DataType::Float16, false),
            Field::new("single", DataType::Float32, true),
            Field::new("double", DataType::Float64, false),
        ]));

        let converted = get_posql_compatible_schema(&schema);

        for field in converted.fields() {
            assert_eq!(field.data_type(), &DataType::Decimal256(20, 10));
        }
        assert!(!converted.field(0).is_nullable());
        assert!(converted.field(1).is_nullable());
        assert!(!converted.field(2).is_nullable());
        assert_eq!(converted.field(0).name(), "half");
        assert_eq!(converted.field(1).name(), "single");
        assert_eq!(converted.field(2).name(), "double");
    }

    #[test]
    fn we_keep_non_floating_point_fields_unchanged() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("name", DataType::Utf8, true),
            Field::new("amount", DataType::Decimal128(18, 4), false),
        ]));

        let converted = get_posql_compatible_schema(&schema);

        assert_eq!(converted.field(0), schema.field(0));
        assert_eq!(converted.field(1), schema.field(1));
        assert_eq!(converted.field(2), schema.field(2));
    }

    #[test]
    fn we_can_convert_empty_schemas() {
        let schema = Arc::new(Schema::empty());

        let converted = get_posql_compatible_schema(&schema);

        assert!(converted.fields().is_empty());
    }
}
