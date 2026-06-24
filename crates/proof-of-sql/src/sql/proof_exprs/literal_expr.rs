use super::ProofExpr;
use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::proof::{FinalRoundBuilder, VerificationBuilder},
    utils::log,
};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Provable CONST expression
///
/// This node allows us to easily represent queries like
///    select * from T
/// and
///    select * from T where 1 = 2
/// as filter expressions with a constant where clause.
///
/// While this wouldn't be as efficient as using a new custom expression for
/// such queries, it allows us to easily support projects with minimal code
/// changes, and the performance is sufficient for present.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiteralExpr {
    value: LiteralValue,
}

impl LiteralExpr {
    /// Create literal expression
    pub fn new(value: LiteralValue) -> Self {
        Self { value }
    }

    /// Get the literal value
    pub fn value(&self) -> &LiteralValue {
        &self.value
    }
}

impl ProofExpr for LiteralExpr {
    fn data_type(&self) -> ColumnType {
        self.value.column_type()
    }

    #[tracing::instrument(name = "LiteralExpr::first_round_evaluate", level = "debug", skip_all)]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        _params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let res = Column::from_literal_with_length(&self.value, table.num_rows(), alloc);

        log::log_memory_usage("End");

        Ok(res)
    }

    #[tracing::instrument(name = "LiteralExpr::final_round_evaluate", level = "debug", skip_all)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        _builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        _params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let table_length = table.num_rows();
        let res = Column::from_literal_with_length(&self.value, table_length, alloc);

        log::log_memory_usage("End");

        Ok(res)
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        _builder: &mut impl VerificationBuilder<S>,
        _accessor: &IndexMap<Ident, S>,
        chi_eval: S,
        _params: &[LiteralValue],
    ) -> Result<S, ProofError> {
        Ok(chi_eval * self.value.to_scalar())
    }

    fn get_column_references(&self, _columns: &mut IndexSet<ColumnRef>) {}
}

#[cfg(test)]
mod tests {
    use super::LiteralExpr;
    use crate::base::database::{ColumnType, LiteralValue};

    #[test]
    fn new_boolean_literal_has_correct_value() {
        let expr = LiteralExpr::new(LiteralValue::Boolean(true));
        assert_eq!(expr.value(), &LiteralValue::Boolean(true));
    }

    #[test]
    fn new_bigint_literal_has_correct_value() {
        let expr = LiteralExpr::new(LiteralValue::BigInt(42));
        assert_eq!(expr.value(), &LiteralValue::BigInt(42));
    }

    #[test]
    fn boolean_literal_data_type_is_boolean() {
        use crate::sql::proof_exprs::ProofExpr;
        let expr = LiteralExpr::new(LiteralValue::Boolean(false));
        assert_eq!(expr.data_type(), ColumnType::Boolean);
    }

    #[test]
    fn bigint_literal_data_type_is_bigint() {
        use crate::sql::proof_exprs::ProofExpr;
        let expr = LiteralExpr::new(LiteralValue::BigInt(0));
        assert_eq!(expr.data_type(), ColumnType::BigInt);
    }

    #[test]
    fn int128_literal_data_type_is_int128() {
        use crate::sql::proof_exprs::ProofExpr;
        let expr = LiteralExpr::new(LiteralValue::Int128(i128::MAX));
        assert_eq!(expr.data_type(), ColumnType::Int128);
    }

    #[test]
    fn two_equal_literals_are_eq() {
        let a = LiteralExpr::new(LiteralValue::Boolean(true));
        let b = LiteralExpr::new(LiteralValue::Boolean(true));
        assert_eq!(a, b);
    }

    #[test]
    fn two_different_literals_are_not_eq() {
        let a = LiteralExpr::new(LiteralValue::Boolean(true));
        let b = LiteralExpr::new(LiteralValue::Boolean(false));
        assert_ne!(a, b);
    }

    #[test]
    fn literal_expr_clone_is_equal() {
        let expr = LiteralExpr::new(LiteralValue::BigInt(99));
        let cloned = expr.clone();
        assert_eq!(expr, cloned);
    }

    #[test]
    fn literal_expr_debug_contains_value() {
        let expr = LiteralExpr::new(LiteralValue::BigInt(7));
        let debug = format!("{:?}", expr);
        assert!(debug.contains("LiteralExpr"));
    }
}
