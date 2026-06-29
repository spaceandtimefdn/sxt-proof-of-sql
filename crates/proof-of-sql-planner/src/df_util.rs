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
    use super::*;

    #[test]
    fn df_column_builds_qualified_column_expr() {
        let expr = df_column("users", "id");
        let Expr::Column(column) = expr else {
            panic!("expected Expr::Column");
        };
        assert_eq!(column.name, "id");
        assert_eq!(
            column.relation.as_ref().map(|r| r.to_string()),
            Some("users".to_string())
        );
    }

    #[test]
    fn df_schema_builds_non_nullable_fields_with_expected_types() {
        let schema = df_schema(
            "orders",
            vec![("order_id", DataType::Int64), ("amount", DataType::Float64)],
        );
        assert_eq!(schema.fields().len(), 2);
        assert_eq!(schema.field(0).name(), "order_id");
        assert_eq!(schema.field(0).data_type(), &DataType::Int64);
        assert!(!schema.field(0).is_nullable());
        assert_eq!(schema.field(1).name(), "amount");
        assert_eq!(schema.field(1).data_type(), &DataType::Float64);
        assert!(!schema.field(1).is_nullable());
    }
}
