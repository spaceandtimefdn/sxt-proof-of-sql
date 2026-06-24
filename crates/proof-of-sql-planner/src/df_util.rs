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
    fn df_column_returns_column_expr_with_table_qualifier() {
        let expr = df_column("my_table", "my_col");
        match expr {
            Expr::Column(col) => {
                assert_eq!(col.name, "my_col");
                assert_eq!(
                    col.relation.as_ref().map(|r| r.to_string()),
                    Some("my_table".to_string())
                );
            }
            other => panic!("Expected Expr::Column, got {other:?}"),
        }
    }

    #[test]
    fn df_column_works_with_different_names() {
        let expr = df_column("schema.employees", "salary");
        match expr {
            Expr::Column(col) => {
                assert_eq!(col.name, "salary");
                assert_eq!(
                    col.relation.as_ref().map(|r| r.to_string()),
                    Some("schema.employees".to_string())
                );
            }
            other => panic!("Expected Expr::Column, got {other:?}"),
        }
    }

    #[test]
    fn df_column_with_uppercase_names() {
        let expr = df_column("PUBLIC.ORDERS", "ORDER_ID");
        match expr {
            Expr::Column(col) => {
                assert_eq!(col.name, "ORDER_ID");
                assert_eq!(
                    col.relation.as_ref().map(|r| r.to_string()),
                    Some("PUBLIC.ORDERS".to_string())
                );
            }
            other => panic!("Expected Expr::Column, got {other:?}"),
        }
    }

    #[test]
    fn df_schema_creates_schema_with_correct_fields() {
        let schema = df_schema(
            "test_table",
            vec![
                ("id", DataType::Int64),
                ("name", DataType::Utf8),
                ("active", DataType::Boolean),
            ],
        );
        assert_eq!(schema.fields().len(), 3);
        assert_eq!(schema.field_names(), vec!["test_table.id", "test_table.name", "test_table.active"]);
    }

    #[test]
    fn df_schema_preserves_column_order() {
        let schema = df_schema(
            "t",
            vec![
                ("z", DataType::Int32),
                ("a", DataType::Int32),
                ("m", DataType::Int32),
            ],
        );
        let names: Vec<_> = schema.field_names();
        assert_eq!(names, vec!["t.z", "t.a", "t.m"]);
    }

    #[test]
    fn df_schema_empty_returns_empty_schema() {
        let schema = df_schema("empty_table", vec![]);
        assert_eq!(schema.fields().len(), 0);
    }

    #[test]
    fn df_schema_single_boolean_column() {
        let schema = df_schema("flags", vec![("is_active", DataType::Boolean)]);
        assert_eq!(schema.fields().len(), 1);
        assert_eq!(schema.field_names(), vec!["flags.is_active"]);
    }
}
