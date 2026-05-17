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
                    .join("."),
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
    fn we_can_construct_table_refs_from_schema_and_table_names() {
        let table_ref = TableRef::new("public", "users");

        assert_eq!(table_ref.schema_id().unwrap().value, "public");
        assert_eq!(table_ref.table_id().value, "users");
        assert_eq!(table_ref.to_string(), "public.users");
    }

    #[test]
    fn empty_schema_names_are_omitted_from_table_refs() {
        let table_ref = TableRef::new("", "users");

        assert!(table_ref.schema_id().is_none());
        assert_eq!(table_ref.table_id().value, "users");
        assert_eq!(table_ref.to_string(), "users");
    }

    #[test]
    fn we_can_parse_table_refs_from_string_components() {
        assert_eq!(
            TableRef::from_strs(&["users"]).unwrap(),
            TableRef::from_names(None, "users")
        );
        assert_eq!(
            TableRef::from_strs(&["public", "users"]).unwrap(),
            TableRef::from_names(Some("public"), "users")
        );
    }

    #[test]
    fn we_error_when_table_ref_has_too_many_string_components() {
        let err = TableRef::from_strs(&["a", "b", "c"]).unwrap_err();

        assert!(matches!(
            err,
            ParseError::InvalidTableReference {
                table_reference
            } if table_reference == "a.b.c"
        ));
    }

    #[test]
    fn we_can_try_from_dot_separated_table_refs() {
        assert_eq!(
            TableRef::try_from("public.users").unwrap(),
            TableRef::from_names(Some("public"), "users")
        );
        assert_eq!(
            TableRef::try_from("users").unwrap(),
            TableRef::from_names(None, "users")
        );
    }

    #[test]
    fn we_error_when_try_from_has_too_many_dot_separated_components() {
        let err = TableRef::try_from("a.b.c").unwrap_err();

        assert!(matches!(
            err,
            ParseError::InvalidTableReference {
                table_reference
            } if table_reference == "a.b.c"
        ));
    }

    #[test]
    fn table_ref_equivalence_matches_schema_and_table_names() {
        let table_ref = TableRef::from_names(Some("public"), "users");
        let same_table_ref = TableRef::from_names(Some("public"), "users");
        let different_schema = TableRef::from_names(Some("private"), "users");
        let different_table = TableRef::from_names(Some("public"), "orders");

        assert!((&same_table_ref).equivalent(&table_ref));
        assert!(!(&different_schema).equivalent(&table_ref));
        assert!(!(&different_table).equivalent(&table_ref));
    }

    #[test]
    fn table_refs_serialize_and_deserialize_as_strings() {
        let table_ref = TableRef::from_names(Some("public"), "users");

        let serialized = serde_json::to_string(&table_ref).unwrap();
        let deserialized: TableRef = serde_json::from_str(&serialized).unwrap();

        assert_eq!(serialized, "\"public.users\"");
        assert_eq!(deserialized, table_ref);
    }

    #[test]
    fn deserializing_invalid_table_ref_strings_returns_an_error() {
        let err = serde_json::from_str::<TableRef>("\"a.b.c\"").unwrap_err();

        assert!(err.to_string().contains("a.b.c"));
    }

    #[test]
    fn table_refs_can_be_constructed_from_idents() {
        let schema = Ident::new("public");
        let table = Ident::new("users");

        let table_ref = TableRef::from_idents(Some(schema), table);

        assert_eq!(table_ref.to_string(), "public.users");
    }

    #[test]
    fn empty_component_slices_are_invalid_table_refs() {
        let err = TableRef::from_strs::<&str>(&[]).unwrap_err();

        assert!(matches!(
            err,
            ParseError::InvalidTableReference {
                table_reference
            } if table_reference.is_empty()
        ));
    }
}
