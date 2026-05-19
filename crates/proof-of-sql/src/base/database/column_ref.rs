use super::{ColumnField, ColumnType, TableRef};
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Reference of a SQL column
#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct ColumnRef {
    column_id: Ident,
    table_ref: TableRef,
    column_type: ColumnType,
    #[serde(default)]
    nullable: bool,
}

impl ColumnRef {
    /// Create a new `ColumnRef` from a table, column identifier and column type
    #[must_use]
    pub fn new(table_ref: TableRef, column_id: Ident, column_type: ColumnType) -> Self {
        Self {
            column_id,
            table_ref,
            column_type,
            nullable: false,
        }
    }

    /// Create a new nullable `ColumnRef` from a table, column identifier and column type.
    #[must_use]
    pub fn new_nullable(table_ref: TableRef, column_id: Ident, column_type: ColumnType) -> Self {
        Self {
            column_id,
            table_ref,
            column_type,
            nullable: true,
        }
    }

    /// Create a new `ColumnRef` from a table and column field.
    #[must_use]
    pub fn from_field(table_ref: TableRef, field: &ColumnField) -> Self {
        if field.is_nullable() {
            Self::new_nullable(table_ref, field.name(), field.data_type())
        } else {
            Self::new(table_ref, field.name(), field.data_type())
        }
    }

    /// Returns the table reference of this column
    #[must_use]
    pub fn table_ref(&self) -> TableRef {
        self.table_ref.clone()
    }

    /// Returns the column identifier of this column
    #[must_use]
    pub fn column_id(&self) -> Ident {
        self.column_id.clone()
    }

    /// Returns the column type of this column
    #[must_use]
    pub fn column_type(&self) -> &ColumnType {
        &self.column_type
    }

    /// Returns whether this column reference can contain SQL `NULL` values.
    #[must_use]
    pub const fn is_nullable(&self) -> bool {
        self.nullable
    }

    /// Wrap the column output name, type, and nullability within a [`ColumnField`].
    #[must_use]
    pub fn column_field(&self) -> ColumnField {
        if self.nullable {
            ColumnField::new_nullable(self.column_id(), self.column_type)
        } else {
            ColumnField::new(self.column_id(), self.column_type)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_column_refs_are_non_nullable_by_default() {
        let column_ref = ColumnRef::new(
            TableRef::new("sxt", "orders"),
            "amount".into(),
            ColumnType::BigInt,
        );

        assert_eq!(column_ref.column_id(), "amount".into());
        assert_eq!(column_ref.column_type(), &ColumnType::BigInt);
        assert!(!column_ref.is_nullable());
        assert_eq!(
            column_ref.column_field(),
            ColumnField::new("amount".into(), ColumnType::BigInt)
        );
    }

    #[test]
    fn nullable_column_refs_carry_nullable_metadata() {
        let column_ref = ColumnRef::new_nullable(
            TableRef::new("sxt", "orders"),
            "amount".into(),
            ColumnType::BigInt,
        );

        assert!(column_ref.is_nullable());
        assert_eq!(
            column_ref.column_field(),
            ColumnField::new_nullable("amount".into(), ColumnType::BigInt)
        );
    }

    #[test]
    fn column_refs_can_be_built_from_column_fields() {
        let field = ColumnField::new_nullable("amount".into(), ColumnType::BigInt);
        let column_ref = ColumnRef::from_field(TableRef::new("sxt", "orders"), &field);

        assert_eq!(column_ref.column_id(), "amount".into());
        assert_eq!(column_ref.column_type(), &ColumnType::BigInt);
        assert!(column_ref.is_nullable());
    }
}
