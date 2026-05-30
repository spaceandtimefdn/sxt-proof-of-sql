use super::DynProofExpr;
use crate::base::database::{ColumnField, ColumnRef, LiteralValue};
use alloc::vec::Vec;
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

    /// Return the physical value-plus-presence expressions for this aliased result.
    #[must_use]
    pub(crate) fn physical_result_exprs(&self) -> Vec<Self> {
        let mut exprs = Vec::with_capacity(2);
        exprs.push(self.clone());

        if self.result_field().is_nullable() {
            exprs.push(Self {
                expr: self
                    .expr
                    .nullable_result_presence_expr()
                    .unwrap_or_else(|| DynProofExpr::new_literal(LiteralValue::Boolean(true))),
                alias: ColumnRef::presence_column_id(&self.alias),
            });
        }

        exprs
    }
}
