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
    use sqlparser::ast::Ident;

    #[test]
    fn we_can_build_table_refs_from_names_and_idents() {
        let without_schema = TableRef::new("", "employees");
        assert_eq!(without_schema.schema_id(), None);
        assert_eq!(without_schema.table_id(), &Ident::new("employees"));
        assert_eq!(without_schema.to_string(), "employees");

        let with_schema = TableRef::new("public", "employees");
        assert_eq!(with_schema.schema_id(), Some(&Ident::new("public")));
        assert_eq!(with_schema.table_id(), &Ident::new("employees"));
        assert_eq!(with_schema.to_string(), "public.employees");

        let from_names = TableRef::from_names(Some("sales"), "orders");
        assert_eq!(from_names.schema_id(), Some(&Ident::new("sales")));
        assert_eq!(from_names.table_id(), &Ident::new("orders"));

        let from_idents = TableRef::from_idents(Some(Ident::new("hr")), Ident::new("people"));
        assert_eq!(from_idents.schema_id(), Some(&Ident::new("hr")));
        assert_eq!(from_idents.table_id(), &Ident::new("people"));
    }

    #[test]
    fn we_can_parse_table_refs_from_slices_and_strings() {
        let single = TableRef::from_strs(&["employees"]).unwrap();
        assert_eq!(single.schema_id(), None);
        assert_eq!(single.table_id(), &Ident::new("employees"));

        let qualified = TableRef::from_strs(&["public", "employees"]).unwrap();
        assert_eq!(qualified.schema_id(), Some(&Ident::new("public")));
        assert_eq!(qualified.table_id(), &Ident::new("employees"));

        let from_try_from = TableRef::try_from("analytics.events").unwrap();
        assert_eq!(from_try_from.schema_id(), Some(&Ident::new("analytics")));
        assert_eq!(from_try_from.table_id(), &Ident::new("events"));

        let from_str = "events".parse::<TableRef>().unwrap();
        assert_eq!(from_str.schema_id(), None);
        assert_eq!(from_str.table_id(), &Ident::new("events"));
    }

    #[test]
    fn we_reject_invalid_table_ref_shapes() {
        let err = TableRef::from_strs(&["a", "b", "c"]).unwrap_err();
        assert_eq!(
            err,
            ParseError::InvalidTableReference {
                table_reference: "a,b,c".to_string(),
            }
        );

        let err = TableRef::try_from("a.b.c").unwrap_err();
        assert_eq!(
            err,
            ParseError::InvalidTableReference {
                table_reference: "a.b.c".to_string(),
            }
        );
    }

    #[test]
    fn we_support_equivalence_and_serde_round_trips() {
        let expected = TableRef::new("public", "employees");
        let borrowed = &expected;
        let parsed = TableRef::try_from("public.employees").unwrap();

        assert!(borrowed.equivalent(&parsed));

        let json = serde_json::to_string(&expected).unwrap();
        assert_eq!(json, "\"public.employees\"");
        let round_trip: TableRef = serde_json::from_str(&json).unwrap();
        assert_eq!(round_trip, expected);
    }
}
