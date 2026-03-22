use crate::base::database::error::ParseError;
use alloc::{string::ToString, vec::Vec};
use core::{
    fmt,
    fmt::{Display, Formatter},
    str::FromStr,
};
use indexmap::Equivalent;
use serde::{Deserialize, Serialize};
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

impl Serialize for TableRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
impl<'d> Deserialize<'d> for TableRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'d>,
    {
        let string = alloc::string::String::deserialize(deserializer)?;
        TableRef::from_str(&string).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn we_can_create_table_ref_with_schema_and_table() {
        let table_ref = TableRef::new("my_schema", "my_table");
        assert_eq!(table_ref.schema_id().unwrap().value, "my_schema");
        assert_eq!(table_ref.table_id().value, "my_table");
    }

    #[test]
    fn we_can_create_table_ref_with_empty_schema() {
        let table_ref = TableRef::new("", "my_table");
        assert!(table_ref.schema_id().is_none());
        assert_eq!(table_ref.table_id().value, "my_table");
    }

    #[test]
    fn we_can_create_table_ref_from_names_with_none_schema() {
        let table_ref = TableRef::from_names(None, "my_table");
        assert!(table_ref.schema_id().is_none());
        assert_eq!(table_ref.table_id().value, "my_table");
    }

    #[test]
    fn we_can_create_table_ref_from_names_with_some_schema() {
        let table_ref = TableRef::from_names(Some("public"), "users");
        assert_eq!(table_ref.schema_id().unwrap().value, "public");
        assert_eq!(table_ref.table_id().value, "users");
    }

    #[test]
    fn we_can_create_table_ref_from_idents() {
        let schema_ident = Ident::new("analytics");
        let table_ident = Ident::new("events");
        let table_ref = TableRef::from_idents(Some(schema_ident.clone()), table_ident.clone());
        assert_eq!(table_ref.schema_id().unwrap().value, "analytics");
        assert_eq!(table_ref.table_id().value, "events");
    }

    #[test]
    fn we_can_create_table_ref_from_idents_with_none_schema() {
        let table_ident = Ident::new("events");
        let table_ref = TableRef::from_idents(None, table_ident.clone());
        assert!(table_ref.schema_id().is_none());
        assert_eq!(table_ref.table_id().value, "events");
    }

    #[test]
    fn we_can_create_table_ref_from_single_component_strs() {
        let components: Vec<&str> = vec!["my_table"];
        let table_ref = TableRef::from_strs(&components).unwrap();
        assert!(table_ref.schema_id().is_none());
        assert_eq!(table_ref.table_id().value, "my_table");
    }

    #[test]
    fn we_can_create_table_ref_from_two_component_strs() {
        let components: Vec<&str> = vec!["my_schema", "my_table"];
        let table_ref = TableRef::from_strs(&components).unwrap();
        assert_eq!(table_ref.schema_id().unwrap().value, "my_schema");
        assert_eq!(table_ref.table_id().value, "my_table");
    }

    #[test]
    fn we_cannot_create_table_ref_from_three_component_strs() {
        let components: Vec<&str> = vec!["a", "b", "c"];
        let result = TableRef::from_strs(&components);
        assert!(result.is_err());
    }

    #[test]
    fn we_cannot_create_table_ref_from_empty_strs() {
        let components: Vec<&str> = vec![];
        let result = TableRef::from_strs(&components);
        assert!(result.is_err());
    }

    #[test]
    fn we_can_try_from_dotted_string() {
        let table_ref = TableRef::try_from("schema.table").unwrap();
        assert_eq!(table_ref.schema_id().unwrap().value, "schema");
        assert_eq!(table_ref.table_id().value, "table");
    }

    #[test]
    fn we_can_try_from_simple_string() {
        let table_ref = TableRef::try_from("users").unwrap();
        assert!(table_ref.schema_id().is_none());
        assert_eq!(table_ref.table_id().value, "users");
    }

    #[test]
    fn we_cannot_try_from_triple_dotted_string() {
        let result = TableRef::try_from("a.b.c");
        assert!(result.is_err());
    }

    #[test]
    fn we_can_parse_table_ref_from_str() {
        let table_ref: TableRef = "public.users".parse().unwrap();
        assert_eq!(table_ref.schema_id().unwrap().value, "public");
        assert_eq!(table_ref.table_id().value, "users");
    }

    #[test]
    fn we_can_display_table_ref_with_schema() {
        let table_ref = TableRef::new("public", "users");
        assert_eq!(table_ref.to_string(), "public.users");
    }

    #[test]
    fn we_can_display_table_ref_without_schema() {
        let table_ref = TableRef::new("", "users");
        assert_eq!(table_ref.to_string(), "users");
    }

    #[test]
    fn we_can_roundtrip_serde_json() {
        let table_ref = TableRef::new("my_schema", "my_table");
        let json = serde_json::to_string(&table_ref).unwrap();
        assert_eq!(json, "\"my_schema.my_table\"");
        let deserialized: TableRef = serde_json::from_str(&json).unwrap();
        assert_eq!(table_ref, deserialized);
    }

    #[test]
    fn we_can_roundtrip_serde_json_without_schema() {
        let table_ref = TableRef::new("", "my_table");
        let json = serde_json::to_string(&table_ref).unwrap();
        assert_eq!(json, "\"my_table\"");
        let deserialized: TableRef = serde_json::from_str(&json).unwrap();
        assert_eq!(table_ref, deserialized);
    }

    #[test]
    fn equivalent_ref_matches() {
        let table_ref = TableRef::new("schema", "table");
        let ref_to_table_ref = &table_ref;
        assert!(Equivalent::<TableRef>::equivalent(
            &ref_to_table_ref,
            &table_ref
        ));
    }

    #[test]
    fn equivalent_ref_does_not_match_different_table() {
        let table_ref_a = TableRef::new("schema", "table_a");
        let table_ref_b = TableRef::new("schema", "table_b");
        let ref_to_a = &table_ref_a;
        assert!(!Equivalent::<TableRef>::equivalent(
            &ref_to_a,
            &table_ref_b
        ));
    }

    #[test]
    fn equivalent_ref_does_not_match_different_schema() {
        let table_ref_a = TableRef::new("schema_a", "table");
        let table_ref_b = TableRef::new("schema_b", "table");
        let ref_to_a = &table_ref_a;
        assert!(!Equivalent::<TableRef>::equivalent(
            &ref_to_a,
            &table_ref_b
        ));
    }

    #[test]
    fn equivalent_ref_does_not_match_none_vs_some_schema() {
        let table_ref_no_schema = TableRef::new("", "table");
        let table_ref_with_schema = TableRef::new("schema", "table");
        let ref_to_no_schema = &table_ref_no_schema;
        assert!(!Equivalent::<TableRef>::equivalent(
            &ref_to_no_schema,
            &table_ref_with_schema
        ));
    }

    #[test]
    fn table_ref_clone_preserves_equality() {
        let table_ref = TableRef::new("schema", "table");
        let cloned = table_ref.clone();
        assert_eq!(table_ref, cloned);
    }

    #[test]
    fn table_ref_debug_output_is_meaningful() {
        let table_ref = TableRef::new("schema", "table");
        let debug_str = format!("{:?}", table_ref);
        assert!(debug_str.contains("TableRef"));
        assert!(debug_str.contains("schema"));
        assert!(debug_str.contains("table"));
    }
}
