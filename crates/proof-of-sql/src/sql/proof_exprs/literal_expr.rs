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
    use super::*;
    use crate::base::{database::LiteralValue, map::indexset};
    use alloc::{string::ToString, vec};

    #[test]
    fn we_can_create_literal_expr_and_read_back_the_value() {
        let v = LiteralValue::Boolean(true);
        let expr = LiteralExpr::new(v.clone());
        assert_eq!(expr.value(), &v);
    }

    #[test]
    fn data_type_matches_literal_value_column_type_for_each_simple_variant() {
        let cases = [
            LiteralValue::Boolean(true),
            LiteralValue::Uint8(1),
            LiteralValue::TinyInt(2),
            LiteralValue::SmallInt(3),
            LiteralValue::Int(4),
            LiteralValue::BigInt(5),
            LiteralValue::Int128(6),
            LiteralValue::VarChar("hello".to_string()),
            LiteralValue::VarBinary(vec![1u8, 2, 3]),
        ];
        for v in cases {
            let expected = v.column_type();
            let expr = LiteralExpr::new(v);
            assert_eq!(expr.data_type(), expected);
        }
    }

    #[test]
    fn literal_expr_records_no_column_references() {
        let expr = LiteralExpr::new(LiteralValue::BigInt(42));
        let mut refs: IndexSet<ColumnRef> = indexset! {};
        expr.get_column_references(&mut refs);
        assert!(refs.is_empty());
    }

    #[test]
    fn literal_expr_does_not_mutate_pre_existing_column_references() {
        // Pre-populated set must remain untouched: literals contribute zero refs.
        let expr = LiteralExpr::new(LiteralValue::Int(7));
        let mut refs: IndexSet<ColumnRef> = indexset! {};
        let before_len = refs.len();
        expr.get_column_references(&mut refs);
        assert_eq!(refs.len(), before_len);
    }

    #[test]
    fn equal_literal_values_produce_equal_literal_exprs() {
        let a = LiteralExpr::new(LiteralValue::Boolean(true));
        let b = LiteralExpr::new(LiteralValue::Boolean(true));
        assert_eq!(a, b);
    }

    #[test]
    fn different_inner_values_produce_different_literal_exprs() {
        let a = LiteralExpr::new(LiteralValue::Boolean(true));
        let b = LiteralExpr::new(LiteralValue::Boolean(false));
        assert_ne!(a, b);
    }

    #[test]
    fn different_variant_values_produce_different_literal_exprs() {
        let a = LiteralExpr::new(LiteralValue::Int(0));
        let b = LiteralExpr::new(LiteralValue::BigInt(0));
        assert_ne!(a, b);
    }

    #[test]
    fn literal_expr_can_be_cloned() {
        let original = LiteralExpr::new(LiteralValue::VarChar("clone-me".to_string()));
        let copy = original.clone();
        assert_eq!(original, copy);
        assert_eq!(copy.value(), original.value());
    }
}
