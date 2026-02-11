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

    #[test]
    fn we_can_get_schema_id() {
        let table_ref = TableRef::new("my_schema", "my_table");
        let schema_id = table_ref.schema_id();
        assert!(schema_id.is_some());
        assert_eq!(schema_id.unwrap().value, "my_schema");

        let table_ref_no_schema = TableRef::new("", "my_table");
        assert!(table_ref_no_schema.schema_id().is_none());
    }

    #[test]
    fn we_can_get_table_id() {
        let table_ref = TableRef::new("my_schema", "my_table");
        let table_id = table_ref.table_id();
        assert_eq!(table_id.value, "my_table");

        let table_ref_no_schema = TableRef::new("", "another_table");
        assert_eq!(table_ref_no_schema.table_id().value, "another_table");
    }

    #[test]
    fn we_can_create_table_ref_from_idents() {
        let schema_ident = Ident::new("test_schema".to_string());
        let table_ident = Ident::new("test_table".to_string());

        let table_ref = TableRef::from_idents(Some(schema_ident.clone()), table_ident.clone());

        assert_eq!(table_ref.schema_id().unwrap().value, "test_schema");
        assert_eq!(table_ref.table_id().value, "test_table");

        // test with None schema
        let table_ref_no_schema = TableRef::from_idents(None, table_ident);
        assert!(table_ref_no_schema.schema_id().is_none());
        assert_eq!(table_ref_no_schema.table_id().value, "test_table");
    }

    #[test]
    fn we_can_create_table_ref_from_strs_with_one_component() {
        let components = vec!["my_table"];
        let table_ref = TableRef::from_strs(&components).unwrap();

        assert!(table_ref.schema_id().is_none());
        assert_eq!(table_ref.table_id().value, "my_table");
    }

    #[test]
    fn we_can_create_table_ref_from_strs_with_two_components() {
        let components = vec!["my_schema", "my_table"];
        let table_ref = TableRef::from_strs(&components).unwrap();

        assert_eq!(table_ref.schema_id().unwrap().value, "my_schema");
        assert_eq!(table_ref.table_id().value, "my_table");
    }

    #[test]
    fn we_cannot_create_table_ref_from_strs_with_invalid_components() {
        let components = vec!["one", "two", "three"];
        let result = TableRef::from_strs(&components);

        assert!(matches!(
            result,
            Err(ParseError::InvalidTableReference { .. })
        ));

        // test with empty components
        let empty_components: Vec<&str> = vec![];
        let result = TableRef::from_strs(&empty_components);
        assert!(matches!(
            result,
            Err(ParseError::InvalidTableReference { .. })
        ));
    }

    #[test]
    fn we_cannot_create_table_ref_from_str_with_too_many_dots() {
        let result = TableRef::try_from("one.two.three");

        assert!(matches!(
            result,
            Err(ParseError::InvalidTableReference { .. })
        ));
    }

    #[test]
    fn we_can_deserialize_table_ref_from_json() {
        let json = r#""schema.table""#;
        let table_ref: TableRef = serde_json::from_str(json).unwrap();

        assert_eq!(table_ref.schema_id().unwrap().value, "schema");
        assert_eq!(table_ref.table_id().value, "table");

        // test without schema
        let json_no_schema = r#""table""#;
        let table_ref_no_schema: TableRef = serde_json::from_str(json_no_schema).unwrap();

        assert!(table_ref_no_schema.schema_id().is_none());
        assert_eq!(table_ref_no_schema.table_id().value, "table");
    }

    #[test]
    fn we_can_serialize_and_deserialize_table_ref() {
        let table_ref = TableRef::new("my_schema", "my_table");
        let serialized = serde_json::to_string(&table_ref).unwrap();
        let deserialized: TableRef = serde_json::from_str(&serialized).unwrap();

        assert_eq!(table_ref, deserialized);

        // test without schema
        let table_ref_no_schema = TableRef::new("", "my_table");
        let serialized = serde_json::to_string(&table_ref_no_schema).unwrap();
        let deserialized: TableRef = serde_json::from_str(&serialized).unwrap();

        assert_eq!(table_ref_no_schema, deserialized);
    }

    #[test]
    fn we_can_use_display_trait_for_table_ref() {
        let table_ref = TableRef::new("my_schema", "my_table");
        assert_eq!(format!("{table_ref}"), "my_schema.my_table");

        let table_ref_no_schema = TableRef::new("", "my_table");
        assert_eq!(format!("{table_ref_no_schema}"), "my_table");
    }

    #[test]
    fn we_can_use_from_str_trait_for_table_ref() {
        let table_ref: TableRef = "schema.table".parse().unwrap();
        assert_eq!(table_ref.schema_id().unwrap().value, "schema");
        assert_eq!(table_ref.table_id().value, "table");

        let table_ref_no_schema: TableRef = "table".parse().unwrap();
        assert!(table_ref_no_schema.schema_id().is_none());
        assert_eq!(table_ref_no_schema.table_id().value, "table");
    }
}
