use crate::base::database::error::ParseError;
use alloc::{string::ToString, vec::Vec};
use core::{
    fmt,
    fmt::{Display, Formatter},
    str::FromStr,
};
use indexmap::Equivalent;
use proof_of_sql_parser::{impl_serde_from_str, ResourceId};
use sqlparser::ast::Ident;

/// Expression for an SQL table
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TableRef {
    schema_name: Option<Ident>,
    table_name: Ident,
}

impl TableRef {
    /// Creates a new table reference from schema and table names.
    /// If the schema name is empty or None, only the table name is used.
    #[must_use]
    pub fn new(schema_name: impl AsRef<str>, table_name: impl AsRef<str>) -> Self {
        let schema = schema_name.as_ref();
        let table = table_name.as_ref();

        Self {
            schema_name: if schema.is_empty() {
                None
            } else {
                Some(Ident::new(schema.to_string()))
            },
            table_name: Ident::new(table.to_string()),
        }
    }

    /// Returns the identifier of the schema
    /// # Panics
    #[must_use]
    pub fn schema_id(&self) -> Option<&Ident> {
        self.schema_name.as_ref()
    }

    /// Returns the identifier of the table
    /// # Panics
    #[must_use]
    pub fn table_id(&self) -> &Ident {
        &self.table_name
    }

    /// Creates a new table reference from an optional schema and table name.
    #[must_use]
    pub fn from_names(schema_name: Option<&str>, table_name: &str) -> Self {
        Self {
            schema_name: schema_name.map(|s| Ident::new(s.to_string())),
            table_name: Ident::new(table_name.to_string()),
        }
    }

    /// Creates a `TableRef` directly from `Option<Ident>` for schema and `Ident` for table.
    #[must_use]
    pub fn from_idents(schema_name: Option<Ident>, table_name: Ident) -> Self {
        Self {
            schema_name,
            table_name,
        }
    }

    /// Creates a `TableRef` from a slice of string components.
    pub fn from_strs<S: AsRef<str>>(components: &[S]) -> Result<Self, ParseError> {
        match components.len() {
            1 => Ok(Self::from_names(None, components[0].as_ref())),
            2 => Ok(Self::from_names(
                Some(components[0].as_ref()),
                components[1].as_ref(),
            )),
            _ => Err(ParseError::InvalidTableReference {
                table_reference: components
                    .iter()
                    .map(AsRef::as_ref)
                    .collect::<Vec<_>>()
                    .join(","),
            }),
        }
    }
}

/// Creates a `TableRef` from a dot-separated string.
impl TryFrom<&str> for TableRef {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, <Self as TryFrom<&str>>::Error> {
        let components: Vec<_> = s.split('.').map(ToString::to_string).collect();
        match components.len() {
            1 => Ok(Self::from_names(None, &components[0])),
            2 => Ok(Self::from_names(Some(&components[0]), &components[1])),
            _ => Err(ParseError::InvalidTableReference {
                table_reference: s.to_string(),
            }),
        }
    }
}

/// Note: We just need this conversion trait until `SelectStatement` refactor is done
impl From<ResourceId> for TableRef {
    fn from(id: ResourceId) -> Self {
        TableRef {
            schema_name: Some(Ident::from(id.schema())),
            table_name: Ident::from(id.object_name()),
        }
    }
}

impl FromStr for TableRef {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl Equivalent<TableRef> for &TableRef {
    fn equivalent(&self, key: &TableRef) -> bool {
        self.schema_name == key.schema_name && self.table_name == key.table_name
    }
}

impl Display for TableRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(schema) = &self.schema_name {
            write!(f, "{}.{}", schema.value, self.table_name.value)
        } else {
            write!(f, "{}", self.table_name.value)
        }
    }
}

impl_serde_from_str!(TableRef);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_ref_new_with_schema() {
        let table_ref = TableRef::new("test_schema", "test_table");
        assert_eq!(table_ref.schema_id().unwrap().value, "test_schema");
        assert_eq!(table_ref.table_id().value, "test_table");
    }

    #[test]
    fn test_table_ref_new_without_schema() {
        let table_ref = TableRef::new("", "test_table");
        assert!(table_ref.schema_id().is_none());
        assert_eq!(table_ref.table_id().value, "test_table");
    }

    #[test]
    fn test_table_ref_from_names_with_schema() {
        let table_ref = TableRef::from_names(Some("schema"), "table");
        assert_eq!(table_ref.schema_id().unwrap().value, "schema");
        assert_eq!(table_ref.table_id().value, "table");
    }

    #[test]
    fn test_table_ref_from_names_without_schema() {
        let table_ref = TableRef::from_names(None, "table");
        assert!(table_ref.schema_id().is_none());
        assert_eq!(table_ref.table_id().value, "table");
    }

    #[test]
    fn test_table_ref_from_idents() {
        let schema_ident = Some(Ident::new("schema"));
        let table_ident = Ident::new("table");
        let table_ref = TableRef::from_idents(schema_ident.clone(), table_ident.clone());

        assert_eq!(table_ref.schema_id().unwrap().value, "schema");
        assert_eq!(table_ref.table_id().value, "table");
    }

    #[test]
    fn test_table_ref_from_strs_single_component() {
        let components = ["table"];
        let table_ref = TableRef::from_strs(&components).unwrap();

        assert!(table_ref.schema_id().is_none());
        assert_eq!(table_ref.table_id().value, "table");
    }

    #[test]
    fn test_table_ref_from_strs_two_components() {
        let components = ["schema", "table"];
        let table_ref = TableRef::from_strs(&components).unwrap();

        assert_eq!(table_ref.schema_id().unwrap().value, "schema");
        assert_eq!(table_ref.table_id().value, "table");
    }

    #[test]
    fn test_table_ref_from_strs_too_many_components() {
        let components = ["db", "schema", "table"];
        let result = TableRef::from_strs(&components);

        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::InvalidTableReference { table_reference } => {
                assert_eq!(table_reference, "db,schema,table");
            }
        }
    }

    #[test]
    fn test_table_ref_from_strs_empty_components() {
        let components: &[&str] = &[];
        let result = TableRef::from_strs(components);

        assert!(result.is_err());
    }

    #[test]
    fn test_table_ref_try_from_str_single_part() {
        let table_ref = TableRef::try_from("table").unwrap();

        assert!(table_ref.schema_id().is_none());
        assert_eq!(table_ref.table_id().value, "table");
    }

    #[test]
    fn test_table_ref_try_from_str_two_parts() {
        let table_ref = TableRef::try_from("schema.table").unwrap();

        assert_eq!(table_ref.schema_id().unwrap().value, "schema");
        assert_eq!(table_ref.table_id().value, "table");
    }

    #[test]
    fn test_table_ref_try_from_str_too_many_parts() {
        let result = TableRef::try_from("db.schema.table");

        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::InvalidTableReference { table_reference } => {
                assert_eq!(table_reference, "db.schema.table");
            }
        }
    }

    #[test]
    fn test_table_ref_from_str() {
        let table_ref: TableRef = "schema.table".parse().unwrap();

        assert_eq!(table_ref.schema_id().unwrap().value, "schema");
        assert_eq!(table_ref.table_id().value, "table");
    }

    // Note: Removed ResourceId test due to private constructor

    #[test]
    fn test_table_ref_equivalent() {
        let table_ref1 = TableRef::new("schema", "table");
        let table_ref2 = TableRef::new("schema", "table");
        let table_ref3 = TableRef::new("other_schema", "table");
        let table_ref4 = TableRef::new("schema", "other_table");

        assert!(table_ref1.equivalent(&table_ref2));
        assert!(!table_ref1.equivalent(&table_ref3));
        assert!(!table_ref1.equivalent(&table_ref4));
    }

    #[test]
    fn test_table_ref_display_with_schema() {
        let table_ref = TableRef::new("test_schema", "test_table");
        let display_str = format!("{table_ref}");

        assert_eq!(display_str, "test_schema.test_table");
    }

    #[test]
    fn test_table_ref_display_without_schema() {
        let table_ref = TableRef::new("", "test_table");
        let display_str = format!("{table_ref}");

        assert_eq!(display_str, "test_table");
    }

    #[test]
    fn test_table_ref_equality() {
        let table_ref1 = TableRef::new("schema", "table");
        let table_ref2 = TableRef::new("schema", "table");
        let table_ref3 = TableRef::new("other_schema", "table");

        assert_eq!(table_ref1, table_ref2);
        assert_ne!(table_ref1, table_ref3);
    }

    #[test]
    fn test_table_ref_clone() {
        let original = TableRef::new("schema", "table");
        let cloned = original.clone();

        assert_eq!(original, cloned);
        assert_eq!(original.schema_id(), cloned.schema_id());
        assert_eq!(original.table_id(), cloned.table_id());
    }

    #[test]
    fn test_table_ref_debug() {
        let table_ref = TableRef::new("schema", "table");
        let debug_str = format!("{table_ref:?}");

        assert!(debug_str.contains("TableRef"));
        assert!(debug_str.contains("schema"));
        assert!(debug_str.contains("table"));
    }

    #[test]
    fn test_table_ref_hash() {
        use std::collections::HashMap;

        let table_ref1 = TableRef::new("schema", "table");
        let table_ref2 = TableRef::new("schema", "table");
        let table_ref3 = TableRef::new("other_schema", "table");

        let mut map = HashMap::new();
        map.insert(table_ref1.clone(), "value1");
        map.insert(table_ref3, "value3");

        // Should be able to find with equivalent table ref
        assert_eq!(map.get(&table_ref2), Some(&"value1"));
    }

    #[test]
    fn test_table_ref_serialization() {
        let table_ref = TableRef::new("test_schema", "test_table");

        // Test JSON serialization
        let serialized = serde_json::to_string(&table_ref).unwrap();
        let deserialized: TableRef = serde_json::from_str(&serialized).unwrap();

        assert_eq!(table_ref, deserialized);
        assert_eq!(table_ref.schema_id(), deserialized.schema_id());
        assert_eq!(table_ref.table_id(), deserialized.table_id());
    }

    #[test]
    fn test_table_ref_serialization_no_schema() {
        let table_ref = TableRef::new("", "test_table");

        // Test JSON serialization
        let serialized = serde_json::to_string(&table_ref).unwrap();
        let deserialized: TableRef = serde_json::from_str(&serialized).unwrap();

        assert_eq!(table_ref, deserialized);
        assert!(deserialized.schema_id().is_none());
        assert_eq!(table_ref.table_id(), deserialized.table_id());
    }

    #[test]
    fn test_edge_cases_empty_strings() {
        // Empty schema name should result in None schema
        let table_ref = TableRef::new("", "table");
        assert!(table_ref.schema_id().is_none());

        // Empty table name should still create an Ident
        let table_ref2 = TableRef::new("schema", "");
        assert_eq!(table_ref2.table_id().value, "");
    }

    #[test]
    fn test_special_characters_in_names() {
        let table_ref = TableRef::new("schema_with_underscore", "table-with-dash");
        assert_eq!(
            table_ref.schema_id().unwrap().value,
            "schema_with_underscore"
        );
        assert_eq!(table_ref.table_id().value, "table-with-dash");

        let display_str = format!("{table_ref}");
        assert_eq!(display_str, "schema_with_underscore.table-with-dash");
    }

    #[test]
    fn test_dotted_names_in_try_from() {
        // Test edge case with empty parts
        let result = TableRef::try_from("schema.");
        assert!(result.is_ok());
        let table_ref = result.unwrap();
        assert_eq!(table_ref.schema_id().unwrap().value, "schema");
        assert_eq!(table_ref.table_id().value, "");

        let result2 = TableRef::try_from(".table");
        assert!(result2.is_ok());
        let table_ref2 = result2.unwrap();
        assert_eq!(table_ref2.schema_id().unwrap().value, "");
        assert_eq!(table_ref2.table_id().value, "table");
    }
}
