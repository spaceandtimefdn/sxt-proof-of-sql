//! Tests for `DynProofPlan` proof plan.

#[cfg(test)]
mod dyn_proof_plan_tests {
    use crate::sql::proof_plans::{DynProofPlan, EmptyExec, TableExec};
    use crate::base::database::{ColumnField, ColumnType, TableRef};
    use alloc::string::ToString;

    #[test]
    fn test_dyn_proof_plan_debug() {
        let plan = DynProofPlan::new_empty();
        let debug_str = format!("{:?}", plan);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_dyn_proof_plan_partial_eq() {
        let plan1 = DynProofPlan::new_empty();
        let plan2 = DynProofPlan::new_empty();
        assert_eq!(plan1, plan2);
    }

    #[test]
    fn test_dyn_proof_plan_clone() {
        let plan = DynProofPlan::new_empty();
        let cloned = plan.clone();
        assert_eq!(plan, cloned);
    }

    #[test]
    fn test_dyn_proof_plan_new_empty() {
        let plan = DynProofPlan::new_empty();
        match plan {
            DynProofPlan::Empty(_) => {}
            _ => panic!("Expected Empty variant"),
        }
    }

    #[test]
    fn test_dyn_proof_plan_new_table() {
        let table_ref = TableRef::from_names(None, "test_table");
        let schema = vec![ColumnField::new("col1".parse().unwrap(), ColumnType::BigInt)];
        let plan = DynProofPlan::new_table(table_ref, schema);
        match plan {
            DynProofPlan::Table(_) => {}
            _ => panic!("Expected Table variant"),
        }
    }

    #[test]
    fn test_dyn_proof_plan_serialize_empty() {
        let plan = DynProofPlan::new_empty();
        let serialized = serde_json::to_string(&plan).unwrap();
        assert!(!serialized.is_empty());
        let deserialized: DynProofPlan = serde_json::from_str(&serialized).unwrap();
        assert_eq!(plan, deserialized);
    }

    #[test]
    fn test_dyn_proof_plan_serialize_table() {
        let table_ref = TableRef::from_names(None, "test_table");
        let schema = vec![ColumnField::new("col1".parse().unwrap(), ColumnType::BigInt)];
        let plan = DynProofPlan::new_table(table_ref, schema.clone());
        let serialized = serde_json::to_string(&plan).unwrap();
        assert!(!serialized.is_empty());
        let deserialized: DynProofPlan = serde_json::from_str(&serialized).unwrap();
        assert_eq!(plan, deserialized);
    }

    #[test]
    fn test_dyn_proof_plan_get_column_result_fields_as_references() {
        let plan = DynProofPlan::new_empty();
        let refs = plan.get_column_result_fields_as_references();
        assert!(refs.is_empty());
    }

    #[test]
    fn test_dyn_proof_plan_to_string() {
        let plan = DynProofPlan::new_empty();
        let s = plan.to_string();
        assert!(!s.is_empty());
    }
}