use super::{DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{can_and_or_types, Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
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

/// Provable logical OR expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrExpr {
    lhs: Box<DynProofExpr>,
    rhs: Box<DynProofExpr>,
}

impl OrExpr {
    /// Create logical OR expression
    pub fn try_new(lhs: Box<DynProofExpr>, rhs: Box<DynProofExpr>) -> AnalyzeResult<Self> {
        let left_datatype = lhs.data_type();
        let right_datatype = rhs.data_type();
        can_and_or_types(left_datatype, right_datatype)
            .then_some(Self { lhs, rhs })
            .ok_or_else(|| AnalyzeError::DataTypeMismatch {
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

impl ProofExpr for OrExpr {
    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean
    }

    #[tracing::instrument(name = "OrExpr::first_round_evaluate", level = "debug", skip_all)]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let lhs_column: Column<'a, S> = self.lhs.first_round_evaluate(alloc, table, params)?;
        let rhs_column: Column<'a, S> = self.rhs.first_round_evaluate(alloc, table, params)?;
        let lhs = lhs_column.as_boolean().expect("lhs is not boolean");
        let rhs = rhs_column.as_boolean().expect("rhs is not boolean");
        let result = Column::Boolean(first_round_evaluate_or(table.num_rows(), alloc, lhs, rhs));

        log::log_memory_usage("End");

        Ok(result)
    }

    #[tracing::instrument(name = "OrExpr::final_round_evaluate", level = "debug", skip_all)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let lhs_column: Column<'a, S> = self
            .lhs
            .final_round_evaluate(builder, alloc, table, params)?;
        let rhs_column: Column<'a, S> = self
            .rhs
            .final_round_evaluate(builder, alloc, table, params)?;
        let lhs = lhs_column.as_boolean().expect("lhs is not boolean");
        let rhs = rhs_column.as_boolean().expect("rhs is not boolean");
        let result = Column::Boolean(final_round_evaluate_or(builder, alloc, lhs, rhs));

        log::log_memory_usage("End");

        Ok(result)
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<Ident, S>,
        chi_eval: S,
        params: &[LiteralValue],
    ) -> Result<S, ProofError> {
        let lhs = self
            .lhs
            .verifier_evaluate(builder, accessor, chi_eval, params)?;
        let rhs = self
            .rhs
            .verifier_evaluate(builder, accessor, chi_eval, params)?;

        verifier_evaluate_or(builder, &lhs, &rhs)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}

#[expect(
    clippy::missing_panics_doc,
    reason = "table_length matches lhs and rhs lengths, ensuring no panic occurs"
)]
pub fn first_round_evaluate_or<'a>(
    table_length: usize,
    alloc: &'a Bump,
    lhs: &[bool],
    rhs: &[bool],
) -> &'a [bool] {
    assert_eq!(table_length, lhs.len());
    assert_eq!(table_length, rhs.len());
    alloc.alloc_slice_fill_with(table_length, |i| lhs[i] || rhs[i])
}

#[expect(
    clippy::missing_panics_doc,
    reason = "lhs and rhs are guaranteed to have the same length, ensuring no panic occurs"
)]
pub fn final_round_evaluate_or<'a, S: Scalar>(
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    lhs: &'a [bool],
    rhs: &'a [bool],
) -> &'a [bool] {
    let n = lhs.len();
    assert_eq!(n, rhs.len());

    // lhs_and_rhs
    let lhs_and_rhs: &[_] = alloc.alloc_slice_fill_with(n, |i| lhs[i] && rhs[i]);
    builder.produce_intermediate_mle(lhs_and_rhs);

    // subpolynomial: lhs_and_rhs - lhs * rhs
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(lhs_and_rhs)]),
            (-S::one(), vec![Box::new(lhs), Box::new(rhs)]),
        ],
    );

    // selection
    alloc.alloc_slice_fill_with(n, |i| lhs[i] || rhs[i])
}

pub fn verifier_evaluate_or<S: Scalar>(
    builder: &mut impl VerificationBuilder<S>,
    lhs: &S,
    rhs: &S,
) -> Result<S, ProofError> {
    // lhs_and_rhs
    let lhs_and_rhs = builder.try_consume_final_round_mle_evaluation()?;

    // subpolynomial: lhs_and_rhs - lhs * rhs
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        lhs_and_rhs - *lhs * *rhs,
        2,
    )?;

    // selection
    Ok(*lhs + *rhs - lhs_and_rhs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::{
            database::{
                table_utility::{borrowed_boolean, table},
                TableRef,
            },
            scalar::test_scalar::TestScalar,
        },
        sql::proof::mock_verification_builder::MockVerificationBuilder,
    };
    use alloc::{collections::VecDeque, vec};

    fn boolean_column_ref(name: &str) -> ColumnRef {
        ColumnRef::new(
            TableRef::new("sxt", "or_inputs"),
            Ident::new(name),
            ColumnType::Boolean,
        )
    }

    #[test]
    fn or_expr_evaluates_rounds_without_blitzar() {
        let alloc = Bump::new();
        let table = table::<TestScalar>([
            borrowed_boolean("lhs", [false, false, true, true], &alloc),
            borrowed_boolean("rhs", [false, true, false, true], &alloc),
        ]);
        let lhs_ref = boolean_column_ref("lhs");
        let rhs_ref = boolean_column_ref("rhs");
        let or_expr = OrExpr::try_new(
            Box::new(DynProofExpr::new_column(lhs_ref.clone())),
            Box::new(DynProofExpr::new_column(rhs_ref.clone())),
        )
        .unwrap();

        assert_eq!(or_expr.data_type(), ColumnType::Boolean);
        assert_eq!(or_expr.lhs().data_type(), ColumnType::Boolean);
        assert_eq!(or_expr.rhs().data_type(), ColumnType::Boolean);

        let first_round = or_expr.first_round_evaluate(&alloc, &table, &[]).unwrap();
        assert_eq!(first_round, Column::Boolean(&[false, true, true, true]));

        let mut final_round_builder = FinalRoundBuilder::new(0, VecDeque::new());
        let final_round = or_expr
            .final_round_evaluate(&mut final_round_builder, &alloc, &table, &[])
            .unwrap();
        assert_eq!(final_round, Column::Boolean(&[false, true, true, true]));
        assert_eq!(final_round_builder.pcs_proof_mles().len(), 1);
        assert_eq!(final_round_builder.num_sumcheck_subpolynomials(), 1);

        let mut column_refs = IndexSet::default();
        or_expr.get_column_references(&mut column_refs);
        assert_eq!(column_refs.len(), 2);
        assert!(column_refs.contains(&lhs_ref));
        assert!(column_refs.contains(&rhs_ref));
    }

    #[test]
    fn verifier_evaluate_or_consumes_intermediate_mle_without_blitzar() {
        let mut builder = MockVerificationBuilder::<TestScalar>::new(
            vec![],
            3,
            vec![],
            vec![vec![TestScalar::from(0u64)]],
            vec![],
            vec![],
            vec![],
        );

        let result = verifier_evaluate_or(
            &mut builder,
            &TestScalar::from(1u64),
            &TestScalar::from(0u64),
        )
        .unwrap();

        assert_eq!(result, TestScalar::from(1u64));
        assert_eq!(builder.get_identity_results(), vec![vec![true]]);
    }

    #[test]
    fn or_expr_rejects_non_boolean_operands_without_blitzar() {
        let error = OrExpr::try_new(
            Box::new(DynProofExpr::new_literal(LiteralValue::Boolean(true))),
            Box::new(DynProofExpr::new_literal(LiteralValue::BigInt(1))),
        )
        .unwrap_err();

        assert!(matches!(error, AnalyzeError::DataTypeMismatch { .. }));
    }
}
