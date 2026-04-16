use super::ColumnType;
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
}

impl ColumnField {
    /// Create a new `ColumnField` from a name and a type
    #[must_use]
    pub fn new(name: Ident, data_type: ColumnType) -> ColumnField {
        ColumnField { name, data_type }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn we_can_build_and_query_column_fields() {
        let field = ColumnField::new(Ident::new("amount"), ColumnType::BigInt);

        assert_eq!(field.name(), Ident::new("amount"));
        assert_eq!(field.data_type(), ColumnType::BigInt);
    }

    #[test]
    fn we_can_hash_compare_and_serialize_column_fields() {
        let first = ColumnField::new(Ident::new("id"), ColumnType::Int);
        let same = ColumnField::new(Ident::new("id"), ColumnType::Int);
        let different = ColumnField::new(Ident::new("created_at"), ColumnType::BigInt);

        assert_eq!(first, same);
        assert_ne!(first, different);

        let mut set = HashSet::new();
        set.insert(first.clone());
        assert!(set.contains(&same));
        assert!(!set.contains(&different));

        let json = serde_json::to_string(&first).unwrap();
        let round_trip: ColumnField = serde_json::from_str(&json).unwrap();
        assert_eq!(round_trip, first);
    }
}
