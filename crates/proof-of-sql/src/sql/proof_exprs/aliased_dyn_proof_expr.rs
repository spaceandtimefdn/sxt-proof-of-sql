use super::DynProofExpr;
use crate::base::database::ColumnId;
use serde::{Deserialize, Serialize};

/// A `DynProofExpr` with an alias.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AliasedDynProofExpr {
    /// The `DynProofExpr` to alias.
    pub expr: DynProofExpr,
    /// The alias for the expression.
    pub alias: ColumnId,
}
