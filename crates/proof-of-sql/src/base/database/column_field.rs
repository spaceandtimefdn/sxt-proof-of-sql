use super::{ColumnRef, ColumnType};
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// This type is used to represent the metadata
/// of a column in a table. Namely: it's name and type.
///
/// This is the analog of a `Field` in Apache Arrow.
#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct ColumnField {
    name: Ident,
    data_type: ColumnType,
    #[serde(default)]
    nullable: bool,
}

impl ColumnField {
    /// Create a new `ColumnField` from a name and a type
    #[must_use]
    pub fn new(name: Ident, data_type: ColumnType) -> ColumnField {
        ColumnField {
            name,
            data_type,
            nullable: false,
        }
    }

    /// Create a new nullable `ColumnField` from a name and a type.
    #[must_use]
    pub fn new_nullable(name: Ident, data_type: ColumnType) -> ColumnField {
        ColumnField {
            name,
            data_type,
            nullable: true,
        }
    }

    /// Returns the name of the column
    #[must_use]
    pub fn name(&self) -> Ident {
        self.name.clone()
    }

    /// Returns the type of the column
    #[must_use]
    pub fn data_type(&self) -> ColumnType {
        self.data_type
    }

    /// Returns whether the column can contain SQL `NULL` values.
    #[must_use]
    pub const fn is_nullable(&self) -> bool {
        self.nullable
    }

    /// Expand logical nullable fields into the physical value plus presence result schema.
    pub(crate) fn value_and_presence_fields<T>(fields: T) -> Vec<ColumnField>
    where
        T: IntoIterator<Item = ColumnField>,
    {
        fields
            .into_iter()
            .flat_map(|field| {
                let presence_field = field.is_nullable().then(|| {
                    ColumnField::new(
                        ColumnRef::presence_column_id(&field.name()),
                        ColumnType::Boolean,
                    )
                });
                core::iter::once(field).chain(presence_field)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_column_fields_are_non_nullable_by_default() {
        let field = ColumnField::new("amount".into(), ColumnType::BigInt);

        assert_eq!(field.name(), "amount".into());
        assert_eq!(field.data_type(), ColumnType::BigInt);
        assert!(!field.is_nullable());
    }

    #[test]
    fn nullable_column_fields_carry_nullable_metadata() {
        let field = ColumnField::new_nullable("amount".into(), ColumnType::BigInt);

        assert_eq!(field.name(), "amount".into());
        assert_eq!(field.data_type(), ColumnType::BigInt);
        assert!(field.is_nullable());
    }

    #[test]
    fn nullable_fields_expand_to_value_and_presence_fields() {
        let physical_fields = ColumnField::value_and_presence_fields([
            ColumnField::new("id".into(), ColumnType::BigInt),
            ColumnField::new_nullable("amount".into(), ColumnType::BigInt),
        ]);

        assert_eq!(
            physical_fields,
            vec![
                ColumnField::new("id".into(), ColumnType::BigInt),
                ColumnField::new_nullable("amount".into(), ColumnType::BigInt),
                ColumnField::new("__posql_presence_amount".into(), ColumnType::Boolean),
            ]
        );
    }
}
