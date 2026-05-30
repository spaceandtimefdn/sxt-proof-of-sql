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
    use crate::base::database::error::ParseError;
    use indexmap::Equivalent;
    use sqlparser::ast::Ident;

    #[test]
    fn we_can_build_table_refs_from_names_and_idents() {
        let bare = TableRef::new("", "orders");
        assert_eq!(bare.schema_id(), None);
        assert_eq!(bare.table_id().value, "orders");
        assert_eq!(bare.to_string(), "orders");

        let qualified = TableRef::from_names(Some("sales"), "orders");
        assert_eq!(qualified.schema_id().unwrap().value, "sales");
        assert_eq!(qualified.table_id().value, "orders");
        assert_eq!(qualified.to_string(), "sales.orders");

        let from_idents =
            TableRef::from_idents(Some(Ident::new("analytics")), Ident::new("events"));
        assert_eq!(from_idents.to_string(), "analytics.events");
    }

    #[test]
    fn we_can_parse_table_refs_from_components_and_strings() {
        let bare = TableRef::from_strs(&["orders"]).unwrap();
        assert_eq!(bare.to_string(), "orders");

        let parsed_qualified = TableRef::from_strs(&["sales", "orders"]).unwrap();
        assert_eq!(parsed_qualified.to_string(), "sales.orders");

        assert_eq!(
            TableRef::try_from("public.users").unwrap(),
            qualified("public", "users")
        );
        assert_eq!(
            "metrics".parse::<TableRef>().unwrap().to_string(),
            "metrics"
        );
    }

    #[test]
    fn invalid_table_refs_report_the_original_input() {
        let empty_components: [&str; 0] = [];
        assert!(matches!(
            TableRef::from_strs(&empty_components),
            Err(ParseError::InvalidTableReference {
                table_reference
            }) if table_reference.is_empty()
        ));

        assert!(matches!(
            TableRef::from_strs(&["a", "b", "c"]),
            Err(ParseError::InvalidTableReference {
                table_reference
            }) if table_reference == "a,b,c"
        ));

        assert!(matches!(
            TableRef::try_from("a.b.c"),
            Err(ParseError::InvalidTableReference {
                table_reference
            }) if table_reference == "a.b.c"
        ));
    }

    #[test]
    fn table_ref_serializes_as_display_string_and_deserializes_back() {
        let table = TableRef::from_names(Some("warehouse"), "shipments");

        let json = serde_json::to_string(&table).unwrap();
        assert_eq!(json, r#""warehouse.shipments""#);

        let round_trip: TableRef = serde_json::from_str(&json).unwrap();
        assert_eq!(round_trip, table);

        assert!(serde_json::from_str::<TableRef>(r#""too.many.parts""#).is_err());
    }

    #[test]
    fn borrowed_table_refs_compare_as_equivalent_keys() {
        let table = TableRef::from_names(Some("public"), "orders");
        let equivalent = TableRef::from_names(Some("public"), "orders");
        let different = TableRef::from_names(Some("private"), "orders");

        assert!((&equivalent).equivalent(&table));
        assert!(!(&different).equivalent(&table));
    }

    fn qualified(schema: &str, table: &str) -> TableRef {
        TableRef::from_names(Some(schema), table)
    }
}
