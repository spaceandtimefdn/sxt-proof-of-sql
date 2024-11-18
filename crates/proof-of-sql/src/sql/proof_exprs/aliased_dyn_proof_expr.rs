use super::DynProofExpr;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident as Identifier;

/// A `DynProofExpr` with an alias.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AliasedDynProofExpr {
    pub expr: DynProofExpr,
    pub alias: Identifier,
}
