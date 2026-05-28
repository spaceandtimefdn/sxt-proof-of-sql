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
    fn new_with_schema_and_table() {
        let tr = TableRef::new("myschema", "mytable");
        assert_eq!(tr.schema_id().unwrap().value, "myschema");
        assert_eq!(tr.table_id().value, "mytable");
    }

    #[test]
    fn new_with_empty_schema() {
        let tr = TableRef::new("", "mytable");
        assert!(tr.schema_id().is_none());
        assert_eq!(tr.table_id().value, "mytable");
    }

    #[test]
    fn from_names_some_schema() {
        let tr = TableRef::from_names(Some("schema1"), "table1");
        assert_eq!(tr.schema_id().unwrap().value, "schema1");
        assert_eq!(tr.table_id().value, "table1");
    }

    #[test]
    fn from_names_none_schema() {
        let tr = TableRef::from_names(None, "table1");
        assert!(tr.schema_id().is_none());
        assert_eq!(tr.table_id().value, "table1");
    }

    #[test]
    fn from_idents_with_schema() {
        let schema = Ident::new("s");
        let table = Ident::new("t");
        let tr = TableRef::from_idents(Some(schema.clone()), table.clone());
        assert_eq!(tr.schema_id().unwrap().value, "s");
        assert_eq!(tr.table_id().value, "t");
    }

    #[test]
    fn from_idents_without_schema() {
        let table = Ident::new("t");
        let tr = TableRef::from_idents(None, table);
        assert!(tr.schema_id().is_none());
        assert_eq!(tr.table_id().value, "t");
    }

    #[test]
    fn from_strs_single_component() {
        let tr = TableRef::from_strs(&["mytable"]).unwrap();
        assert!(tr.schema_id().is_none());
        assert_eq!(tr.table_id().value, "mytable");
    }

    #[test]
    fn from_strs_two_components() {
        let tr = TableRef::from_strs(&["myschema", "mytable"]).unwrap();
        assert_eq!(tr.schema_id().unwrap().value, "myschema");
        assert_eq!(tr.table_id().value, "mytable");
    }

    #[test]
    fn from_strs_three_components_errors() {
        let result = TableRef::from_strs(&["a", "b", "c"]);
        assert!(result.is_err());
    }

    #[test]
    fn try_from_dot_separated() {
        let tr: TableRef = "schema1.table1".try_into().unwrap();
        assert_eq!(tr.schema_id().unwrap().value, "schema1");
        assert_eq!(tr.table_id().value, "table1");
    }

    #[test]
    fn try_from_single_name() {
        let tr: TableRef = "justtable".try_into().unwrap();
        assert!(tr.schema_id().is_none());
        assert_eq!(tr.table_id().value, "justtable");
    }

    #[test]
    fn try_from_too_many_dots_errors() {
        let result: Result<TableRef, _> = "a.b.c".try_into();
        assert!(result.is_err());
    }

    #[test]
    fn from_str_works() {
        let tr: TableRef = "schema.table".parse().unwrap();
        assert_eq!(tr.to_string(), "schema.table");
    }

    #[test]
    fn display_with_schema() {
        let tr = TableRef::new("s", "t");
        assert_eq!(tr.to_string(), "s.t");
    }

    #[test]
    fn display_without_schema() {
        let tr = TableRef::new("", "t");
        assert_eq!(tr.to_string(), "t");
    }

    #[test]
    fn serialize_deserialize_roundtrip() {
        let tr = TableRef::new("schema", "table");
        let json = serde_json::to_string(&tr).unwrap();
        let deserialized: TableRef = serde_json::from_str(&json).unwrap();
        assert_eq!(tr, deserialized);
    }

    #[test]
    fn serialize_without_schema() {
        let tr = TableRef::new("", "table");
        let json = serde_json::to_string(&tr).unwrap();
        assert_eq!(json, r#""table""#);
    }

    #[test]
    fn equivalent_ref() {
        let tr1 = TableRef::new("s", "t");
        let tr2 = TableRef::new("s", "t");
        assert!(Equivalent::equivalent(&&tr1, &tr2));
    }

    #[test]
    fn not_equivalent_different_schema() {
        let tr1 = TableRef::new("s1", "t");
        let tr2 = TableRef::new("s2", "t");
        assert!(!Equivalent::equivalent(&&tr1, &tr2));
    }

    #[test]
    fn clone_and_eq() {
        let tr1 = TableRef::new("s", "t");
        let tr2 = tr1.clone();
        assert_eq!(tr1, tr2);
    }

    #[test]
    fn debug_format() {
        let tr = TableRef::new("s", "t");
        let debug = format!("{:?}", tr);
        assert!(debug.contains("TableRef"));
    }
}
