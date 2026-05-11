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
    use indexmap::Equivalent;
    use sqlparser::ast::Ident;

    #[test]
    fn table_ref_constructors_and_display_cover_schema_variants() {
        let table_ref = TableRef::new("sxt", "blocks");
        assert_eq!(table_ref.schema_id().unwrap().value, "sxt");
        assert_eq!(table_ref.table_id().value, "blocks");
        assert_eq!(table_ref.to_string(), "sxt.blocks");

        let table_ref = TableRef::new("", "transactions");
        assert!(table_ref.schema_id().is_none());
        assert_eq!(table_ref.table_id().value, "transactions");
        assert_eq!(table_ref.to_string(), "transactions");

        let from_names = TableRef::from_names(Some("public"), "balances");
        let from_idents = TableRef::from_idents(Some(Ident::new("public")), Ident::new("balances"));
        assert_eq!(from_names, from_idents);
        assert!((&from_names).equivalent(&from_idents));
    }

    #[test]
    fn table_ref_parsing_accepts_one_or_two_components() {
        assert_eq!(
            TableRef::from_strs(&["orders"]).unwrap(),
            TableRef::from_names(None, "orders")
        );
        assert_eq!(
            TableRef::from_strs(&["sales", "orders"]).unwrap(),
            TableRef::from_names(Some("sales"), "orders")
        );
        assert_eq!(
            TableRef::try_from("sales.orders").unwrap(),
            TableRef::from_names(Some("sales"), "orders")
        );
        assert_eq!(
            "orders".parse::<TableRef>().unwrap(),
            TableRef::from_names(None, "orders")
        );
    }

    #[test]
    fn table_ref_parsing_rejects_more_than_two_components() {
        assert!(matches!(
            TableRef::from_strs(&["too", "many", "parts"]),
            Err(ParseError::InvalidTableReference { table_reference })
                if table_reference == "too,many,parts"
        ));

        assert!(matches!(
            TableRef::try_from("too.many.parts"),
            Err(ParseError::InvalidTableReference { table_reference })
                if table_reference == "too.many.parts"
        ));
    }

    #[test]
    fn table_ref_serializes_as_its_display_form() {
        let table_ref = TableRef::from_names(Some("sxt"), "blocks");

        let serialized = serde_json::to_string(&table_ref).unwrap();
        assert_eq!(serialized, r#""sxt.blocks""#);

        let deserialized: TableRef = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, table_ref);

        let error = serde_json::from_str::<TableRef>(r#""too.many.parts""#).unwrap_err();
        assert!(error.to_string().contains("Invalid table reference"));
    }
}
