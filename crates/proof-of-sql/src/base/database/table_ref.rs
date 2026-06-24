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
    use super::TableRef;
    use alloc::string::ToString;
    use core::str::FromStr;

    #[test]
    fn new_with_schema_and_table_sets_both_names() {
        let t = TableRef::new("myschema", "mytable");
        assert_eq!(t.schema_id().unwrap().value, "myschema");
        assert_eq!(t.table_id().value, "mytable");
    }

    #[test]
    fn new_with_empty_schema_gives_no_schema() {
        let t = TableRef::new("", "standalone");
        assert!(t.schema_id().is_none());
        assert_eq!(t.table_id().value, "standalone");
    }

    #[test]
    fn from_names_with_some_schema() {
        let t = TableRef::from_names(Some("ns"), "t");
        assert_eq!(t.schema_id().unwrap().value, "ns");
        assert_eq!(t.table_id().value, "t");
    }

    #[test]
    fn from_names_with_none_schema() {
        let t = TableRef::from_names(None, "only_table");
        assert!(t.schema_id().is_none());
        assert_eq!(t.table_id().value, "only_table");
    }

    #[test]
    fn display_with_schema_uses_dot_notation() {
        let t = TableRef::new("myschema", "mytable");
        assert_eq!(t.to_string(), "myschema.mytable");
    }

    #[test]
    fn display_without_schema_shows_table_only() {
        let t = TableRef::new("", "standalone");
        assert_eq!(t.to_string(), "standalone");
    }

    #[test]
    fn try_from_dot_separated_string_parses_schema_and_table() {
        let t = TableRef::try_from("schema.table").unwrap();
        assert_eq!(t.schema_id().unwrap().value, "schema");
        assert_eq!(t.table_id().value, "table");
    }

    #[test]
    fn try_from_single_name_gives_no_schema() {
        let t = TableRef::try_from("mytable").unwrap();
        assert!(t.schema_id().is_none());
        assert_eq!(t.table_id().value, "mytable");
    }

    #[test]
    fn try_from_three_parts_is_error() {
        assert!(TableRef::try_from("a.b.c").is_err());
    }

    #[test]
    fn from_strs_one_component_gives_table_only() {
        let t = TableRef::from_strs(&["table"]).unwrap();
        assert_eq!(t.to_string(), "table");
        assert!(t.schema_id().is_none());
    }

    #[test]
    fn from_strs_two_components_gives_schema_and_table() {
        let t = TableRef::from_strs(&["schema", "table"]).unwrap();
        assert_eq!(t.to_string(), "schema.table");
    }

    #[test]
    fn from_strs_three_components_is_error() {
        assert!(TableRef::from_strs(&["a", "b", "c"]).is_err());
    }

    #[test]
    fn equal_table_refs_compare_equal() {
        let a = TableRef::new("s", "t");
        let b = TableRef::new("s", "t");
        assert_eq!(a, b);
    }

    #[test]
    fn different_schema_refs_compare_unequal() {
        let a = TableRef::new("s1", "t");
        let b = TableRef::new("s2", "t");
        assert_ne!(a, b);
    }

    #[test]
    fn clone_creates_equal_instance() {
        let a = TableRef::new("schema", "table");
        assert_eq!(a.clone(), a);
    }

    #[test]
    fn from_str_roundtrip_preserves_display() {
        let original = "myschema.mytable";
        let t = TableRef::from_str(original).unwrap();
        assert_eq!(t.to_string(), original);
    }
}
