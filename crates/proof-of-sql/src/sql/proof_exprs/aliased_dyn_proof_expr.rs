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
    fn we_clone_and_round_trip_aliased_expressions_through_json() {
        let expr = AliasedDynProofExpr {
            expr: DynProofExpr::new_literal(LiteralValue::Boolean(true)),
            alias: "is_active".into(),
        };

        assert_eq!(expr.clone(), expr);

        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: AliasedDynProofExpr = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized, expr);
    }
}
