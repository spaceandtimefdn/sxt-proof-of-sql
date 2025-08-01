use super::{verification_builder::VerificationBuilder, FinalRoundBuilder, FirstRoundBuilder};
use crate::base::{
    database::{
        ColumnField, ColumnRef, LiteralValue, OwnedTable, Table, TableEvaluation, TableRef,
    },
    map::{IndexMap, IndexSet},
    proof::{PlaceholderResult, ProofError},
    scalar::Scalar,
};
use alloc::vec::Vec;
use bumpalo::Bump;
use core::fmt::Debug;
use sqlparser::ast::Ident;

/// Provable nodes in the provable AST.
#[enum_dispatch::enum_dispatch(DynProofPlan)]
pub trait ProofPlan: Debug + Send + Sync + ProverEvaluate {
    /// Form components needed to verify and proof store into `VerificationBuilder`
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<TableRef, IndexMap<Ident, S>>,
        result: Option<&OwnedTable<S>>,
        chi_eval_map: &IndexMap<TableRef, (S, usize)>,
        params: &[LiteralValue],
    ) -> Result<TableEvaluation<S>, ProofError>;

    /// Return all the result column fields
    fn get_column_result_fields(&self) -> Vec<ColumnField>;

    /// Return all the columns referenced in the Query
    fn get_column_references(&self) -> IndexSet<ColumnRef>;

    /// Return all the tables referenced in the Query
    fn get_table_references(&self) -> IndexSet<TableRef>;
}

#[enum_dispatch::enum_dispatch(DynProofPlan)]
pub trait ProverEvaluate {
    /// Evaluate the query, modify `FirstRoundBuilder` and return the result.
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>>;

    /// Evaluate the query and modify `FinalRoundBuilder` to store an intermediate representation
    /// of the query result and track all the components needed to form the query's proof.
    ///
    /// Intermediate values that are needed to form the proof are allocated into the arena
    /// allocator alloc. These intermediate values will persist through proof creation and
    /// will be bulk deallocated once the proof is formed.
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>>;
}

/// Marker used as a trait bound for generic [`ProofPlan`] types to indicate the honesty of their implementation.
///
/// This allows us to define alternative prover implementations that misbehave, and test that the verifier rejects their results.
pub trait ProverHonestyMarker: Debug + Send + Sync + PartialEq + 'static {}

/// [`ProverHonestyMarker`] for generic [`ProofPlan`] types whose implementation is canonical/honest.
#[derive(Debug, PartialEq, Clone)]
pub struct HonestProver;
impl ProverHonestyMarker for HonestProver {}
