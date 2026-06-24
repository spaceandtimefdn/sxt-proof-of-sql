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
    use crate::sql::proof_exprs::{DynProofExpr, LiteralExpr};
    use crate::base::database::LiteralValue;
    use sqlparser::ast::Ident;

    fn make_aliased(alias: &str) -> AliasedDynProofExpr {
        AliasedDynProofExpr {
            expr: DynProofExpr::from(LiteralExpr::new(LiteralValue::Boolean(true))),
            alias: Ident::new(alias),
        }
    }

    #[test]
    fn aliased_expr_stores_alias() {
        let a = make_aliased("my_col");
        assert_eq!(a.alias, Ident::new("my_col"));
    }

    #[test]
    fn two_equal_aliased_exprs_are_eq() {
        let a = make_aliased("col");
        let b = make_aliased("col");
        assert_eq!(a, b);
    }

    #[test]
    fn two_different_aliases_are_not_eq() {
        let a = make_aliased("col1");
        let b = make_aliased("col2");
        assert_ne!(a, b);
    }

    #[test]
    fn clone_aliased_expr_is_equal() {
        let a = make_aliased("test");
        assert_eq!(a.clone(), a);
    }

    #[test]
    fn debug_output_contains_struct_name() {
        let a = make_aliased("debug_test");
        let debug = format!("{:?}", a);
        assert!(debug.contains("AliasedDynProofExpr"));
    }
}
