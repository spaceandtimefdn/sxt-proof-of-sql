//! Tests for ProofPlan trait.

#[cfg(test)]
mod proof_plan_test {
    use crate::sql::proof::ProofPlan;
    use crate::sql::proof::ProverEvaluate;
    use crate::base::database::TableRef;
    use crate::base::map::IndexSet;

    #[test]
    fn test_proof_plan_trait_exists() {
        // Verify ProofPlan trait exists and can be referenced
        let _: Option<&dyn ProofPlan> = None;
    }

    #[test]
    fn test_prover_evaluate_trait_exists() {
        // Verify ProverEvaluate trait exists and can be referenced
        let _: Option<&dyn ProverEvaluate> = None;
    }

    #[test]
    fn test_table_ref_default() {
        let table_ref = TableRef::from_names(None, "test");
        let refs = IndexSet::default();
        let _ = refs;
        assert_eq!(table_ref.table_name(), "test");
    }
}