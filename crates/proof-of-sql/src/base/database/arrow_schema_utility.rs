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
    use super::get_posql_compatible_schema;
    use alloc::{sync::Arc, vec};
    use arrow::datatypes::{DataType, Field, Schema};

    #[test]
    fn we_convert_all_floating_point_fields_to_decimal256() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("half", DataType::Float16, false),
            Field::new("single", DataType::Float32, true),
            Field::new("double", DataType::Float64, false),
        ]));

        let compatible_schema = get_posql_compatible_schema(&schema);

        for (index, (name, nullable)) in [("half", false), ("single", true), ("double", false)]
            .into_iter()
            .enumerate()
        {
            let field = compatible_schema.field(index);
            assert_eq!(field.name().as_str(), name);
            assert_eq!(field.data_type(), &DataType::Decimal256(20, 10));
            assert_eq!(field.is_nullable(), nullable);
        }

        assert_eq!(schema.field(0).data_type(), &DataType::Float16);
        assert_eq!(schema.field(1).data_type(), &DataType::Float32);
        assert_eq!(schema.field(2).data_type(), &DataType::Float64);
    }

    #[test]
    fn we_preserve_non_floating_point_field_types_and_nullability() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("name", DataType::Utf8, true),
            Field::new("amount", DataType::Decimal256(75, 3), false),
        ]));

        let compatible_schema = get_posql_compatible_schema(&schema);

        assert_eq!(compatible_schema.field(0).name().as_str(), "id");
        assert_eq!(compatible_schema.field(0).data_type(), &DataType::Int64);
        assert!(!compatible_schema.field(0).is_nullable());

        assert_eq!(compatible_schema.field(1).name().as_str(), "name");
        assert_eq!(compatible_schema.field(1).data_type(), &DataType::Utf8);
        assert!(compatible_schema.field(1).is_nullable());

        assert_eq!(compatible_schema.field(2).name().as_str(), "amount");
        assert_eq!(
            compatible_schema.field(2).data_type(),
            &DataType::Decimal256(75, 3)
        );
        assert!(!compatible_schema.field(2).is_nullable());
    }
}
