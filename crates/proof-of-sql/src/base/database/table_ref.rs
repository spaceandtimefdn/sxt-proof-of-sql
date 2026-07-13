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
    use alloc::vec;

    #[test]
    fn table_ref_constructors_and_accessors_handle_schema_variants() {
        let unqualified = TableRef::new("", "transactions");
        assert_eq!(unqualified.schema_id(), None);
        assert_eq!(unqualified.table_id().value, "transactions");
        assert_eq!(unqualified.to_string(), "transactions");

        let qualified = TableRef::from_names(Some("sxt"), "blocks");
        assert_eq!(
            qualified.schema_id().map(|schema| schema.value.as_str()),
            Some("sxt")
        );
        assert_eq!(qualified.table_id().value, "blocks");
        assert_eq!(qualified.to_string(), "sxt.blocks");

        let from_idents =
            TableRef::from_idents(Some(Ident::new("analytics")), Ident::new("events"));
        assert_eq!(
            from_idents.schema_id().map(|schema| schema.value.as_str()),
            Some("analytics")
        );
        assert_eq!(from_idents.table_id().value, "events");
        assert_eq!(from_idents.to_string(), "analytics.events");
    }

    #[test]
    fn table_ref_from_strs_accepts_one_or_two_components() {
        assert_eq!(
            TableRef::from_strs(&["orders"]).unwrap(),
            TableRef::from_names(None, "orders")
        );
        assert_eq!(
            TableRef::from_strs(&["warehouse", "inventory"]).unwrap(),
            TableRef::from_names(Some("warehouse"), "inventory")
        );
    }

    #[test]
    fn table_ref_from_strs_reports_invalid_component_count() {
        let err = TableRef::from_strs(&["too", "many", "parts"]).unwrap_err();
        assert_eq!(
            err,
            ParseError::InvalidTableReference {
                table_reference: "too,many,parts".into()
            }
        );
    }

    #[test]
    fn table_ref_try_from_and_serde_reject_nested_paths() {
        assert_eq!(
            TableRef::try_from("schema.table").unwrap(),
            TableRef::from_names(Some("schema"), "table")
        );

        let err = TableRef::try_from("catalog.schema.table").unwrap_err();
        assert_eq!(
            err,
            ParseError::InvalidTableReference {
                table_reference: "catalog.schema.table".into()
            }
        );

        let serialized = serde_json::to_string(&TableRef::new("public", "receipts")).unwrap();
        assert_eq!(serialized, "\"public.receipts\"");
        let round_trip: TableRef = serde_json::from_str(&serialized).unwrap();
        assert_eq!(round_trip, TableRef::new("public", "receipts"));

        let serde_err = serde_json::from_value::<TableRef>(serde_json::json!("a.b.c")).unwrap_err();
        assert!(serde_err
            .to_string()
            .contains("Invalid table reference: a.b.c"));
    }

    #[test]
    fn borrowed_table_ref_equivalence_matches_full_key() {
        let key = TableRef::new("sxt", "balances");
        assert!((&key).equivalent(&TableRef::new("sxt", "balances")));
        assert!(!(&key).equivalent(&TableRef::new("other", "balances")));
        assert!(!(&key).equivalent(&TableRef::new("sxt", "transfers")));

        let refs = vec![
            TableRef::new("sxt", "balances"),
            TableRef::new("sxt", "transfers"),
        ];
        assert_eq!(
            refs.iter()
                .filter(|candidate| (&key).equivalent(candidate))
                .count(),
            1
        );
    }
}
