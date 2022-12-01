use super::{ProofBuilder, ProofCounts, VerificationBuilder};
use arrow::datatypes::SchemaRef;

use crate::base::database::{CommitmentAccessor, DataAccessor, MetadataAccessor};

use bumpalo::Bump;
use std::fmt::Debug;

/// A query expression that we can evaluate, prove, and verify
pub trait QueryExpr: Debug + Send + Sync {
    /// Count terms used within the Query's proof
    fn count(&self, counts: &mut ProofCounts, accessor: &dyn MetadataAccessor);

    /// Evaluate the query and modify `ProofBuilder` to store an intermediate representation
    /// of the query result and track all the components needed to form the query's proof.
    ///
    /// Intermediate values that are needed to form the proof are allocated into the arena
    /// allocator alloc. These intermediate values will persist through proof creation and
    /// will be bulk deallocated once the proof is formed.
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        counts: &ProofCounts,
        accessor: &'a dyn DataAccessor,
    );

    /// Form components needed to verify and proof store into VerificationBuilder
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        counts: &ProofCounts,
        accessor: &dyn CommitmentAccessor,
    );

    /// Return the Schema of the Query’s result
    fn get_result_schema(&self) -> SchemaRef;
}
