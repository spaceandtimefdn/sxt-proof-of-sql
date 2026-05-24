//! Tests for demo_mock_plan.

#[cfg(test)]
mod demo_mock_plan_test {
    use crate::sql::proof_plans::demo_mock_plan::DemoMockPlan;

    #[test]
    fn test_demo_mock_plan_type_exists() {
        let _: Option<DemoMockPlan> = None;
    }

    #[test]
    fn test_demo_mock_plan_debug() {
        let debug_str = format!("{:?}", std::any::type_name::<DemoMockPlan>());
        assert!(!debug_str.is_empty());
    }
}
