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
    fn constructors_normalize_empty_schema_and_preserve_idents() {
        let table = TableRef::new("", "orders");
        assert_eq!(table.schema_id(), None);
        assert_eq!(table.table_id().value, "orders");
        assert_eq!(table.to_string(), "orders");

        let table = TableRef::from_names(Some("analytics"), "orders");
        assert_eq!(table.schema_id().unwrap().value, "analytics");
        assert_eq!(table.table_id().value, "orders");
        assert_eq!(table.to_string(), "analytics.orders");

        let table = TableRef::from_idents(Some(Ident::new("public")), Ident::new("lineitem"));
        assert_eq!(table.schema_id().unwrap().value, "public");
        assert_eq!(table.table_id().value, "lineitem");
        assert_eq!(table.to_string(), "public.lineitem");
    }

    #[test]
    fn parses_from_component_slices_and_dot_separated_strings() {
        assert_eq!(
            TableRef::from_strs(&["orders"]).unwrap(),
            TableRef::from_names(None, "orders")
        );
        assert_eq!(
            TableRef::from_strs(&["public", "orders"]).unwrap(),
            TableRef::from_names(Some("public"), "orders")
        );
        assert_eq!(
            TableRef::try_from("public.orders").unwrap(),
            TableRef::from_names(Some("public"), "orders")
        );
        assert_eq!(
            TableRef::from_str("orders").unwrap(),
            TableRef::from_names(None, "orders")
        );
    }

    #[test]
    fn rejects_table_references_with_too_many_components() {
        assert_eq!(
            TableRef::from_strs(&["catalog", "schema", "orders"]).unwrap_err(),
            ParseError::InvalidTableReference {
                table_reference: "catalog,schema,orders".to_string()
            }
        );
        assert_eq!(
            TableRef::try_from("catalog.schema.orders").unwrap_err(),
            ParseError::InvalidTableReference {
                table_reference: "catalog.schema.orders".to_string()
            }
        );
    }

    #[test]
    fn serializes_as_display_string_and_deserializes_back() {
        let table = TableRef::from_names(Some("public"), "orders");
        let json = serde_json::to_string(&table).unwrap();
        assert_eq!(json, r#""public.orders""#);
        assert_eq!(serde_json::from_str::<TableRef>(&json).unwrap(), table);

        let invalid = serde_json::from_str::<TableRef>(r#""a.b.c""#).unwrap_err();
        assert!(invalid.to_string().contains("Invalid table reference"));
    }

    #[test]
    fn borrowed_table_refs_are_equivalent_to_matching_keys() {
        let key = TableRef::from_names(Some("public"), "orders");
        let same = TableRef::from_names(Some("public"), "orders");
        let different_schema = TableRef::from_names(Some("private"), "orders");
        let different_table = TableRef::from_names(Some("public"), "lineitem");

        assert!(Equivalent::<TableRef>::equivalent(&&same, &key));
        assert!(!Equivalent::<TableRef>::equivalent(
            &&different_schema,
            &key
        ));
        assert!(!Equivalent::<TableRef>::equivalent(&&different_table, &key));
    }

    #[test]
    fn parse_error_display_includes_bad_reference() {
        let error = ParseError::InvalidTableReference {
            table_reference: "a.b.c".to_string(),
        };
        assert_eq!(error.to_string(), "Invalid table reference: a.b.c");
    }
}
