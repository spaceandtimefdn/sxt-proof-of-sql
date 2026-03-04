use crate::base::database::TableRef;
use alloc::string::String;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// column identifier with qualifier
#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct ColumnId {
    name: Ident,
    qualifier: Option<TableRef>,
}

impl ColumnId {
    /// Create new `ColumnId`
    #[must_use]
    pub fn new(name: Ident, qualifier: Option<TableRef>) -> Self {
        Self { name, qualifier }
    }

    /// Get the name of the column
    #[must_use]
    pub fn name(&self) -> &Ident {
        &self.name
    }

    /// Get the qualfier of the column
    #[must_use]
    pub fn qualifier(&self) -> &Option<TableRef> {
        &self.qualifier
    }
}

impl From<Ident> for ColumnId {
    fn from(value: Ident) -> Self {
        ColumnId {
            name: value,
            qualifier: None,
        }
    }
}

impl From<&Ident> for ColumnId {
    fn from(value: &Ident) -> Self {
        value.clone().into()
    }
}

impl From<&str> for ColumnId {
    fn from(value: &str) -> Self {
        value.rsplit_once('.').map_or_else(
            || ColumnId {
                name: value.into(),
                qualifier: None,
            },
            |(table, column)| ColumnId {
                name: column.into(),
                qualifier: Some(TableRef::try_from(table).expect("Invalid column id")),
            },
        )
    }
}

impl From<&String> for ColumnId {
    fn from(value: &String) -> Self {
        ColumnId {
            name: Ident::new(value),
            qualifier: None,
        }
    }
}
