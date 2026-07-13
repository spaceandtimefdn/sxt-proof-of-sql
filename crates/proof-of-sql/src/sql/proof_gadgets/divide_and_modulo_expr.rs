use crate::{
    base::{
        database::{Column, LiteralValue, Table},
        map::IndexMap,
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{FinalRoundBuilder, VerificationBuilder},
        proof_exprs::{divide_columns, modulo_columns, DynProofExpr, ProofExpr},
    },
    utils::log,
};
use alloc::boxed::Box;
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// TODO: This struct is only partially complete. This should not be used yet. Several constraints still need to be added.
/// A gadget for proving divide and modulo expressions in tandem.
/// They must be proved in tandem under this protocol.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DivideAndModuloExpr {
    pub lhs: Box<DynProofExpr>,
    pub rhs: Box<DynProofExpr>,
}

trait DivideAndModuloExprUtilities<S: Scalar> {
    fn divide_columns<'a>(
        &self,
        lhs: &Column<'a, S>,
        rhs: &Column<'a, S>,
        alloc: &'a Bump,
    ) -> (Column<'a, S>, &'a [S]);

    fn modulo_columns<'a>(
        &self,
        lhs: &Column<'a, S>,
        rhs: &Column<'a, S>,
        alloc: &'a Bump,
    ) -> Column<'a, S>;
}

struct StandardDivideAndModuloExprUtilities;

impl<S: Scalar> DivideAndModuloExprUtilities<S> for StandardDivideAndModuloExprUtilities {
    fn divide_columns<'a>(
        &self,
        lhs: &Column<'a, S>,
        rhs: &Column<'a, S>,
        alloc: &'a Bump,
    ) -> (Column<'a, S>, &'a [S]) {
        divide_columns(lhs, rhs, alloc)
    }

    fn modulo_columns<'a>(
        &self,
        lhs: &Column<'a, S>,
        rhs: &Column<'a, S>,
        alloc: &'a Bump,
    ) -> Column<'a, S> {
        modulo_columns(lhs, rhs, alloc)
    }
}

impl DivideAndModuloExpr {
    #[cfg_attr(not(test), expect(dead_code))]
    fn new(lhs: Box<DynProofExpr>, rhs: Box<DynProofExpr>) -> Self {
        Self { lhs, rhs }
    }

    /// This is abstracted into its own function for ease of unit testing.
    /// The `utilities` function is where any functionality that needs to be mocked
    /// can be provided.
    fn final_round_evaluate_base<'a, S: Scalar, U: DivideAndModuloExprUtilities<S>>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        utilities: &U,
        params: &[LiteralValue],
    ) -> PlaceholderResult<(Column<'a, S>, Column<'a, S>)> {
        let lhs_column: Column<'a, S> = self
            .lhs
            .final_round_evaluate(builder, alloc, table, params)?;
        let rhs_column: Column<'a, S> = self
            .rhs
            .final_round_evaluate(builder, alloc, table, params)?;

        let (quotient_wrapped, _quotient) =
            utilities.divide_columns(&lhs_column, &rhs_column, alloc);
        let remainder = utilities.modulo_columns(&lhs_column, &rhs_column, alloc);

        builder.produce_intermediate_mle(quotient_wrapped);
        builder.produce_intermediate_mle(remainder);

        Ok((quotient_wrapped, remainder))
    }

    #[cfg_attr(not(test), expect(dead_code))]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<(Column<'a, S>, Column<'a, S>)> {
        log::log_memory_usage("Start");
        let utilities = StandardDivideAndModuloExprUtilities {};

        let res = self.final_round_evaluate_base(builder, alloc, table, &utilities, params)?;

        log::log_memory_usage("End");

        Ok(res)
    }

    #[cfg_attr(not(test), expect(dead_code))]
    fn verifier_evaluate<S: Scalar, B: VerificationBuilder<S>>(
        &self,
        builder: &mut B,
        accessor: &IndexMap<Ident, S>,
        one_eval: S,
        params: &[LiteralValue],
    ) -> Result<(S, S), ProofError> {
        let _lhs = self
            .lhs
            .verifier_evaluate(builder, accessor, one_eval, params)?;
        let _rhs = self
            .rhs
            .verifier_evaluate(builder, accessor, one_eval, params)?;

        // lhs_times_rhs
        let quotient_wrapped = builder.try_consume_final_round_mle_evaluation()?;
        let remainder = builder.try_consume_final_round_mle_evaluation()?;

        Ok((quotient_wrapped, remainder))
    }
}

#[cfg(test)]
mod tests {
    use super::DivideAndModuloExpr;
    use crate::{
        base::{
            database::{Column, ColumnRef, ColumnType, Table, TableRef},
            map::indexmap,
            polynomial::MultilinearExtension,
            proof::{ProofError, ProofSizeMismatch},
            scalar::{test_scalar::TestScalar, Scalar},
        },
        sql::{
            proof::{
                mock_verification_builder::{run_verify_for_each_row, MockVerificationBuilder},
                FinalRoundBuilder, FirstRoundBuilder,
            },
            proof_exprs::{ColumnExpr, DynProofExpr},
        },
    };
    use bumpalo::Bump;
    use sqlparser::ast::Ident;
    use std::collections::VecDeque;

    fn divide_and_modulo_expr() -> (DivideAndModuloExpr, ColumnRef, ColumnRef, Ident, Ident) {
        let table_ref: TableRef = "sxt.t".parse().unwrap();
        let lhs_ident = Ident::from("lhs");
        let rhs_ident = Ident::from("rhs");
        let lhs_ref = ColumnRef::new(table_ref.clone(), lhs_ident.clone(), ColumnType::Int128);
        let rhs_ref = ColumnRef::new(table_ref, rhs_ident.clone(), ColumnType::Int128);
        let divide_and_modulo_expr = DivideAndModuloExpr::new(
            Box::new(DynProofExpr::Column(ColumnExpr::new(lhs_ref.clone()))),
            Box::new(DynProofExpr::Column(ColumnExpr::new(rhs_ref.clone()))),
        );
        (
            divide_and_modulo_expr,
            lhs_ref,
            rhs_ref,
            lhs_ident,
            rhs_ident,
        )
    }

    fn mock_verification_builder(
        final_round_mles: Vec<Vec<TestScalar>>,
    ) -> MockVerificationBuilder<TestScalar> {
        MockVerificationBuilder::new(
            Vec::new(),
            2,
            Vec::new(),
            final_round_mles,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
    }

    #[test]
    fn we_can_verify_simple_expr() {
        let alloc = Bump::new();
        let (divide_and_modulo_expr, lhs_ref, rhs_ref, lhs_ident, rhs_ident) =
            divide_and_modulo_expr();
        let lhs = &[i128::MAX, i128::MIN, 2];
        let rhs = &[3i128, 3, -4];
        let first_round_builder: FirstRoundBuilder<'_, _> = FirstRoundBuilder::new(lhs.len());
        let mut final_round_builder = FinalRoundBuilder::new(lhs.len(), VecDeque::new());
        let table = Table::try_new(indexmap! {
            lhs_ident => Column::Int128::<TestScalar>(lhs),
            rhs_ident => Column::Int128::<TestScalar>(rhs),
        })
        .unwrap();
        divide_and_modulo_expr
            .final_round_evaluate(&mut final_round_builder, &alloc, &table, &[])
            .unwrap();
        let mock_verification_builder = run_verify_for_each_row(
            lhs.len(),
            &first_round_builder,
            &final_round_builder,
            Vec::new(),
            4,
            |verification_builder, chi_eval, evaluation_point| {
                let accessor = indexmap! {
                    lhs_ref.clone().column_id() => lhs.inner_product(evaluation_point),
                    rhs_ref.clone().column_id() => rhs.inner_product(evaluation_point)
                };
                divide_and_modulo_expr
                    .verifier_evaluate(verification_builder, &accessor, chi_eval, &[])
                    .unwrap();
            },
        );
        let matrix = mock_verification_builder.get_identity_results();
        assert!(matrix.into_iter().all(|v| v.into_iter().all(|b| b)));
    }

    #[test]
    fn we_can_return_wrapped_quotient_and_remainder_columns() {
        let alloc = Bump::new();
        let (divide_and_modulo_expr, _lhs_ref, _rhs_ref, lhs_ident, rhs_ident) =
            divide_and_modulo_expr();
        let lhs = &[i128::MIN, 7, -7, 2];
        let rhs = &[-1i128, 0, 3, -4];
        let mut final_round_builder = FinalRoundBuilder::new(lhs.len(), VecDeque::new());
        let table = Table::try_new(indexmap! {
            lhs_ident => Column::Int128::<TestScalar>(lhs),
            rhs_ident => Column::Int128::<TestScalar>(rhs),
        })
        .unwrap();

        let (quotient_wrapped, remainder) = divide_and_modulo_expr
            .final_round_evaluate(&mut final_round_builder, &alloc, &table, &[])
            .unwrap();

        let expected_quotient = [i128::MIN, 0, -2, 0];
        let expected_remainder = [0, 7, -1, 2];
        assert_eq!(quotient_wrapped, Column::Int128(&expected_quotient));
        assert_eq!(remainder, Column::Int128(&expected_remainder));
        assert_eq!(final_round_builder.pcs_proof_mles().len(), 2);
    }

    #[test]
    fn we_can_return_verifier_quotient_and_remainder_evaluations() {
        let (divide_and_modulo_expr, lhs_ref, rhs_ref, _lhs_ident, _rhs_ident) =
            divide_and_modulo_expr();
        let mut verification_builder =
            mock_verification_builder(vec![vec![TestScalar::ONE, TestScalar::TWO]]);
        let accessor = indexmap! {
            lhs_ref.column_id() => TestScalar::from(7u64),
            rhs_ref.column_id() => TestScalar::from(3u64),
        };

        let (quotient_wrapped, remainder) = divide_and_modulo_expr
            .verifier_evaluate(&mut verification_builder, &accessor, TestScalar::ONE, &[])
            .unwrap();

        assert_eq!(quotient_wrapped, TestScalar::ONE);
        assert_eq!(remainder, TestScalar::TWO);
    }

    #[test]
    fn we_can_error_if_verifier_accessor_is_missing_a_column() {
        let (divide_and_modulo_expr, lhs_ref, _rhs_ref, _lhs_ident, _rhs_ident) =
            divide_and_modulo_expr();
        let mut verification_builder = mock_verification_builder(Vec::new());
        let err = divide_and_modulo_expr
            .verifier_evaluate(
                &mut verification_builder,
                &indexmap! {},
                TestScalar::ONE,
                &[],
            )
            .unwrap_err();
        assert!(matches!(
            err,
            ProofError::VerificationError {
                error: "Column Not Found"
            }
        ));

        let mut verification_builder = mock_verification_builder(Vec::new());
        let accessor = indexmap! {
            lhs_ref.column_id() => TestScalar::from(7u64),
        };
        let err = divide_and_modulo_expr
            .verifier_evaluate(&mut verification_builder, &accessor, TestScalar::ONE, &[])
            .unwrap_err();
        assert!(matches!(
            err,
            ProofError::VerificationError {
                error: "Column Not Found"
            }
        ));
    }

    #[test]
    fn we_can_error_if_verifier_has_too_few_final_round_evaluations() {
        let (divide_and_modulo_expr, lhs_ref, rhs_ref, _lhs_ident, _rhs_ident) =
            divide_and_modulo_expr();
        let mut verification_builder = mock_verification_builder(vec![vec![TestScalar::ONE]]);
        let accessor = indexmap! {
            lhs_ref.column_id() => TestScalar::from(7u64),
            rhs_ref.column_id() => TestScalar::from(3u64),
        };

        let err = divide_and_modulo_expr
            .verifier_evaluate(&mut verification_builder, &accessor, TestScalar::ONE, &[])
            .unwrap_err();

        assert!(matches!(
            err,
            ProofError::ProofSizeMismatch {
                source: ProofSizeMismatch::TooFewMLEEvaluations
            }
        ));
    }
}
