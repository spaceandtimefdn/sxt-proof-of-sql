//! Tests for EVM Proof Plans.

#[cfg(test)]
mod evm_proof_plan_tests {
    use crate::sql::evm_proof_plan::{EVMEmptyExec, EVMDynProofPlan};
    use crate::base::database::{ColumnField, ColumnRef, ColumnType, TableRef};
    use crate::base::map::IndexSet;
    use alloc::string::ToString;

    #[test]
    fn test_evm_empty_exec_debug() {
        let empty = EVMEmptyExec;
        let debug_str = format!("{:?}", empty);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_evm_empty_exec_clone() {
        let empty = EVMEmptyExec;
        let cloned = empty.clone();
        assert_eq!(empty, cloned);
    }

    #[test]
    fn test_evm_dyn_proof_plan_empty_variant() {
        let empty = EVMEmptyExec;
        let plan = EVMDynProofPlan::Empty(empty);
        match plan {
            EVMDynProofPlan::Empty(_) => {}
            _ => panic!("Expected Empty variant"),
        }
    }

    #[test]
    fn test_evm_dyn_proof_plan_debug() {
        let empty = EVMEmptyExec;
        let plan = EVMDynProofPlan::Empty(empty);
        let debug_str = format!("{:?}", plan);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_evm_dyn_proof_plan_clone() {
        let empty = EVMEmptyExec;
        let plan = EVMDynProofPlan::Empty(empty);
        let cloned = plan.clone();
        match cloned {
            EVMDynProofPlan::Empty(_) => {}
            _ => panic!("Expected Empty variant after clone"),
        }
    }

    #[test]
    fn test_evm_dyn_proof_plan_serialize() {
        let empty = EVMEmptyExec;
        let plan = EVMDynProofPlan::Empty(empty);
        let serialized = serde_json::to_string(&plan).unwrap();
        assert!(!serialized.is_empty());
    }

    #[test]
    fn test_evm_dyn_proof_plan_partial_eq() {
        let empty1 = EVMDynProofPlan::Empty(EVMEmptyExec);
        let empty2 = EVMDynProofPlan::Empty(EVMEmptyExec);
        assert_eq!(empty1, empty2);
    }

    #[test]
    fn test_evm_empty_exec_to_string() {
        let empty = EVMEmptyExec;
        let s = empty.to_string();
        assert_eq!(s, "EVMEmptyExec");
    }
}