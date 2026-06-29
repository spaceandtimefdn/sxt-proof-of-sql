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
    use indexmap::Equivalent;

    #[test]
    fn new_omits_empty_schema_name() {
        let table_ref = TableRef::new("", "orders");

        assert_eq!(table_ref.schema_id(), None);
        assert_eq!(table_ref.table_id().value, "orders");
        assert_eq!(table_ref.to_string(), "orders");
    }

    #[test]
    fn constructors_preserve_schema_and_table_names() {
        let from_new = TableRef::new("analytics", "events");
        let from_names = TableRef::from_names(Some("analytics"), "events");
        let from_idents =
            TableRef::from_idents(Some(Ident::new("analytics")), Ident::new("events"));

        assert_eq!(from_new, from_names);
        assert_eq!(from_names, from_idents);
        assert_eq!(from_idents.schema_id().unwrap().value, "analytics");
        assert_eq!(from_idents.table_id().value, "events");
        assert_eq!(from_idents.to_string(), "analytics.events");
    }

    #[test]
    fn from_strs_accepts_one_or_two_components() {
        let table_only = TableRef::from_strs(&["orders"]).unwrap();
        assert_eq!(table_only, TableRef::from_names(None, "orders"));

        let schema_table = TableRef::from_strs(&["analytics", "events"]).unwrap();
        assert_eq!(
            schema_table,
            TableRef::from_names(Some("analytics"), "events")
        );
    }

    #[test]
    fn from_strs_rejects_invalid_component_counts() {
        let empty_components: [&str; 0] = [];
        let empty_error = TableRef::from_strs(&empty_components).unwrap_err();
        assert!(matches!(
            empty_error,
            ParseError::InvalidTableReference { .. }
        ));

        let too_many_error = TableRef::from_strs(&["a", "b", "c"]).unwrap_err();
        assert!(matches!(
            too_many_error,
            ParseError::InvalidTableReference { table_reference }
                if table_reference == "a,b,c"
        ));
    }

    #[test]
    fn dot_separated_parsing_matches_from_str() {
        let parsed = TableRef::try_from("analytics.events").unwrap();
        let from_str = "analytics.events".parse::<TableRef>().unwrap();

        assert_eq!(parsed, TableRef::new("analytics", "events"));
        assert_eq!(parsed, from_str);
    }

    #[test]
    fn dot_separated_parsing_rejects_too_many_components() {
        let error = TableRef::try_from("catalog.analytics.events").unwrap_err();

        assert!(matches!(
            error,
            ParseError::InvalidTableReference { table_reference }
                if table_reference == "catalog.analytics.events"
        ));
    }

    #[test]
    fn equivalent_matches_identical_table_refs_only() {
        let table_ref = TableRef::new("analytics", "events");
        let same = TableRef::new("analytics", "events");
        let different_schema = TableRef::new("warehouse", "events");
        let different_table = TableRef::new("analytics", "orders");

        assert!((&table_ref).equivalent(&same));
        assert!(!(&table_ref).equivalent(&different_schema));
        assert!(!(&table_ref).equivalent(&different_table));
    }

    #[test]
    fn serde_round_trips_table_refs_as_strings() {
        let table_ref = TableRef::new("analytics", "events");

        let encoded = serde_json::to_string(&table_ref).unwrap();
        assert_eq!(encoded, "\"analytics.events\"");

        let decoded: TableRef = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, table_ref);
    }

    #[test]
    fn serde_rejects_invalid_table_ref_strings() {
        let error = serde_json::from_str::<TableRef>("\"a.b.c\"").unwrap_err();

        assert!(error.to_string().contains("Invalid table reference: a.b.c"));
    }
}
