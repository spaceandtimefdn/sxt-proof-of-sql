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
    use crate::{
        base::{database::TableOptions, map::indexmap, scalar::test_scalar::TestScalar},
        sql::proof::mock_verification_builder::MockVerificationBuilder,
    };
    use alloc::{collections::VecDeque, vec};

    #[test]
    fn we_can_inspect_literal_expr_value_and_type() {
        let expr = LiteralExpr::new(LiteralValue::BigInt(42));

        assert_eq!(expr.value(), &LiteralValue::BigInt(42));
        assert_eq!(expr.data_type(), ColumnType::BigInt);
    }

    #[test]
    fn we_can_evaluate_literal_expr_in_prover_rounds() {
        let alloc = Bump::new();
        let table = Table::<TestScalar>::try_new_with_options(
            indexmap! {
                Ident::new("row_marker") => Column::Boolean(&[true, false, true]),
            },
            TableOptions::new(Some(3)),
        )
        .unwrap();
        let expr = LiteralExpr::new(LiteralValue::BigInt(7));

        let first_round = expr.first_round_evaluate(&alloc, &table, &[]).unwrap();
        assert_eq!(first_round, Column::BigInt(&[7, 7, 7]));

        let mut builder = FinalRoundBuilder::new(2, VecDeque::new());
        let final_round = expr
            .final_round_evaluate(&mut builder, &alloc, &table, &[])
            .unwrap();
        assert_eq!(final_round, Column::BigInt(&[7, 7, 7]));
    }

    #[test]
    fn we_can_verify_literal_expr_evaluation() {
        let expr = LiteralExpr::new(LiteralValue::BigInt(7));
        let mut builder =
            MockVerificationBuilder::new(vec![], 0, vec![], vec![], vec![], vec![], vec![]);
        let chi_eval = TestScalar::from(11u64);

        let evaluation = expr
            .verifier_evaluate(&mut builder, &IndexMap::default(), chi_eval, &[])
            .unwrap();

        assert_eq!(evaluation, chi_eval * TestScalar::from(7u64));
    }

    #[test]
    fn literal_expr_has_no_column_references() {
        let expr = LiteralExpr::new(LiteralValue::Boolean(true));
        let mut columns = IndexSet::default();

        expr.get_column_references(&mut columns);

        assert!(columns.is_empty());
    }
}
