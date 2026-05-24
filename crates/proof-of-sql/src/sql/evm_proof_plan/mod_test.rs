#[cfg(test)]
mod evm_proof_plan_mod_test {
    #[test]
    fn test_module_re_exports() {
        use crate::sql::evm_proof_plan::{EVMProofPlan, EVMDynProofExpr, EVMProofPlanError};
        let _ = EVMProofPlanError::default();
    }
}
