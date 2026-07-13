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
    use alloc::{format, vec};
    use indexmap::Equivalent;

    #[test]
    fn table_refs_preserve_schema_and_table_identifiers() {
        let with_schema = TableRef::new("sxt", "blocks");
        assert_eq!(with_schema.schema_id().unwrap().value, "sxt");
        assert_eq!(with_schema.table_id().value, "blocks");
        assert_eq!(with_schema.to_string(), "sxt.blocks");

        let without_schema = TableRef::new("", "transactions");
        assert!(without_schema.schema_id().is_none());
        assert_eq!(without_schema.table_id().value, "transactions");
        assert_eq!(format!("{without_schema}"), "transactions");
    }

    #[test]
    fn table_refs_parse_components_and_dot_separated_names() {
        assert_eq!(
            TableRef::from_names(Some("chain"), "events"),
            TableRef::from_strs(&["chain", "events"]).unwrap()
        );
        assert_eq!(
            TableRef::from_names(None, "events"),
            TableRef::from_strs(&["events"]).unwrap()
        );
        assert_eq!(
            TableRef::from_names(Some("chain"), "events"),
            TableRef::try_from("chain.events").unwrap()
        );
        assert_eq!(
            TableRef::from_names(None, "events"),
            "events".parse::<TableRef>().unwrap()
        );
    }

    #[test]
    fn table_refs_reject_invalid_component_counts() {
        assert_eq!(
            TableRef::from_strs::<&str>(&[]).unwrap_err(),
            ParseError::InvalidTableReference {
                table_reference: "".to_string()
            }
        );
        assert_eq!(
            TableRef::from_strs(&["too", "many", "parts"]).unwrap_err(),
            ParseError::InvalidTableReference {
                table_reference: "too,many,parts".to_string()
            }
        );
        assert_eq!(
            TableRef::try_from("too.many.parts").unwrap_err(),
            ParseError::InvalidTableReference {
                table_reference: "too.many.parts".to_string()
            }
        );
    }

    #[test]
    fn table_refs_support_ident_construction_equivalence_and_serde() {
        let table = TableRef::from_idents(Some(Ident::new("public")), Ident::new("balances"));
        let same = TableRef::new("public", "balances");
        let different_schema = TableRef::new("archive", "balances");
        let different_table = TableRef::new("public", "transfers");

        assert!((&table).equivalent(&same));
        assert!(!(&table).equivalent(&different_schema));
        assert!(!(&table).equivalent(&different_table));

        let encoded = serde_json::to_string(&table).unwrap();
        assert_eq!(encoded, "\"public.balances\"");
        let decoded: TableRef = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, table);

        let invalid = serde_json::from_str::<TableRef>("\"a.b.c\"").unwrap_err();
        assert!(invalid.to_string().contains("Invalid table reference"));

        let refs = vec![table, same];
        assert_eq!(refs[0], refs[1]);
    }
}
