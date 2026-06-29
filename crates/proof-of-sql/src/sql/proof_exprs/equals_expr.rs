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
mod tests {
    use super::{
        final_round_evaluate_equals_zero, first_round_evaluate_equals_zero,
        verifier_evaluate_equals_zero, EqualsExpr,
    };
    use crate::{
        base::{
            database::{
                table_utility::{borrowed_bigint, borrowed_boolean, table},
                ColumnRef, ColumnType, LiteralValue, TableRef,
            },
            map::{IndexMap, IndexSet},
            scalar::test_scalar::TestScalar,
        },
        sql::{
            proof::{
                mock_verification_builder::MockVerificationBuilder, FinalRoundBuilder,
                SumcheckSubpolynomialType,
            },
            proof_exprs::{DynProofExpr, ProofExpr},
            AnalyzeError,
        },
    };
    use alloc::collections::VecDeque;
    use bumpalo::Bump;
    use sqlparser::ast::Ident;

    fn bigint_literal(value: i64) -> DynProofExpr {
        DynProofExpr::new_literal(LiteralValue::BigInt(value))
    }

    fn bigint_column(table_ref: &TableRef, column: &str) -> DynProofExpr {
        DynProofExpr::new_column(ColumnRef::new(
            table_ref.clone(),
            Ident::new(column),
            ColumnType::BigInt,
        ))
    }

    #[test]
    fn first_round_equals_zero_marks_exact_zero_entries() {
        let alloc = Bump::new();
        let lhs = [
            TestScalar::from(0),
            TestScalar::from(5),
            TestScalar::from(0),
            TestScalar::from(9),
        ];

        let selection = first_round_evaluate_equals_zero(lhs.len(), &alloc, &lhs);

        assert_eq!(selection, &[true, false, true, false]);
    }

    #[test]
    fn equals_expr_constructor_accessors_and_references_are_direct() {
        let lhs = bigint_literal(10);
        let rhs = bigint_literal(10);

        let expr = EqualsExpr::try_new(Box::new(lhs.clone()), Box::new(rhs.clone())).unwrap();

        assert_eq!(expr.lhs(), &lhs);
        assert_eq!(expr.rhs(), &rhs);
        assert_eq!(expr.data_type(), ColumnType::Boolean);

        let mismatch = EqualsExpr::try_new(
            Box::new(lhs),
            Box::new(DynProofExpr::new_literal(LiteralValue::VarChar(
                "not numeric".into(),
            ))),
        )
        .unwrap_err();
        assert!(matches!(
            mismatch,
            AnalyzeError::DataTypeMismatch {
                left_type: _,
                right_type: _
            }
        ));

        let table_ref = TableRef::new("sxt", "t");
        let lhs_ref = ColumnRef::new(table_ref.clone(), Ident::new("a"), ColumnType::BigInt);
        let rhs_ref = ColumnRef::new(table_ref.clone(), Ident::new("b"), ColumnType::BigInt);
        let expr = EqualsExpr::try_new(
            Box::new(DynProofExpr::new_column(lhs_ref.clone())),
            Box::new(DynProofExpr::new_column(rhs_ref.clone())),
        )
        .unwrap();
        let mut column_references = IndexSet::default();

        expr.get_column_references(&mut column_references);

        assert_eq!(column_references.len(), 2);
        assert!(column_references.contains(&lhs_ref));
        assert!(column_references.contains(&rhs_ref));
    }

    #[test]
    fn equals_expr_evaluates_columns_without_blitzar() {
        let alloc = Bump::new();
        let table_ref = TableRef::new("sxt", "t");
        let data = table::<TestScalar>([
            borrowed_bigint("a", [1_i64, 2, 2, 4], &alloc),
            borrowed_bigint("b", [1_i64, 3, 2, 0], &alloc),
        ]);
        let expr = EqualsExpr::try_new(
            Box::new(bigint_column(&table_ref, "a")),
            Box::new(bigint_column(&table_ref, "b")),
        )
        .unwrap();
        let expected = borrowed_boolean("expected", [true, false, true, false], &alloc).1;

        let first_round = expr.first_round_evaluate(&alloc, &data, &[]).unwrap();

        assert_eq!(first_round, expected);

        let mut final_round_builder = FinalRoundBuilder::new(2, VecDeque::new());
        let final_round = expr
            .final_round_evaluate(&mut final_round_builder, &alloc, &data, &[])
            .unwrap();

        assert_eq!(final_round, expected);
        assert_eq!(final_round_builder.pcs_proof_mles().len(), 2);
        assert_eq!(final_round_builder.num_sumcheck_subpolynomials(), 2);
    }

    #[test]
    fn equals_expr_verifier_evaluates_columns_and_records_constraints() {
        let table_ref = TableRef::new("sxt", "t");
        let expr = EqualsExpr::try_new(
            Box::new(bigint_column(&table_ref, "a")),
            Box::new(bigint_column(&table_ref, "b")),
        )
        .unwrap();
        let mut accessor = IndexMap::default();
        accessor.insert(Ident::new("a"), TestScalar::from(7));
        accessor.insert(Ident::new("b"), TestScalar::from(7));
        let mut builder = MockVerificationBuilder::new(
            vec![],
            3,
            vec![],
            vec![vec![TestScalar::from(0), TestScalar::from(5)]],
            vec![],
            vec![],
            vec![],
        );

        let selection_eval = expr
            .verifier_evaluate(&mut builder, &accessor, TestScalar::from(5), &[])
            .unwrap();

        assert_eq!(selection_eval, TestScalar::from(5));
        assert_eq!(
            builder.identity_subpolynomial_evaluations,
            vec![vec![TestScalar::from(0), TestScalar::from(0)]]
        );
    }

    #[test]
    fn final_round_equals_zero_produces_selection_and_constraints() {
        let alloc = Bump::new();
        let lhs = alloc.alloc_slice_copy(&[
            TestScalar::from(0),
            TestScalar::from(5),
            TestScalar::from(7),
            TestScalar::from(0),
        ]);
        let mut builder = FinalRoundBuilder::new(2, VecDeque::new());

        let selection =
            final_round_evaluate_equals_zero(lhs.len(), &mut builder, &alloc, lhs as &[_]);

        assert_eq!(selection, &[true, false, false, true]);
        assert_eq!(builder.pcs_proof_mles().len(), 2);
        assert_eq!(builder.num_sumcheck_subpolynomials(), 2);
        assert!(builder
            .sumcheck_subpolynomials()
            .iter()
            .all(|subpoly| subpoly.subpolynomial_type() == SumcheckSubpolynomialType::Identity));
    }

    #[test]
    fn verifier_equals_zero_uses_final_round_mles_and_records_identity_checks() {
        let mut builder = MockVerificationBuilder::new(
            vec![],
            3,
            vec![],
            vec![vec![TestScalar::from(3), TestScalar::from(2)]],
            vec![],
            vec![],
            vec![],
        );

        let selection_eval =
            verifier_evaluate_equals_zero(&mut builder, TestScalar::from(1), TestScalar::from(5))
                .unwrap();

        assert_eq!(selection_eval, TestScalar::from(2));
        assert_eq!(
            builder.identity_subpolynomial_evaluations,
            vec![vec![TestScalar::from(2), TestScalar::from(0)]]
        );
    }

    #[test]
    fn verifier_equals_zero_errors_when_final_round_mles_are_missing() {
        let mut builder = MockVerificationBuilder::new(
            vec![],
            3,
            vec![],
            vec![vec![TestScalar::from(3)]],
            vec![],
            vec![],
            vec![],
        );

        assert!(verifier_evaluate_equals_zero(
            &mut builder,
            TestScalar::from(1),
            TestScalar::from(5)
        )
        .is_err());
    }
}
