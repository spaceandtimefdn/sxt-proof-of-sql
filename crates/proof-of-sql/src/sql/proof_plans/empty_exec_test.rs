//! Tests for `EmptyExec` proof plan.

#[cfg(test)]
mod empty_exec_tests {
    use crate::sql::proof_plans::EmptyExec;
    use alloc::string::ToString;

    #[test]
    fn test_empty_exec_debug() {
        let empty = EmptyExec::new();
        let debug_str = format!("{:?}", empty);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_empty_exec_partial_eq() {
        let empty1 = EmptyExec::new();
        let empty2 = EmptyExec::new();
        assert_eq!(empty1, empty2);
    }

    #[test]
    fn test_empty_exec_clone() {
        let empty = EmptyExec::new();
        let cloned = empty.clone();
        assert_eq!(empty, cloned);
    }

    #[test]
    fn test_empty_exec_default() {
        let default_empty = EmptyExec::default();
        let new_empty = EmptyExec::new();
        assert_eq!(default_empty, new_empty);
    }

    #[test]
    fn test_empty_exec_new() {
        let empty = EmptyExec::new();
        let _ = empty;
    }

    #[test]
    fn test_empty_exec_serialize() {
        let empty = EmptyExec::new();
        let serialized = serde_json::to_string(&empty).unwrap();
        assert!(!serialized.is_empty());
        let deserialized: EmptyExec = serde_json::from_str(&serialized).unwrap();
        assert_eq!(empty, deserialized);
    }

    #[test]
    fn test_empty_exec_get_column_result_fields() {
        let empty = EmptyExec::new();
        let fields = empty.get_column_result_fields();
        assert!(fields.is_empty());
    }

    #[test]
    fn test_empty_exec_get_column_references() {
        let empty = EmptyExec::new();
        let refs = empty.get_column_references();
        assert!(refs.is_empty());
    }

    #[test]
    fn test_empty_exec_get_table_references() {
        let empty = EmptyExec::new();
        let tables = empty.get_table_references();
        assert!(tables.is_empty());
    }

    #[test]
    fn test_empty_exec_to_string() {
        let empty = EmptyExec::new();
        let s = empty.to_string();
        assert_eq!(s, "EmptyExec");
    }
}