use crate::{
    base::{
        database::{ColumnRef, CommitmentAccessor, DataAccessor},
        proof::ProofError,
        scalar::ArkScalar,
    },
    sql::{
        ast::BoolExpr,
        proof::{CountBuilder, ProofBuilder, SumcheckSubpolynomialType, VerificationBuilder},
    },
};
use bumpalo::Bump;
use curve25519_dalek::ristretto::RistrettoPoint;
use dyn_partial_eq::DynPartialEq;
use num_traits::One;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Provable logical OR expression
#[derive(Debug, DynPartialEq, PartialEq, Serialize, Deserialize)]
pub struct OrExpr {
    lhs: Box<dyn BoolExpr>,
    rhs: Box<dyn BoolExpr>,
}

impl OrExpr {
    /// Create logical OR expression
    pub fn new(lhs: Box<dyn BoolExpr>, rhs: Box<dyn BoolExpr>) -> Self {
        Self { lhs, rhs }
    }
}

#[typetag::serde]
impl BoolExpr for OrExpr {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        self.lhs.count(builder)?;
        self.rhs.count(builder)?;
        count_or(builder);
        Ok(())
    }

    fn result_evaluate<'a>(
        &self,
        table_length: usize,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<ArkScalar>,
    ) -> &'a [bool] {
        let lhs = self.lhs.result_evaluate(table_length, alloc, accessor);
        let rhs = self.rhs.result_evaluate(table_length, alloc, accessor);
        result_evaluate_or(table_length, alloc, lhs, rhs)
    }

    #[tracing::instrument(
        name = "proofs.sql.ast.or_expr.prover_evaluate",
        level = "info",
        skip_all
    )]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<ArkScalar>,
    ) -> &'a [bool] {
        let lhs = self.lhs.prover_evaluate(builder, alloc, accessor);
        let rhs = self.rhs.prover_evaluate(builder, alloc, accessor);
        return prover_evaluate_or(builder, alloc, lhs, rhs);
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        accessor: &dyn CommitmentAccessor<RistrettoPoint>,
    ) -> Result<ArkScalar, ProofError> {
        let lhs = self.lhs.verifier_evaluate(builder, accessor)?;
        let rhs = self.rhs.verifier_evaluate(builder, accessor)?;

        Ok(verifier_evaluate_or(builder, &lhs, &rhs))
    }

    fn get_column_references(&self, columns: &mut HashSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}

pub fn result_evaluate_or<'a>(
    table_length: usize,
    alloc: &'a Bump,
    lhs: &[bool],
    rhs: &[bool],
) -> &'a [bool] {
    assert_eq!(table_length, lhs.len());
    assert_eq!(table_length, rhs.len());
    alloc.alloc_slice_fill_with(table_length, |i| lhs[i] || rhs[i])
}

pub fn prover_evaluate_or<'a>(
    builder: &mut ProofBuilder<'a>,
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
            (ArkScalar::one(), vec![Box::new(lhs_and_rhs)]),
            (-ArkScalar::one(), vec![Box::new(lhs), Box::new(rhs)]),
        ],
    );

    // selection
    alloc.alloc_slice_fill_with(n, |i| lhs[i] || rhs[i])
}

pub fn verifier_evaluate_or(
    builder: &mut VerificationBuilder,
    lhs: &ArkScalar,
    rhs: &ArkScalar,
) -> ArkScalar {
    // lhs_and_rhs
    let lhs_and_rhs = builder.consume_intermediate_mle();

    // subpolynomial: lhs_and_rhs - lhs * rhs
    let eval = builder.mle_evaluations.random_evaluation * (lhs_and_rhs - *lhs * *rhs);
    builder.produce_sumcheck_subpolynomial_evaluation(&eval);

    // selection
    *lhs + *rhs - lhs_and_rhs
}

pub fn count_or(builder: &mut CountBuilder) {
    builder.count_subpolynomials(1);
    builder.count_intermediate_mles(1);
    builder.count_degree(3);
}
