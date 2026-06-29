use super::{ColumnField, ColumnType};
use alloc::{format, vec::Vec};
use snafu::Snafu;
use sqlparser::ast::Ident;

/// Suffix reserved for generated nullable-column presence fields.
pub const PRESENCE_COLUMN_SUFFIX: &str = "__presence";

/// Errors from converting between logical nullable schemas and physical proof schemas.
#[derive(Snafu, Debug, PartialEq, Eq)]
pub enum NullableSchemaError {
    /// A logical user column collides with the generated presence column name.
    #[snafu(display(
        "nullable column {column_id} collides with generated presence column {presence_column_id}"
    ))]
    PresenceColumnCollision {
        /// The nullable logical value column.
        column_id: Ident,
        /// The generated physical presence column.
        presence_column_id: Ident,
    },
}

/// Result type for nullable schema contract conversions.
pub type NullableSchemaResult<T> = Result<T, NullableSchemaError>;

/// Return the generated physical presence-column identifier for a nullable logical column.
#[must_use]
pub fn presence_column_id(column_id: &Ident) -> Ident {
    Ident::new(format!("{}{}", column_id.value, PRESENCE_COLUMN_SUFFIX))
}

/// Return the value-column identifier if `column_id` uses the generated presence suffix.
#[must_use]
pub fn value_column_id_from_presence(column_id: &Ident) -> Option<Ident> {
    column_id
        .value
        .strip_suffix(PRESENCE_COLUMN_SUFFIX)
        .filter(|value_id| !value_id.is_empty())
        .map(Ident::new)
}

/// Return true when `field` is a generated presence field for a nullable value field.
#[must_use]
pub fn is_generated_presence_field(field: &ColumnField, fields: &[ColumnField]) -> bool {
    if field.data_type() != ColumnType::Boolean {
        return false;
    }
    let Some(value_column_id) = value_column_id_from_presence(&field.name()) else {
        return false;
    };
    fields.iter().any(|candidate| {
        candidate.name() == value_column_id && presence_column_id(&candidate.name()) == field.name()
    })
}

/// Convert a logical nullable schema into the physical value/presence schema used by proofs.
pub fn physical_column_fields_from_logical_schema(
    logical_fields: &[ColumnField],
) -> NullableSchemaResult<Vec<ColumnField>> {
    let mut physical_fields = Vec::new();
    for field in logical_fields {
        if field.is_nullable() {
            let presence_id = presence_column_id(&field.name());
            if logical_fields
                .iter()
                .any(|candidate| candidate.name() == presence_id)
            {
                return Err(NullableSchemaError::PresenceColumnCollision {
                    column_id: field.name(),
                    presence_column_id: presence_id,
                });
            }
            physical_fields.push(ColumnField::new(field.name(), field.data_type()));
            physical_fields.push(ColumnField::new(presence_id, ColumnType::Boolean));
        } else {
            physical_fields.push(field.clone());
        }
    }
    Ok(physical_fields)
}

/// Convert a physical value/presence schema back into its logical nullable schema.
#[must_use]
pub fn logical_column_fields_from_physical_schema(
    physical_fields: &[ColumnField],
) -> Vec<ColumnField> {
    physical_fields
        .iter()
        .filter_map(|field| {
            if is_generated_presence_field(field, physical_fields) {
                None
            } else {
                let presence_id = presence_column_id(&field.name());
                let has_presence = physical_fields.iter().any(|candidate| {
                    candidate.name() == presence_id && candidate.data_type() == ColumnType::Boolean
                });
                if has_presence {
                    Some(ColumnField::new_nullable(field.name(), field.data_type()))
                } else {
                    Some(field.clone())
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(fields: &[ColumnField]) -> Vec<Ident> {
        fields.iter().map(ColumnField::name).collect()
    }

    #[test]
    fn nullable_logical_fields_expand_to_value_and_presence_fields() {
        let logical_fields = vec![ColumnField::new_nullable(
            "score".into(),
            ColumnType::BigInt,
        )];

        let physical_fields = physical_column_fields_from_logical_schema(&logical_fields).unwrap();

        assert_eq!(
            names(&physical_fields),
            vec!["score".into(), "score__presence".into()]
        );
        assert!(!physical_fields[0].is_nullable());
        assert_eq!(physical_fields[1].data_type(), ColumnType::Boolean);
    }

    #[test]
    fn user_presence_suffix_fields_are_preserved_without_matching_nullable_value() {
        let physical_fields = vec![
            ColumnField::new("score__presence".into(), ColumnType::Boolean),
            ColumnField::new("other".into(), ColumnType::BigInt),
        ];

        let logical_fields = logical_column_fields_from_physical_schema(&physical_fields);

        assert_eq!(
            names(&logical_fields),
            vec!["score__presence".into(), "other".into()]
        );
        assert!(logical_fields.iter().all(|field| !field.is_nullable()));
    }

    #[test]
    fn logical_schemas_reject_generated_presence_name_collisions() {
        let logical_fields = vec![
            ColumnField::new_nullable("score".into(), ColumnType::BigInt),
            ColumnField::new("score__presence".into(), ColumnType::Boolean),
        ];

        let err = physical_column_fields_from_logical_schema(&logical_fields).unwrap_err();

        assert_eq!(
            err,
            NullableSchemaError::PresenceColumnCollision {
                column_id: "score".into(),
                presence_column_id: "score__presence".into(),
            }
        );
    }

    #[test]
    fn physical_value_and_boolean_presence_round_trip_to_logical_nullable_field() {
        let physical_fields =
            physical_column_fields_from_logical_schema(&[ColumnField::new_nullable(
                "score".into(),
                ColumnType::BigInt,
            )])
            .unwrap();

        let logical_fields = logical_column_fields_from_physical_schema(&physical_fields);

        assert_eq!(names(&logical_fields), vec!["score".into()]);
        assert!(logical_fields[0].is_nullable());
        assert_eq!(logical_fields[0].data_type(), ColumnType::BigInt);
    }

    #[test]
    fn non_boolean_suffix_fields_are_not_generated_presence_fields() {
        let physical_fields = vec![
            ColumnField::new_nullable("score".into(), ColumnType::BigInt),
            ColumnField::new("score__presence".into(), ColumnType::BigInt),
        ];

        let logical_fields = logical_column_fields_from_physical_schema(&physical_fields);

        assert_eq!(
            names(&logical_fields),
            vec!["score".into(), "score__presence".into()]
        );
        assert!(logical_fields[0].is_nullable());
        assert!(!logical_fields[1].is_nullable());
    }
}
