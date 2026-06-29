use super::DynProofExpr;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::database::LiteralValue;

    #[test]
    fn aliased_dyn_proof_expr_round_trips_through_json() {
        let expr = AliasedDynProofExpr {
            expr: DynProofExpr::new_literal(LiteralValue::BigInt(42)),
            alias: Ident::new("answer"),
        };

        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: AliasedDynProofExpr = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, expr);
    }

    #[test]
    fn aliased_dyn_proof_expr_clone_preserves_expr_and_alias() {
        let expr = AliasedDynProofExpr {
            expr: DynProofExpr::new_literal(LiteralValue::Boolean(true)),
            alias: Ident::new("is_selected"),
        };

        let cloned = expr.clone();

        assert_eq!(cloned.expr, expr.expr);
        assert_eq!(cloned.alias, expr.alias);
    }
}
