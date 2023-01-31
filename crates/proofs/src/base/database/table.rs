use std::str::FromStr;

use proofs_sql::{Identifier, ResourceId};

/// Expression for an SQL table
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TableRef {
    resource_id: ResourceId,
}

impl TableRef {
    pub fn new(resource_id: ResourceId) -> Self {
        Self { resource_id }
    }

    pub fn schema_id(&self) -> &Identifier {
        self.resource_id.schema()
    }

    pub fn table_id(&self) -> &Identifier {
        self.resource_id.object_name()
    }
}

impl FromStr for TableRef {
    type Err = proofs_sql::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s.parse()?))
    }
}
