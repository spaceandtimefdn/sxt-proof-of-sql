/// Tests for QueryContext – SQL parse-level context builder
#[cfg(test)]
mod tests {
    use crate::sql::parse::QueryContext;
    use proof_of_sql_parser::intermediate_ast::BinaryOperator;

    // We construct a minimal QueryContext and verify basic bookkeeping.

    #[test]
    fn test_new_query_context_has_no_tables() {
        let ctx = QueryContext::default();
        assert!(ctx.get_any_result_column_fields().is_empty());
    }

    #[test]
    fn test_push_and_pop_table_ref() {
        let mut ctx = QueryContext::default();
        // push a context table
        ctx.push_table_factor("schema.users").unwrap();
        // There should be one table alias now.
        assert_eq!(ctx.table_count(), 1);
    }

    #[test]
    fn test_duplicate_table_ref_returns_error() {
        let mut ctx = QueryContext::default();
        ctx.push_table_factor("schema.users").unwrap();
        let result = ctx.push_table_factor("schema.users");
        assert!(
            result.is_err(),
            "Registering the same table twice should fail"
        );
    }

    #[test]
    fn test_set_where_clause_expression() {
        let mut ctx = QueryContext::default();
        ctx.push_table_factor("ns.tbl").unwrap();
        // Setting a trivial WHERE clause should succeed.
        ctx.set_where_expr(Some(proof_of_sql_parser::intermediate_ast::Expression::Literal(
            proof_of_sql_parser::intermediate_ast::Literal::Boolean(true),
        )));
        assert!(ctx.where_expr().is_some());
    }

    #[test]
    fn test_result_columns_are_ordered() {
        let mut ctx = QueryContext::default();
        ctx.push_table_factor("ns.tbl").unwrap();
        ctx.push_result_column_field("col_a", proof_of_sql_parser::intermediate_ast::ColumnType::BigInt)
            .unwrap();
        ctx.push_result_column_field("col_b", proof_of_sql_parser::intermediate_ast::ColumnType::VarChar)
            .unwrap();

        let fields = ctx.get_any_result_column_fields();
        assert_eq!(fields.len(), 2);
        // The first pushed column comes first.
        assert_eq!(fields[0].name().as_str(), "col_a");
        assert_eq!(fields[1].name().as_str(), "col_b");
    }
}
