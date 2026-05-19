use super::DynProofExpr;
use crate::base::database::ColumnField;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// A `DynProofExpr` with an alias.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AliasedDynProofExpr {
    /// The `DynProofExpr` to alias.
    pub expr: DynProofExpr,
    /// The alias for the expression.
    pub alias: Ident,
}

impl AliasedDynProofExpr {
    /// Return the result field exposed by this aliased expression.
    #[must_use]
    pub(crate) fn result_field(&self) -> ColumnField {
        self.expr
            .nullable_propagating_result_field(self.alias.clone())
    }
}
