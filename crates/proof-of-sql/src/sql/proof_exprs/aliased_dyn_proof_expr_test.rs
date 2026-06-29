use super::{AliasedDynProofExpr, DynProofExpr};
use crate::base::database::LiteralValue;
use sqlparser::ast::Ident;

#[test]
fn aliased_dyn_proof_expr_round_trips_through_serde() {
    let aliased_expr = AliasedDynProofExpr {
        expr: DynProofExpr::new_literal(LiteralValue::Int(42)),
        alias: Ident::new("answer"),
    };

    let serialized = serde_json::to_string(&aliased_expr).unwrap();
    let deserialized: AliasedDynProofExpr = serde_json::from_str(&serialized).unwrap();

    assert_eq!(deserialized, aliased_expr);
    assert_eq!(deserialized.alias.value, "answer");
}
