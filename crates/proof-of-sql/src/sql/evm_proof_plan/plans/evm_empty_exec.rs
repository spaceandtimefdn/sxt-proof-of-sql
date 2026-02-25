use crate::{
    base::{
        database::{Column, LiteralValue, TableEvaluation},
        proof::ProofError,
        scalar::Scalar,
        PlaceholderResult,
    },
    sql::{
        proof::{
            FinalRoundBuilder, FirstRoundBuilder, StreamlinedProoPlan, StreamlinedProverEvaluate,
            VerificationBuilder,
        },
        proof_plans::EmptyExec,
    },
};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

/// Represents a empty execution plan in EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMEmptyExec {}

impl EVMEmptyExec {
    /// Create a `EVMEmptyExec` from a `EmptyExec`.
    pub(crate) fn try_from_proof_plan(_plan: &EmptyExec) -> Self {
        Self {}
    }

    /// Convert into a proof plan
    pub(crate) fn try_into_proof_plan() -> EmptyExec {
        EmptyExec::new()
    }
}

impl StreamlinedProoPlan for EVMEmptyExec {
    fn verifier_evaluate<S: Scalar>(
        &self,
        _builder: &mut impl VerificationBuilder<S>,
        _accessor: &Vec<S>,
        _chi_eval_map: &Vec<(S, usize)>,
        _params: &[LiteralValue],
    ) -> Result<TableEvaluation<S>, ProofError> {
        unimplemented!()
    }
}

impl StreamlinedProverEvaluate for EVMEmptyExec {
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        _builder: &mut FirstRoundBuilder<'a, S>,
        _alloc: &'a Bump,
        _column_map: &Vec<Column<'a, S>>,
        _table_length_lookup: Vec<usize>,
        _params: &[LiteralValue],
    ) -> PlaceholderResult<(Vec<Column<'a, S>>, usize)> {
        unimplemented!()
    }

    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        _builder: &mut FinalRoundBuilder<'a, S>,
        _alloc: &'a Bump,
        _column_map: &Vec<Column<'a, S>>,
        _table_length_lookup: Vec<usize>,
        _params: &[LiteralValue],
    ) -> PlaceholderResult<(Vec<Column<'a, S>>, usize)> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use crate::sql::{evm_proof_plan::plans::EVMEmptyExec, proof_plans::EmptyExec};

    #[test]
    fn we_can_put_empty_exec_in_evm() {
        let empty_exec = EmptyExec::new();

        // Roundtrip
        let roundtripped_empty_exec = EVMEmptyExec::try_into_proof_plan();
        assert_eq!(roundtripped_empty_exec, empty_exec);
    }
}
