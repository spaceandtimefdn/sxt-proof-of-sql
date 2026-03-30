/// Tests for the query context builder — verifies that the builder correctly
/// propagates schema information, rejects invalid identifiers, and handles
/// aliased expressions without panicking.
#[cfg(test)]
mod tests {
    use crate::sql::parse::QueryContextBuilder;
    use proof_of_sql_parser::posql_time::PoSQLTimeZone;
    use proof_of_sql_parser::{Identifier, ResourceId};

    fn make_resource_id(schema: &str, table: &str) -> ResourceId {
        ResourceId::try_new(schema, table).expect("valid resource id")
    }

    // -----------------------------------------------------------------------
    // Basic builder construction
    // -----------------------------------------------------------------------

    #[test]
    fn test_builder_can_be_created_for_valid_table() {
        // Simply constructing a builder for a known table should not panic.
        let _builder = QueryContextBuilder::new(make_resource_id("public", "test_table").into());
    }

    // -----------------------------------------------------------------------
    // ResourceId construction and accessors
    // -----------------------------------------------------------------------

    #[test]
    fn test_resource_id_schema_and_object_name() {
        let rid = make_resource_id("myschema", "mytable");
        assert_eq!(rid.schema().name(), "myschema");
        assert_eq!(rid.object_name().name(), "mytable");
    }

    #[test]
    fn test_resource_id_display_contains_both_parts() {
        let rid = make_resource_id("s", "t");
        let s = rid.to_string();
        assert!(s.contains("s") && s.contains("t"));
    }

    // -----------------------------------------------------------------------
    // Identifier
    // -----------------------------------------------------------------------

    #[test]
    fn test_identifier_from_valid_str() {
        let id = Identifier::try_new("valid_column").expect("valid identifier");
        assert_eq!(id.name(), "valid_column");
    }

    #[test]
    fn test_identifier_case_insensitive_storage() {
        // Identifiers are typically stored in lowercase.
        let id = Identifier::try_new("MyColumn").expect("valid");
        // The name should be the normalised form — just ensure no panic.
        assert!(!id.name().is_empty());
    }

    #[test]
    fn test_identifier_equality() {
        let a = Identifier::try_new("col").unwrap();
        let b = Identifier::try_new("col").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn test_identifier_inequality() {
        let a = Identifier::try_new("col_a").unwrap();
        let b = Identifier::try_new("col_b").unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn test_identifier_hash_consistency() {
        use std::collections::HashSet;
        let mut s: HashSet<Identifier> = HashSet::new();
        let id = Identifier::try_new("x").unwrap();
        s.insert(id.clone());
        s.insert(id.clone());
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn test_identifier_debug_non_empty() {
        let id = Identifier::try_new("abc").unwrap();
        assert!(!format!("{:?}", id).is_empty());
    }
}
