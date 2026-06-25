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
    use super::AliasedDynProofExpr;
    use crate::{base::database::LiteralValue, sql::proof_exprs::DynProofExpr};
    use sqlparser::ast::Ident;

    fn bigint_expr() -> DynProofExpr {
        DynProofExpr::new_literal(LiteralValue::BigInt(1))
    }

    #[test]
    fn fields_are_accessible() {
        let aliased = AliasedDynProofExpr {
            expr: bigint_expr(),
            alias: Ident::new("myalias"),
        };
        assert_eq!(aliased.alias.value.as_str(), "myalias");
    }

    #[test]
    fn equality_holds_for_same_values() {
        let a = AliasedDynProofExpr { expr: bigint_expr(), alias: Ident::new("a") };
        let b = AliasedDynProofExpr { expr: bigint_expr(), alias: Ident::new("a") };
        assert_eq!(a, b);
    }

    #[test]
    fn clone_produces_equal_value() {
        let a = AliasedDynProofExpr { expr: bigint_expr(), alias: Ident::new("a") };
        assert_eq!(a.clone(), a);
    }

    #[test]
    fn debug_contains_struct_name() {
        let a = AliasedDynProofExpr { expr: bigint_expr(), alias: Ident::new("a") };
        assert!(alloc::format!("{a:?}").contains("AliasedDynProofExpr"));
    }
}
