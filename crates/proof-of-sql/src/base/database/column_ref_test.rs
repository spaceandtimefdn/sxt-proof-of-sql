#[cfg(test)]
mod tests {
    use crate::base::database::ColumnRef;
    use crate::base::database::{ColumnType, TableRef};
    use sqlparser::ast::Ident;

    #[test]
    fn we_can_create_column_ref_and_access_fields() {
        let table_ref = TableRef::new("sxt", "users");
        let column_id = Ident::new("email");
        let column_type = ColumnType::VarChar;
        let col_ref = ColumnRef::new(table_ref.clone(), column_id.clone(), column_type);

        assert_eq!(col_ref.table_ref(), table_ref);
        assert_eq!(col_ref.column_id(), column_id);
        assert_eq!(*col_ref.column_type(), column_type);
    }

    #[test]
    fn column_ref_works_with_numeric_types() {
        for col_type in [
            ColumnType::Boolean,
            ColumnType::Uint8,
            ColumnType::TinyInt,
            ColumnType::SmallInt,
            ColumnType::Int,
            ColumnType::BigInt,
        ] {
            let col_ref = ColumnRef::new(
                TableRef::new("", "t"),
                Ident::new("c"),
                col_type,
            );
            assert_eq!(*col_ref.column_type(), col_type);
        }
    }

    #[test]
    fn column_ref_works_without_schema() {
        let col_ref = ColumnRef::new(
            TableRef::new("", "products"),
            Ident::new("price"),
            ColumnType::BigInt,
        );
        assert_eq!(col_ref.table_ref().to_string(), "products");
    }
}
