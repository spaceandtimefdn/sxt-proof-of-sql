use crate::sql::proof_plans::EmptyExec;
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

#[cfg(test)]
mod tests{
    use crate::sql::{evm_proof_plan::plans::EVMEmptyExec, proof_plans::EmptyExec};


    #[test]
    fn we_can_put_empty_exec_in_evm() {
        let empty_exec = EmptyExec::new();

        // Roundtrip
        let roundtripped_empty_exec = EVMEmptyExec::try_into_proof_plan();
        assert_eq!(roundtripped_empty_exec, empty_exec);
    }

}
