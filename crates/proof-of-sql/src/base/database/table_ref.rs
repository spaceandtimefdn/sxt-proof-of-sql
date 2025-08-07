use crate::base::database::error::ParseError;
use alloc::{string::ToString, vec::Vec};
use core::{
    fmt,
    fmt::{Display, Formatter},
    str::FromStr,
};
use indexmap::Equivalent;
use proof_of_sql_parser::{impl_serde_from_str, ResourceId};
use sqlparser::ast::Ident;

/// Expression for an SQL table
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TableRef {
    /// Fully qualified table reference with schema and table name
    FullyQualified {
        /// Schema name
        schema_name: Ident,
        /// Table name
        table_name: Ident,
    },
    /// Table reference without schema
    TableOnly {
        /// Table name
        table_name: Ident,
    },
    /// No table reference
    None,
}

impl TableRef {
    /// Creates a new table reference from schema and table names.
    /// If the schema name is empty or None, only the table name is used.
    #[must_use]
    pub fn new(schema_name: impl AsRef<str>, table_name: impl AsRef<str>) -> Self {
        let schema = schema_name.as_ref();
        let table = table_name.as_ref();

        if schema.is_empty() {
            Self::TableOnly {
                table_name: Ident::new(table.to_string()),
            }
        } else {
            Self::FullyQualified {
                schema_name: Ident::new(schema.to_string()),
                table_name: Ident::new(table.to_string()),
            }
        }
    }

    /// Returns the identifier of the schema if it exists. Otherwise returns `None`.
    #[must_use]
    pub fn schema_id(&self) -> Option<&Ident> {
        match self {
            Self::FullyQualified { schema_name, .. } => Some(schema_name),
            Self::TableOnly { .. } | Self::None => None,
        }
    }

    /// Returns the identifier of the table if it exists. Otherwise returns `None`.
    #[must_use]
    pub fn table_id(&self) -> Option<&Ident> {
        match self {
            Self::FullyQualified { table_name, .. } | Self::TableOnly { table_name } => {
                Some(table_name)
            }
            Self::None => None,
        }
    }

    /// Creates a new table reference from an optional schema and table name.
    #[must_use]
    pub fn from_names(schema_name: Option<&str>, table_name: &str) -> Self {
        if let Some(schema) = schema_name {
            Self::FullyQualified {
                schema_name: Ident::new(schema.to_string()),
                table_name: Ident::new(table_name.to_string()),
            }
        } else {
            Self::TableOnly {
                table_name: Ident::new(table_name.to_string()),
            }
        }
    }

    /// Creates a `TableRef` directly from `Option<Ident>` for schema and `Ident` for table.
    #[must_use]
    pub fn from_idents(schema_name: Option<Ident>, table_name: Ident) -> Self {
        match schema_name {
            Some(schema) => Self::FullyQualified {
                schema_name: schema,
                table_name,
            },
            None => Self::TableOnly { table_name },
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

/// Note: We just need this conversion trait until `SelectStatement` refactor is done
impl From<ResourceId> for TableRef {
    fn from(id: ResourceId) -> Self {
        Self::FullyQualified {
            schema_name: Ident::from(id.schema()),
            table_name: Ident::from(id.object_name()),
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
        match (self, key) {
            (
                TableRef::FullyQualified {
                    schema_name: s1,
                    table_name: t1,
                },
                TableRef::FullyQualified {
                    schema_name: s2,
                    table_name: t2,
                },
            ) => s1 == s2 && t1 == t2,
            (TableRef::TableOnly { table_name: t1 }, TableRef::TableOnly { table_name: t2 }) => {
                t1 == t2
            }
            (TableRef::None, TableRef::None) => true,
            _ => false,
        }
    }
}

impl Display for TableRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TableRef::FullyQualified {
                schema_name,
                table_name,
            } => write!(f, "{}.{}", schema_name.value, table_name.value),
            TableRef::TableOnly { table_name } => write!(f, "{}", table_name.value),
            TableRef::None => write!(f, "<no_table>"),
        }
    }
}

impl_serde_from_str!(TableRef);
