/// Tests for [`TableRef`] — parsing, equality, hashing, and Display.
#[cfg(test)]
mod tests {
    use crate::base::database::TableRef;
    use std::collections::HashSet;
    use std::str::FromStr;

    // -----------------------------------------------------------------------
    // FromStr / Display roundtrip
    // -----------------------------------------------------------------------

    #[test]
    fn test_from_str_simple_table_ref() {
        let t: TableRef = "schema.table".parse().expect("valid table ref");
        assert_eq!(t.schema_id().name(), "schema");
        assert_eq!(t.table_id().name(), "table");
    }

    #[test]
    fn test_display_roundtrips_from_str() {
        let original = "myschema.mytable";
        let t: TableRef = original.parse().expect("valid");
        assert_eq!(t.to_string(), original);
    }

    #[test]
    fn test_from_str_rejects_missing_schema() {
        // A bare identifier without a dot should fail.
        let result = TableRef::from_str("justatable");
        assert!(
            result.is_err(),
            "TableRef without schema should fail, got {:?}",
            result
        );
    }

    // -----------------------------------------------------------------------
    // Equality & hashing
    // -----------------------------------------------------------------------

    #[test]
    fn test_equal_table_refs_are_equal() {
        let a: TableRef = "s.t".parse().unwrap();
        let b: TableRef = "s.t".parse().unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn test_different_table_refs_are_not_equal() {
        let a: TableRef = "s.t1".parse().unwrap();
        let b: TableRef = "s.t2".parse().unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn test_table_ref_hash_consistency() {
        let mut set: HashSet<TableRef> = HashSet::new();
        let t: TableRef = "public.users".parse().unwrap();
        set.insert(t.clone());
        set.insert(t.clone());
        assert_eq!(set.len(), 1, "Duplicate TableRefs should hash to same slot");
    }

    #[test]
    fn test_different_table_refs_hash_differently_in_set() {
        let mut set: HashSet<TableRef> = HashSet::new();
        set.insert("a.b".parse::<TableRef>().unwrap());
        set.insert("a.c".parse::<TableRef>().unwrap());
        assert_eq!(set.len(), 2);
    }

    // -----------------------------------------------------------------------
    // Clone / Debug
    // -----------------------------------------------------------------------

    #[test]
    fn test_clone_produces_equal_table_ref() {
        let t: TableRef = "foo.bar".parse().unwrap();
        let t2 = t.clone();
        assert_eq!(t, t2);
    }

    #[test]
    fn test_debug_is_non_empty() {
        let t: TableRef = "x.y".parse().unwrap();
        assert!(!format!("{:?}", t).is_empty());
    }

    // -----------------------------------------------------------------------
    // schema_id / table_id accessors
    // -----------------------------------------------------------------------

    #[test]
    fn test_schema_id_name() {
        let t: TableRef = "production.orders".parse().unwrap();
        assert_eq!(t.schema_id().name(), "production");
    }

    #[test]
    fn test_table_id_name() {
        let t: TableRef = "production.orders".parse().unwrap();
        assert_eq!(t.table_id().name(), "orders");
    }
}
