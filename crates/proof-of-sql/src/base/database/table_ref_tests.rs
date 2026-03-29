/// Tests for TableRef to improve coverage of display, parsing, and equality
#[cfg(test)]
mod tests {
    use crate::base::database::TableRef;
    use proof_of_sql_parser::Identifier;

    fn make_table_ref(schema: &str, name: &str) -> TableRef {
        let resource_id = format!("{schema}.{name}").parse().unwrap();
        TableRef::new(resource_id)
    }

    #[test]
    fn test_table_ref_schema_and_name() {
        let t = make_table_ref("myschema", "mytable");
        assert_eq!(t.schema_id().name(), "myschema");
        assert_eq!(t.table_id().name(), "mytable");
    }

    #[test]
    fn test_table_ref_display() {
        let t = make_table_ref("sxt", "test");
        let s = t.to_string();
        assert!(s.contains("sxt"));
        assert!(s.contains("test"));
    }

    #[test]
    fn test_table_ref_equality() {
        let t1 = make_table_ref("sxt", "test");
        let t2 = make_table_ref("sxt", "test");
        assert_eq!(t1, t2);
    }

    #[test]
    fn test_table_ref_inequality() {
        let t1 = make_table_ref("sxt", "test");
        let t2 = make_table_ref("sxt", "other");
        assert_ne!(t1, t2);
    }

    #[test]
    fn test_table_ref_parse() {
        let t: TableRef = "sxt.employees".parse().unwrap();
        assert_eq!(t.table_id().name(), "employees");
        assert_eq!(t.schema_id().name(), "sxt");
    }

    #[test]
    fn test_table_ref_clone() {
        let t1 = make_table_ref("sxt", "test");
        let t2 = t1.clone();
        assert_eq!(t1, t2);
    }

    #[test]
    fn test_table_ref_hash() {
        use std::collections::HashSet;
        let t1 = make_table_ref("sxt", "test");
        let t2 = make_table_ref("sxt", "test");
        let t3 = make_table_ref("sxt", "other");
        let mut set = HashSet::new();
        set.insert(t1);
        assert!(set.contains(&t2));
        assert!(!set.contains(&t3));
    }
}
