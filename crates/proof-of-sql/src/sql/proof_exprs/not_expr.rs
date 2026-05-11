use super::{DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{can_not_type, Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{FinalRoundBuilder, VerificationBuilder},
        AnalyzeError, AnalyzeResult,
    },
    utils::log,
};
use alloc::boxed::Box;
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Provable logical NOT expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NotExpr {
    expr: Box<DynProofExpr>,
}

impl NotExpr {
    /// Create logical NOT expression
    pub fn try_new(expr: Box<DynProofExpr>) -> AnalyzeResult<Self> {
        let expr_type = expr.data_type();
        can_not_type(expr_type)
            .then_some(Self { expr })
            .ok_or(AnalyzeError::InvalidDataType { expr_type })
    }

    /// Get the input expression
    pub fn input(&self) -> &DynProofExpr {
        &self.expr
    }
}

impl ProofExpr for NotExpr {
    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean
    }

    #[tracing::instrument(name = "NotExpr::first_round_evaluate", level = "debug", skip_all)]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let expr_column: Column<'a, S> = self.expr.first_round_evaluate(alloc, table, params)?;
        let expr = expr_column.as_boolean().expect("expr is not boolean");
        let res = Column::Boolean(alloc.alloc_slice_fill_with(expr.len(), |i| !expr[i]));

        log::log_memory_usage("End");

        Ok(res)
    }

    #[tracing::instrument(name = "NotExpr::final_round_evaluate", level = "debug", skip_all)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let expr_column: Column<'a, S> = self
            .expr
            .final_round_evaluate(builder, alloc, table, params)?;
        let expr = expr_column.as_boolean().expect("expr is not boolean");
        let res = Column::Boolean(alloc.alloc_slice_fill_with(expr.len(), |i| !expr[i]));

        log::log_memory_usage("End");

        Ok(res)
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<Ident, S>,
        chi_eval: S,
        params: &[LiteralValue],
    ) -> Result<S, ProofError> {
        let eval = self
            .expr
            .verifier_evaluate(builder, accessor, chi_eval, params)?;
        Ok(chi_eval - eval)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.expr.get_column_references(columns);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::{
            database::{Column, TableOptions},
            map::IndexMap,
            scalar::test_scalar::TestScalar,
        },
        sql::proof::mock_verification_builder::MockVerificationBuilder,
    };
    use alloc::collections::VecDeque;

    fn empty_table(row_count: usize) -> Table<'static, TestScalar> {
        Table::try_new_with_options(IndexMap::default(), TableOptions::new(Some(row_count)))
            .unwrap()
    }

    fn bool_expr(value: bool) -> Box<DynProofExpr> {
        Box::new(DynProofExpr::new_literal(LiteralValue::Boolean(value)))
    }

    #[test]
    fn try_new_accepts_boolean_input_and_exposes_child() {
        let input = bool_expr(true);
        let expr = NotExpr::try_new(input.clone()).unwrap();

        assert_eq!(expr.data_type(), ColumnType::Boolean);
        assert_eq!(expr.input(), input.as_ref());
    }

    #[test]
    fn first_and_final_round_evaluate_boolean_literal() {
        let alloc = Bump::new();
        let table = empty_table(3);
        let expr = NotExpr::try_new(bool_expr(true)).unwrap();

        let first_round = expr.first_round_evaluate(&alloc, &table, &[]).unwrap();
        assert_eq!(first_round, Column::Boolean(&[false, false, false]));

        let mut builder = FinalRoundBuilder::new(2, VecDeque::new());
        let final_round = expr
            .final_round_evaluate(&mut builder, &alloc, &table, &[])
            .unwrap();
        assert_eq!(final_round, Column::Boolean(&[false, false, false]));
        assert!(builder.pcs_proof_mles().is_empty());
        assert_eq!(builder.num_sumcheck_subpolynomials(), 0);
    }

    #[test]
    fn verifier_evaluate_subtracts_input_from_chi() {
        let chi_eval = TestScalar::from(9u64);
        let mut builder = MockVerificationBuilder::new(
            Vec::new(),
            0,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
        let expr = NotExpr::try_new(bool_expr(false)).unwrap();

        let result = expr
            .verifier_evaluate(&mut builder, &IndexMap::default(), chi_eval, &[])
            .unwrap();

        assert_eq!(result, chi_eval);
    }
}
