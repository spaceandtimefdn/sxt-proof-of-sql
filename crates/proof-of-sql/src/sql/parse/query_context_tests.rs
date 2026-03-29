/// Tests for QueryContext covering edge cases and error paths
#[cfg(test)]
mod tests {
    use crate::sql::parse::{ConversionError, QueryContext};
    use proof_of_sql_parser::Identifier;

    #[test]
    fn test_query_context_new_is_empty() {
        let ctx = QueryContext::default();
        assert!(ctx.get_aliased_columns().is_empty());
    }

    #[test]
    fn test_query_context_display_conversion_error() {
        let err = ConversionError::MissingColumn {
            identifier: Box::new("col1".parse::<Identifier>().unwrap()),
            resource_id: Box::new(
                "schema.table"
                    .parse::<proof_of_sql_parser::ResourceId>()
                    .unwrap(),
            ),
        };
        let s = err.to_string();
        assert!(s.contains("col1"));
    }
}
