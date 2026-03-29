#[cfg(test)]
mod tests {
    use crate::sql::parse::QueryContext;

    // -----------------------------------------------------------------------
    // QueryContext construction and basic accessors
    // -----------------------------------------------------------------------

    #[test]
    fn test_default_query_context_has_no_columns() {
        let ctx = QueryContext::default();
        assert!(ctx.get_any_result_column_fields().is_empty());
    }

    #[test]
    fn test_query_context_toggle_result_scope() {
        let mut ctx = QueryContext::default();
        // The result-column scope must start inactive.
        assert!(!ctx.is_in_result_scope());
        ctx.toggle_result_scope();
        assert!(ctx.is_in_result_scope());
        ctx.toggle_result_scope();
        assert!(!ctx.is_in_result_scope());
    }

    #[test]
    fn test_query_context_set_and_get_table_ref() {
        use proof_of_sql_parser::Identifier;
        use sqlparser::ast::ObjectName;

        let mut ctx = QueryContext::default();
        let name: ObjectName = "namespace.tbl".parse().unwrap();
        ctx.set_table_ref(name);
        let table_ref = ctx.get_table_ref();
        // The resource ID should reflect the parsed table name.
        assert!(
            table_ref.resource_id().object_name().to_string().contains("tbl"),
            "unexpected table ref: {:?}", table_ref
        );
    }
}
