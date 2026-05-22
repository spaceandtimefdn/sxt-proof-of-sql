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

    fn aliased_literal() -> AliasedDynProofExpr {
        AliasedDynProofExpr {
            expr: DynProofExpr::new_literal(LiteralValue::VarChar("value".into())),
            alias: "result_alias".into(),
        }
    }

    #[test]
    fn aliased_dyn_proof_expr_clones_with_alias_and_expr() {
        let aliased = aliased_literal();

        assert_eq!(aliased.clone(), aliased);
    }

    #[test]
    fn aliased_dyn_proof_expr_round_trips_through_json() {
        let aliased = aliased_literal();
        let serialized = serde_json::to_string(&aliased).unwrap();
        let deserialized: AliasedDynProofExpr = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, aliased);
    }
}
