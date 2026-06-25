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
    use core::str::FromStr;
    use sqlparser::ast::Ident;

    #[test]
    fn table_ref_new_with_schema_and_table() {
        let t = TableRef::new("myschema", "mytable");
        assert_eq!(t.table_id().value, "mytable");
        assert_eq!(t.schema_id().map(|i| i.value.as_str()), Some("myschema"));
    }

    #[test]
    fn table_ref_new_without_schema() {
        let t = TableRef::new("", "mytable");
        assert!(t.schema_id().is_none());
        assert_eq!(t.table_id().value, "mytable");
    }

    #[test]
    fn table_ref_display_with_schema() {
        let t = TableRef::new("s", "t");
        assert_eq!(alloc::format!("{t}"), "s.t");
    }

    #[test]
    fn table_ref_display_without_schema() {
        let t = TableRef::new("", "t");
        assert_eq!(alloc::format!("{t}"), "t");
    }

    #[test]
    fn table_ref_from_str_with_dot() {
        let t = TableRef::from_str("schema.table").unwrap();
        assert_eq!(t.table_id().value, "table");
        assert_eq!(t.schema_id().map(|i| i.value.as_str()), Some("schema"));
    }

    #[test]
    fn table_ref_from_str_without_dot() {
        let t = TableRef::from_str("table").unwrap();
        assert_eq!(t.table_id().value, "table");
        assert!(t.schema_id().is_none());
    }

    #[test]
    fn table_ref_from_str_two_dots_is_error() {
        assert!(TableRef::from_str("a.b.c").is_err());
    }

    #[test]
    fn table_ref_from_strs_single_component() {
        let t = TableRef::from_strs(&["mytable"]).unwrap();
        assert!(t.schema_id().is_none());
    }

    #[test]
    fn table_ref_from_strs_two_components() {
        let t = TableRef::from_strs(&["s", "t"]).unwrap();
        assert_eq!(t.schema_id().unwrap().value, "s");
    }

    #[test]
    fn table_ref_from_strs_three_components_is_error() {
        assert!(TableRef::from_strs(&["a", "b", "c"]).is_err());
    }

    #[test]
    fn table_ref_from_idents() {
        let t = TableRef::from_idents(Some(Ident::new("s")), Ident::new("t"));
        assert_eq!(t.table_id().value, "t");
        assert_eq!(t.schema_id().unwrap().value, "s");
    }

    #[test]
    fn table_ref_equality() {
        let a = TableRef::new("s", "t");
        let b = TableRef::new("s", "t");
        assert_eq!(a, b);
    }

    #[test]
    fn table_ref_clone_equals_original() {
        let t = TableRef::new("s", "t");
        assert_eq!(t.clone(), t);
    }

    #[test]
    fn table_ref_is_debug_formattable() {
        let t = TableRef::new("s", "t");
        let s = alloc::format!("{t:?}");
        assert!(!s.is_empty());
    }
}
