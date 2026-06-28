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
    fn test_df_column() {
        let expr = df_column("my_table", "my_column");
        if let Expr::Column(col) = expr {
            assert_eq!(col.name, "my_column");
            assert_eq!(col.relation.unwrap().to_string(), "my_table");
        } else {
            panic!("Expected Expr::Column");
        }
    }

    #[test]
    fn test_df_schema() {
        let schema = df_schema("my_table", vec![("id", DataType::Int32)]);
        assert_eq!(schema.fields().len(), 1);
        let field = schema.field(0);
        assert_eq!(field.name(), "id");
        assert_eq!(field.data_type(), &DataType::Int32);
        assert_eq!(field.is_nullable(), false);
    }
}
