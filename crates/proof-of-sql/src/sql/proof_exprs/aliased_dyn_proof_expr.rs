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
    fn we_can_clone_and_compare_aliased_dyn_proof_exprs() {
        let aliased_expr = AliasedDynProofExpr {
            expr: DynProofExpr::new_literal(LiteralValue::Int(42)),
            alias: "answer".into(),
        };

        assert_eq!(aliased_expr.clone(), aliased_expr);
        assert!(format!("{aliased_expr:?}").contains("answer"));
        assert_eq!(aliased_expr.alias.value, "answer");
        assert_eq!(
            aliased_expr.expr,
            DynProofExpr::new_literal(LiteralValue::Int(42))
        );
    }

    #[test]
    fn we_can_serialize_and_deserialize_aliased_dyn_proof_exprs() {
        let aliased_expr = AliasedDynProofExpr {
            expr: DynProofExpr::new_literal(LiteralValue::VarChar("north".into())),
            alias: "direction".into(),
        };

        let json = serde_json::to_string(&aliased_expr).unwrap();
        let decoded: AliasedDynProofExpr = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded, aliased_expr);
    }
}
