use super::{
    CountBuilder, ProofBuilder, ProofExpr, ProverEvaluate, VerifiableQueryResult,
    VerificationBuilder,
};
use crate::{
    base::{
        commitment::{Commitment, InnerProductProof},
        database::{
            owned_table_utility::{bigint, owned_table},
            ColumnField, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor, MetadataAccessor,
            OwnedTable, TestAccessor, UnimplementedTestAccessor,
        },
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{QueryData, ResultBuilder},
};
use bumpalo::Bump;
use indexmap::IndexSet;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub(super) struct EmptyTestQueryExpr {
    pub(super) length: usize,
    pub(super) columns: usize,
}
impl<S: Scalar> ProverEvaluate<S> for EmptyTestQueryExpr {
    fn result_evaluate<'a>(
        &self,
        _builder: &mut ResultBuilder<'a>,
        _alloc: &'a Bump,
        _accessor: &'a dyn DataAccessor<S>,
    ) {
    }
    fn prover_evaluate<'a>(
        &self,
        _builder: &mut ProofBuilder<'a, S>,
        _alloc: &'a Bump,
        _accessor: &'a dyn DataAccessor<S>,
    ) {
    }
}
impl<C: Commitment> ProofExpr<C> for EmptyTestQueryExpr {
    fn count(
        &self,
        builder: &mut CountBuilder,
        _accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError> {
        builder.count_result_columns(1);
        Ok(())
    }
    fn get_length(&self, _accessor: &dyn MetadataAccessor) -> usize {
        self.length
    }
    fn get_offset(&self, _accessor: &dyn MetadataAccessor) -> usize {
        0
    }
    fn verifier_evaluate(
        &self,
        _builder: &mut VerificationBuilder<C>,
        _accessor: &dyn CommitmentAccessor<C>,
        _result: Option<&OwnedTable<<C as Commitment>::Scalar>>,
    ) -> Result<(), ProofError> {
        Ok(())
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        (1..=self.columns)
            .map(|i| ColumnField::new(format!("a{i}").parse().unwrap(), ColumnType::BigInt))
            .collect()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        unimplemented!("no real usage for this function yet")
    }
}

#[test]
fn we_can_verify_queries_on_an_empty_table() {
    let expr = EmptyTestQueryExpr {
        columns: 1,
        ..Default::default()
    };
    let accessor = UnimplementedTestAccessor::new_empty();
    let res = VerifiableQueryResult::<InnerProductProof>::new(&expr, &accessor, &());
    let QueryData {
        verification_hash: _,
        table,
    } = res.verify(&expr, &accessor, &()).unwrap();
    let expected_res = owned_table([bigint("a1", [0; 0])]);
    assert_eq!(table, expected_res);
}

#[test]
fn empty_verification_fails_if_the_result_contains_non_null_members() {
    let expr = EmptyTestQueryExpr {
        columns: 1,
        ..Default::default()
    };
    let accessor = UnimplementedTestAccessor::new_empty();
    let res = VerifiableQueryResult::<InnerProductProof> {
        provable_result: Some(Default::default()),
        proof: None,
    };
    assert!(res.verify(&expr, &accessor, &()).is_err());
}
