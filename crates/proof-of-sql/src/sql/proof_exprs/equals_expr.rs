use super::{add_subtract_columns, DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{try_equals_types, Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
        slice_ops,
    },
    sql::{
        proof::{FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder},
        AnalyzeError, AnalyzeResult,
    },
    utils::log,
};
use alloc::{boxed::Box, string::ToString, vec};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Provable AST expression for an equals expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EqualsExpr {
    lhs: Box<DynProofExpr>,
    rhs: Box<DynProofExpr>,
}

impl EqualsExpr {
    /// Create a new equals expression
    pub fn try_new(lhs: Box<DynProofExpr>, rhs: Box<DynProofExpr>) -> AnalyzeResult<Self> {
        let left_datatype = lhs.data_type();
        let right_datatype = rhs.data_type();
        try_equals_types(left_datatype, right_datatype)
            .map(|()| Self { lhs, rhs })
            .map_err(|_| AnalyzeError::DataTypeMismatch {
                left_type: left_datatype.to_string(),
                right_type: right_datatype.to_string(),
            })
    }

    /// Get the left-hand side expression
    pub fn lhs(&self) -> &DynProofExpr {
        &self.lhs
    }

    /// Get the right-hand side expression
    pub fn rhs(&self) -> &DynProofExpr {
        &self.rhs
    }
}

impl ProofExpr for EqualsExpr {
    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean
    }

    #[tracing::instrument(name = "EqualsExpr::first_round_evaluate", level = "debug", skip_all)]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let lhs_column = self.lhs.first_round_evaluate(alloc, table, params)?;
        let rhs_column = self.rhs.first_round_evaluate(alloc, table, params)?;
        let res = add_subtract_columns(lhs_column, rhs_column, alloc, true);
        let res = Column::Boolean(first_round_evaluate_equals_zero(
            table.num_rows(),
            alloc,
            res,
        ));

        log::log_memory_usage("End");

        Ok(res)
    }

    #[tracing::instrument(name = "EqualsExpr::final_round_evaluate", level = "debug", skip_all)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let lhs_column = self
            .lhs
            .final_round_evaluate(builder, alloc, table, params)?;
        let rhs_column = self
            .rhs
            .final_round_evaluate(builder, alloc, table, params)?;
        let scale_and_subtract_res = add_subtract_columns(lhs_column, rhs_column, alloc, true);
        let res = Column::Boolean(final_round_evaluate_equals_zero(
            table.num_rows(),
            builder,
            alloc,
            scale_and_subtract_res,
        ));

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
        let lhs_eval = self
            .lhs
            .verifier_evaluate(builder, accessor, chi_eval, params)?;
        let rhs_eval = self
            .rhs
            .verifier_evaluate(builder, accessor, chi_eval, params)?;
        verifier_evaluate_equals_zero(builder, lhs_eval - rhs_eval, chi_eval)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}

#[tracing::instrument(
    name = "EqualsExpr::first_round_evaluate_equals_zero",
    level = "debug",
    skip_all
)]
pub fn first_round_evaluate_equals_zero<'a, S: Scalar>(
    table_length: usize,
    alloc: &'a Bump,
    lhs: &'a [S],
) -> &'a [bool] {
    assert_eq!(table_length, lhs.len());
    alloc.alloc_slice_fill_with(table_length, |i| lhs[i] == S::zero())
}

#[tracing::instrument(
    name = "EqualsExpr::final_round_evaluate_equals_zero",
    level = "debug",
    skip_all
)]
pub fn final_round_evaluate_equals_zero<'a, S: Scalar>(
    table_length: usize,
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    lhs: &'a [S],
) -> &'a [bool] {
    // lhs_pseudo_inv
    let lhs_pseudo_inv = alloc.alloc_slice_copy(lhs);
    slice_ops::batch_inversion(lhs_pseudo_inv);

    builder.produce_intermediate_mle(lhs_pseudo_inv as &[_]);

    // selection_not
    let selection_not: &[_] = alloc.alloc_slice_fill_with(table_length, |i| lhs[i] != S::zero());

    // selection
    let selection: &[_] = alloc.alloc_slice_fill_with(table_length, |i| !selection_not[i]);
    builder.produce_intermediate_mle(selection);

    // subpolynomial: selection * lhs
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![(S::one(), vec![Box::new(lhs), Box::new(selection)])],
    );

    // subpolynomial: selection_not - lhs * lhs_pseudo_inv
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(selection_not)]),
            (
                -S::one(),
                vec![Box::new(lhs), Box::new(lhs_pseudo_inv as &[_])],
            ),
        ],
    );

    selection
}

pub fn verifier_evaluate_equals_zero<S: Scalar>(
    builder: &mut impl VerificationBuilder<S>,
    lhs_eval: S,
    chi_eval: S,
) -> Result<S, ProofError> {
    // consume mle evaluations
    let lhs_pseudo_inv_eval = builder.try_consume_final_round_mle_evaluation()?;
    let selection_eval = builder.try_consume_final_round_mle_evaluation()?;
    let selection_not_eval = chi_eval - selection_eval;

    // subpolynomial: selection * lhs
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        selection_eval * lhs_eval,
        2,
    )?;

    // subpolynomial: selection_not - lhs * lhs_pseudo_inv
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        selection_not_eval - lhs_eval * lhs_pseudo_inv_eval,
        2,
    )?;

    Ok(selection_eval)
}

#[cfg(test)]
mod tests_equals {
    use crate::{
        base::database::{ColumnType, LiteralValue},
        sql::proof_exprs::{DynProofExpr, EqualsExpr, ProofExpr},
    };

    fn bigint_expr() -> DynProofExpr {
        DynProofExpr::new_literal(LiteralValue::BigInt(5))
    }

    fn bool_expr() -> DynProofExpr {
        DynProofExpr::new_literal(LiteralValue::Boolean(true))
    }

    #[test]
    fn try_new_with_same_types_returns_ok() {
        assert!(EqualsExpr::try_new(alloc::boxed::Box::new(bigint_expr()), alloc::boxed::Box::new(bigint_expr())).is_ok());
    }

    #[test]
    fn try_new_with_different_types_returns_err() {
        assert!(EqualsExpr::try_new(alloc::boxed::Box::new(bigint_expr()), alloc::boxed::Box::new(bool_expr())).is_err());
    }

    #[test]
    fn data_type_is_boolean() {
        let e = EqualsExpr::try_new(alloc::boxed::Box::new(bigint_expr()), alloc::boxed::Box::new(bigint_expr())).unwrap();
        assert_eq!(e.data_type(), ColumnType::Boolean);
    }

    #[test]
    fn lhs_has_correct_type() {
        let e = EqualsExpr::try_new(alloc::boxed::Box::new(bigint_expr()), alloc::boxed::Box::new(bigint_expr())).unwrap();
        assert_eq!(e.lhs().data_type(), ColumnType::BigInt);
    }

    #[test]
    fn equality_holds_for_same_exprs() {
        let a = EqualsExpr::try_new(alloc::boxed::Box::new(bigint_expr()), alloc::boxed::Box::new(bigint_expr())).unwrap();
        let b = EqualsExpr::try_new(alloc::boxed::Box::new(bigint_expr()), alloc::boxed::Box::new(bigint_expr())).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn debug_contains_struct_name() {
        let e = EqualsExpr::try_new(alloc::boxed::Box::new(bigint_expr()), alloc::boxed::Box::new(bigint_expr())).unwrap();
        assert!(alloc::format!("{e:?}").contains("EqualsExpr"));
    }
}

#[cfg(test)]
mod tests_projection {
    use crate::{
        base::database::{LiteralValue},
        sql::proof_exprs::{AliasedDynProofExpr, DynProofExpr},
        sql::proof_plans::{DynProofPlan, ProjectionExec},
        sql::proof::ProofPlan,
    };
    use sqlparser::ast::Ident;

    fn make_aliased() -> AliasedDynProofExpr {
        AliasedDynProofExpr {
            expr: DynProofExpr::new_literal(LiteralValue::BigInt(1)),
            alias: Ident::new("col"),
        }
    }

    #[test]
    fn new_stores_aliased_results() {
        let e = ProjectionExec::new(alloc::vec![make_aliased()], alloc::boxed::Box::new(DynProofPlan::new_empty()));
        assert_eq!(e.aliased_results().len(), 1);
    }

    #[test]
    fn input_returns_inner_plan() {
        let e = ProjectionExec::new(alloc::vec![], alloc::boxed::Box::new(DynProofPlan::new_empty()));
        assert!(matches!(e.input(), DynProofPlan::Empty(_)));
    }

    #[test]
    fn get_column_result_fields_matches_aliased_results() {
        let e = ProjectionExec::new(alloc::vec![], alloc::boxed::Box::new(DynProofPlan::new_empty()));
        assert!(e.get_column_result_fields().is_empty());
    }

    #[test]
    fn equality_holds() {
        let a = ProjectionExec::new(alloc::vec![], alloc::boxed::Box::new(DynProofPlan::new_empty()));
        let b = ProjectionExec::new(alloc::vec![], alloc::boxed::Box::new(DynProofPlan::new_empty()));
        assert_eq!(a, b);
    }

    #[test]
    fn debug_contains_struct_name() {
        let e = ProjectionExec::new(alloc::vec![], alloc::boxed::Box::new(DynProofPlan::new_empty()));
        assert!(alloc::format!("{e:?}").contains("ProjectionExec"));
    }
}
