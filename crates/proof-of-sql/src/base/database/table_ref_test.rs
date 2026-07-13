//! Tests for TableRef.

#[cfg(test)]
mod table_ref_test {
    use crate::base::database::TableRef;
    use alloc::string::ToString;

    #[test]
    fn test_table_ref_from_names() {
        let table_ref = TableRef::from_names(Some("namespace".to_string()), "table_name");
        assert_eq!(table_ref.namespace(), Some("namespace"));
        assert_eq!(table_ref.table_name(), "table_name");
    }

    #[test]
    fn test_table_ref_from_names_none() {
        let table_ref = TableRef::from_names(None, "simple_table");
        assert_eq!(table_ref.namespace(), None);
        assert_eq!(table_ref.table_name(), "simple_table");
    }

    #[test]
    fn test_table_ref_parse() {
        let table_ref: TableRef = "namespace.table_name".parse().unwrap();
        assert_eq!(table_ref.namespace(), Some("namespace"));
        assert_eq!(table_ref.table_name(), "table_name");
    }

    #[test]
    fn test_table_ref_parse_simple() {
        let table_ref: TableRef = "simple_table".parse().unwrap();
        assert_eq!(table_ref.namespace(), None);
        assert_eq!(table_ref.table_name(), "simple_table");
    }

    #[test]
    fn test_table_ref_display() {
        let table_ref = TableRef::from_names(Some("ns".to_string()), "tbl");
        assert_eq!(table_ref.to_string(), "ns.tbl");
    }

    #[test]
    fn test_table_ref_display_no_namespace() {
        let table_ref = TableRef::from_names(None, "tbl");
        assert_eq!(table_ref.to_string(), "tbl");
    }

    #[test]
    fn test_table_ref_clone() {
        let table_ref = TableRef::from_names(Some("ns".to_string()), "tbl");
        let cloned = table_ref.clone();
        assert_eq!(table_ref, cloned);
    }

    #[test]
    fn test_table_ref_debug() {
        let table_ref = TableRef::from_names(Some("ns".to_string()), "tbl");
        let debug_str = format!("{:?}", table_ref);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_table_ref_partial_eq() {
        let ref1 = TableRef::from_names(Some("ns".to_string()), "tbl");
        let ref2 = TableRef::from_names(Some("ns".to_string()), "tbl");
        let ref3 = TableRef::from_names(Some("ns".to_string()), "other");
        assert_eq!(ref1, ref2);
        assert_ne!(ref1, ref3);
    }

    #[test]
    fn test_table_ref_hash() {
        use core::hash::{Hash, Hasher};
        let ref1 = TableRef::from_names(Some("ns".to_string()), "tbl");
        let ref2 = TableRef::from_names(Some("ns".to_string()), "tbl");
        let mut h1 = std::collections::hash_map::DefaultHasher::new();
        let mut h2 = std::collections::hash_map::DefaultHasher::new();
        ref1.hash(&mut h1);
        ref2.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }
}