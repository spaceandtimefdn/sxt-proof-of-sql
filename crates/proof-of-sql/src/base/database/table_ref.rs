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
    use crate::base::database::ParseError;
    use alloc::{format, vec};
    use sqlparser::ast::Ident;

    #[test]
    fn constructors_preserve_schema_and_table_identifiers() {
        let unqualified = TableRef::new("", "orders");
        assert!(unqualified.schema_id().is_none());
        assert_eq!(unqualified.table_id().value, "orders");
        assert_eq!(unqualified.to_string(), "orders");

        let qualified = TableRef::new("sxt", "orders");
        assert_eq!(qualified.schema_id().unwrap().value, "sxt");
        assert_eq!(qualified.table_id().value, "orders");
        assert_eq!(qualified.to_string(), "sxt.orders");

        let from_idents =
            TableRef::from_idents(Some(Ident::new("analytics")), Ident::new("events"));
        assert_eq!(from_idents.schema_id().unwrap().value, "analytics");
        assert_eq!(from_idents.table_id().value, "events");
        assert_eq!(from_idents.to_string(), "analytics.events");
    }

    #[test]
    fn parses_one_and_two_part_table_references() {
        let table_only = TableRef::try_from("orders").unwrap();
        assert!(table_only.schema_id().is_none());
        assert_eq!(table_only.table_id().value, "orders");

        let schema_and_table: TableRef = "sxt.orders".parse().unwrap();
        assert_eq!(schema_and_table.schema_id().unwrap().value, "sxt");
        assert_eq!(schema_and_table.table_id().value, "orders");

        assert_eq!(
            TableRef::from_strs(&["sxt", "orders"]).unwrap(),
            schema_and_table
        );
        assert_eq!(TableRef::from_strs(&["orders"]).unwrap(), table_only);
    }

    #[test]
    fn rejects_table_references_with_too_many_components() {
        let from_str_error = TableRef::try_from("catalog.sxt.orders").unwrap_err();
        assert!(matches!(
            from_str_error,
            ParseError::InvalidTableReference { .. }
        ));
        assert_eq!(
            format!("{from_str_error}"),
            "Invalid table reference: catalog.sxt.orders"
        );

        let from_slice_error = TableRef::from_strs(&vec!["catalog", "sxt", "orders"]).unwrap_err();
        assert!(matches!(
            from_slice_error,
            ParseError::InvalidTableReference { .. }
        ));
        assert_eq!(
            format!("{from_slice_error}"),
            "Invalid table reference: catalog,sxt,orders"
        );
    }

    #[test]
    fn serializes_and_deserializes_as_display_string() {
        let table_ref = TableRef::new("sxt", "orders");

        let serialized = serde_json::to_string(&table_ref).unwrap();
        assert_eq!(serialized, "\"sxt.orders\"");

        let deserialized: TableRef = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, table_ref);

        let invalid = serde_json::from_str::<TableRef>("\"catalog.sxt.orders\"");
        assert!(invalid.is_err());
    }
}
