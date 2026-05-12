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
    use datafusion::{catalog::TableReference, common::Column, logical_expr::Expr};

    #[test]
    fn we_can_create_a_datafusion_column_expression() {
        assert_eq!(
            df_column("namespace.table", "a"),
            Expr::Column(Column::new(
                Some(TableReference::from("namespace.table")),
                "a"
            ))
        );
    }

    #[test]
    fn we_can_create_a_qualified_datafusion_schema() {
        let schema = df_schema("table", vec![("a", DataType::Int32), ("b", DataType::Utf8)]);
        let fields = schema.fields();
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name(), "a");
        assert_eq!(fields[0].data_type(), &DataType::Int32);
        assert_eq!(fields[1].name(), "b");
        assert_eq!(fields[1].data_type(), &DataType::Utf8);
    }
}
