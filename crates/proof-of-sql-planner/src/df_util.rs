use arrow::datatypes::{DataType, Field, Schema};
use datafusion::{
    catalog::TableReference,
    common::{Column, DFSchema},
    logical_expr::Expr,
};

/// Create a `Expr::Column` from full table name and column
pub(crate) fn df_column(table_name: &str, column: &str) -> Expr {
    Expr::Column(Column::new(
        Some(TableReference::from(table_name)),
        column.to_string(),
    ))
}

/// Create a `DFSchema` from table name, column name and data type pairs
///
/// Note that nulls are not allowed in the schema
pub(crate) fn df_schema(table_name: &str, pairs: Vec<(&str, DataType)>) -> DFSchema {
    let arrow_schema = Schema::new(
        pairs
            .into_iter()
            .map(|(name, data_type)| Field::new(name, data_type, false))
            .collect::<Vec<_>>(),
    );
    DFSchema::try_from_qualified_schema(table_name, &arrow_schema).unwrap()
}

#[cfg(test)]
mod tests {
    use super::{df_column, df_schema};
    use arrow::datatypes::DataType;
    use datafusion::logical_expr::Expr;

    #[test]
    fn df_column_returns_column_expr() {
        let expr = df_column("schema.table", "col");
        assert!(matches!(expr, Expr::Column(_)));
    }

    #[test]
    fn df_column_contains_column_name() {
        let expr = df_column("myschema.mytable", "mycolumn");
        if let Expr::Column(col) = expr {
            assert_eq!(col.name, "mycolumn");
        } else {
            panic!("expected Expr::Column");
        }
    }

    #[test]
    fn df_schema_creates_schema_with_correct_column_count() {
        let schema = df_schema("t", vec![("a", DataType::Int64), ("b", DataType::Boolean)]);
        assert_eq!(schema.fields().len(), 2);
    }

    #[test]
    fn df_schema_preserves_column_names() {
        let schema = df_schema("t", vec![("myfield", DataType::Utf8)]);
        let field_names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();
        assert!(field_names.iter().any(|n| n.contains("myfield")));
    }

    #[test]
    fn df_schema_empty_pairs_creates_empty_schema() {
        let schema = df_schema("t", vec![]);
        assert_eq!(schema.fields().len(), 0);
    }
}
