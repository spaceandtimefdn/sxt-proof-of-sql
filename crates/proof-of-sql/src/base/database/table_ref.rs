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
    use crate::base::IndexMap;
    use alloc::vec;

    #[test]
    fn new_treats_empty_schema_as_unqualified_table() {
        let table_ref = TableRef::new("", "blocks");

        assert!(table_ref.schema_id().is_none());
        assert_eq!(table_ref.table_id().value, "blocks");
        assert_eq!(table_ref.to_string(), "blocks");
    }

    #[test]
    fn from_names_preserves_schema_and_table() {
        let table_ref = TableRef::from_names(Some("analytics"), "blocks");

        assert_eq!(table_ref.schema_id().unwrap().value, "analytics");
        assert_eq!(table_ref.table_id().value, "blocks");
        assert_eq!(table_ref.to_string(), "analytics.blocks");
    }

    #[test]
    fn from_idents_preserves_identifier_values() {
        let table_ref = TableRef::from_idents(Some("sxt".into()), "proofs".into());

        assert_eq!(table_ref.schema_id().unwrap().value, "sxt");
        assert_eq!(table_ref.table_id().value, "proofs");
    }

    #[test]
    fn from_strs_accepts_one_or_two_components() {
        let unqualified = TableRef::from_strs(&["blocks"]).unwrap();
        let qualified = TableRef::from_strs(&["analytics", "blocks"]).unwrap();

        assert_eq!(unqualified.to_string(), "blocks");
        assert_eq!(qualified.to_string(), "analytics.blocks");
    }

    #[test]
    fn from_strs_rejects_invalid_component_counts() {
        let empty = TableRef::from_strs::<&str>(&[]).unwrap_err();
        let too_many = TableRef::from_strs(&["a", "b", "c"]).unwrap_err();

        assert_eq!(
            empty,
            ParseError::InvalidTableReference {
                table_reference: "".into(),
            }
        );
        assert_eq!(
            too_many,
            ParseError::InvalidTableReference {
                table_reference: "a,b,c".into(),
            }
        );
    }

    #[test]
    fn try_from_dot_separated_string_matches_from_str() {
        let expected = TableRef::from_names(Some("analytics"), "blocks");

        assert_eq!(TableRef::try_from("analytics.blocks").unwrap(), expected);
        assert_eq!("analytics.blocks".parse::<TableRef>().unwrap(), expected);
    }

    #[test]
    fn try_from_rejects_more_than_two_dot_components() {
        assert_eq!(
            TableRef::try_from("a.b.c").unwrap_err(),
            ParseError::InvalidTableReference {
                table_reference: "a.b.c".into(),
            }
        );
    }

    #[test]
    fn table_ref_serializes_as_display_string() {
        let table_ref = TableRef::from_names(Some("analytics"), "blocks");

        assert_eq!(
            serde_json::to_string(&table_ref).unwrap(),
            r#""analytics.blocks""#
        );
        assert_eq!(
            serde_json::from_str::<TableRef>(r#""analytics.blocks""#).unwrap(),
            table_ref
        );
    }

    #[test]
    fn borrowed_table_ref_can_lookup_index_map_entries() {
        let table_ref = TableRef::from_names(Some("analytics"), "blocks");
        let mut map: IndexMap<TableRef, i32> = IndexMap::with_hasher(Default::default());
        map.insert(table_ref.clone(), 7);

        assert_eq!(map.get(&&table_ref), Some(&7));
    }

    #[test]
    fn table_refs_keep_insertion_order_in_index_map() {
        let mut map: IndexMap<TableRef, i32> = IndexMap::with_hasher(Default::default());
        map.insert(TableRef::from_names(None, "blocks"), 1);
        map.insert(TableRef::from_names(Some("analytics"), "proofs"), 2);

        assert_eq!(
            map.keys()
                .map(ToString::to_string)
                .collect::<alloc::vec::Vec<_>>(),
            vec!["blocks", "analytics.proofs"]
        );
    }
}
