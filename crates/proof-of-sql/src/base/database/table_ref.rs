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
    fn we_can_construct_table_refs_from_schema_and_table_names() {
        let scoped = TableRef::new("schema", "table");
        assert_eq!(scoped.schema_id().unwrap().value, "schema");
        assert_eq!(scoped.table_id().value, "table");
        assert_eq!(scoped.to_string(), "schema.table");

        let unscoped = TableRef::new("", "table");
        assert!(unscoped.schema_id().is_none());
        assert_eq!(unscoped.table_id().value, "table");
        assert_eq!(unscoped.to_string(), "table");
    }

    #[test]
    fn we_can_construct_table_refs_from_optional_names_and_idents() {
        let from_names = TableRef::from_names(Some("schema"), "table");
        let from_idents = TableRef::from_idents(Some(Ident::new("schema")), Ident::new("table"));
        assert_eq!(from_names, from_idents);

        let unscoped = TableRef::from_names(None, "table");
        assert!(unscoped.schema_id().is_none());
        assert_eq!(unscoped.table_id().value, "table");
    }

    #[test]
    fn we_can_parse_table_refs_from_component_slices() {
        assert_eq!(
            TableRef::from_strs(&["table"]).unwrap(),
            TableRef::from_names(None, "table")
        );
        assert_eq!(
            TableRef::from_strs(&["schema", "table"]).unwrap(),
            TableRef::from_names(Some("schema"), "table")
        );

        assert_eq!(
            TableRef::from_strs(&["catalog", "schema", "table"]).unwrap_err(),
            ParseError::InvalidTableReference {
                table_reference: "catalog,schema,table".to_string()
            }
        );
    }

    #[test]
    fn we_can_parse_table_refs_from_dot_separated_strings() {
        assert_eq!(
            TableRef::try_from("table").unwrap(),
            TableRef::from_names(None, "table")
        );
        assert_eq!(
            "schema.table".parse::<TableRef>().unwrap(),
            TableRef::from_names(Some("schema"), "table")
        );

        assert_eq!(
            TableRef::try_from("catalog.schema.table").unwrap_err(),
            ParseError::InvalidTableReference {
                table_reference: "catalog.schema.table".to_string()
            }
        );
    }

    #[test]
    fn we_can_compare_borrowed_table_refs_as_equivalent_keys() {
        let table_ref = TableRef::from_names(Some("schema"), "table");
        let same = TableRef::new("schema", "table");
        let different_table = TableRef::new("schema", "other");
        let different_schema = TableRef::new("other", "table");

        assert!(<&TableRef as Equivalent<TableRef>>::equivalent(
            &&same, &table_ref
        ));
        assert!(!<&TableRef as Equivalent<TableRef>>::equivalent(
            &&different_table,
            &table_ref
        ));
        assert!(!<&TableRef as Equivalent<TableRef>>::equivalent(
            &&different_schema,
            &table_ref
        ));
    }

    #[test]
    fn we_can_serde_round_trip_table_refs() {
        let table_ref = TableRef::new("schema", "table");
        let serialized = serde_json::to_string(&table_ref).unwrap();
        assert_eq!(serialized, "\"schema.table\"");

        let deserialized: TableRef = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, table_ref);
    }

    #[test]
    fn we_cannot_deserialize_invalid_table_refs() {
        let error = serde_json::from_str::<TableRef>("\"catalog.schema.table\"").unwrap_err();
        assert!(error.to_string().contains("Invalid table reference"));
    }
}
