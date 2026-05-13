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
    use proptest::prelude::*;
    use serde_json;

    #[test]
    fn from_strs_handles_schema_variants() {
        let unqualified = TableRef::from_strs(&["orders"]).unwrap();
        assert_eq!(unqualified.schema_id(), None);
        assert_eq!(unqualified.table_id().value, "orders");
        assert_eq!(unqualified.to_string(), "orders");

        let qualified = TableRef::from_strs(&["analytics", "orders"]).unwrap();
        assert_eq!(qualified.schema_id().unwrap().value, "analytics");
        assert_eq!(qualified.table_id().value, "orders");
        assert_eq!(qualified.to_string(), "analytics.orders");
    }

    #[test]
    fn from_strs_rejects_invalid_component_counts() {
        let empty = TableRef::from_strs::<&str>(&[]).unwrap_err();
        assert_eq!(
            empty,
            ParseError::InvalidTableReference {
                table_reference: "".to_string()
            }
        );

        let too_many = TableRef::from_strs(&["a", "b", "c"]).unwrap_err();
        assert_eq!(
            too_many,
            ParseError::InvalidTableReference {
                table_reference: "a,b,c".to_string()
            }
        );
    }

    #[test]
    fn serde_round_trips_table_refs_as_strings() {
        let table_ref = TableRef::new("analytics", "orders");
        let serialized = serde_json::to_string(&table_ref).unwrap();
        assert_eq!(serialized, "\"analytics.orders\"");

        let deserialized: TableRef = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, table_ref);
    }

    proptest! {
        #[test]
        fn prop_unqualified_table_refs_roundtrip(table in "[A-Za-z_][A-Za-z0-9_]{0,31}") {
            let table_ref = TableRef::try_from(table.as_str()).unwrap();

            prop_assert_eq!(table_ref.schema_id(), None);
            prop_assert_eq!(table_ref.table_id().value.as_str(), table.as_str());
            prop_assert_eq!(TableRef::try_from(table_ref.to_string().as_str()).unwrap(), table_ref);
        }

        #[test]
        fn prop_qualified_table_refs_roundtrip(
            schema in "[A-Za-z_][A-Za-z0-9_]{0,31}",
            table in "[A-Za-z_][A-Za-z0-9_]{0,31}",
        ) {
            let input = alloc::format!("{schema}.{table}");
            let table_ref = TableRef::try_from(input.as_str()).unwrap();

            prop_assert_eq!(table_ref.schema_id().unwrap().value.as_str(), schema.as_str());
            prop_assert_eq!(table_ref.table_id().value.as_str(), table.as_str());
            prop_assert_eq!(TableRef::try_from(table_ref.to_string().as_str()).unwrap(), table_ref);
        }

        #[test]
        fn prop_dot_separated_refs_with_too_many_components_are_rejected(
            a in "[A-Za-z_][A-Za-z0-9_]{0,8}",
            b in "[A-Za-z_][A-Za-z0-9_]{0,8}",
            c in "[A-Za-z_][A-Za-z0-9_]{0,8}",
        ) {
            let input = alloc::format!("{a}.{b}.{c}");
            let err = TableRef::try_from(input.as_str()).unwrap_err();

            prop_assert_eq!(
                err,
                ParseError::InvalidTableReference {
                    table_reference: input
                }
            );
        }
    }
}
