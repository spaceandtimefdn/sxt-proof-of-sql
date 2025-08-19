use arrow::record_batch::RecordBatch;
use bumpalo::Bump;
use indexmap::IndexMap;
use proof_of_sql::base::{
    arrow::arrow_array_to_column_conversion::ArrayRefExt,
    database::{Column, ColumnType, DataAccessor, MetadataAccessor, SchemaAccessor, TableRef},
    scalar::Scalar,
};
use sqlparser::ast::Ident;
#[derive(Default)]
/// An implementation of a data accessor that uses a record batch as the underlying data source.
///
/// This type implements the `DataAccessor`, `MetadataAccessor`, and `SchemaAccessor` traits.
pub struct RecordBatchAccessor {
    alloc: Bump,
    tables: IndexMap<TableRef, RecordBatch>,
}
impl RecordBatchAccessor {
    /// Inserts a new table into the accessor.
    pub fn insert_table(&mut self, table_ref: TableRef, batch: RecordBatch) {
        self.tables.insert(table_ref, batch);
    }
}
impl<S: Scalar> DataAccessor<S> for RecordBatchAccessor {
    fn get_column(&self, table_ref: &TableRef, column_id: &Ident) -> Column<S> {
        let table = self.tables.get(table_ref).expect("Table not found.");
        let arrow_column = table
            .column_by_name(column_id.value.as_str())
            .expect("Column not found.");
        let result = arrow_column
            .to_column(&self.alloc, &(0..table.num_rows()), None)
            .expect("Failed to convert arrow column.");
        result
    }
}
impl MetadataAccessor for RecordBatchAccessor {
    fn get_length(&self, table_ref: &TableRef) -> usize {
        self.tables
            .get(&table_ref)
            .expect("Table not found.")
            .num_rows()
    }

    fn get_offset(&self, table_ref: &TableRef) -> usize {
        assert!(self.tables.contains_key(&table_ref), "Table not found.");
        0
    }
}
impl SchemaAccessor for RecordBatchAccessor {
    fn lookup_column(&self, table_ref: &TableRef, column_id: &Ident) -> Option<ColumnType> {
        self.tables
            .get(table_ref)
            .expect("Table not found.")
            .schema()
            .column_with_name(column_id.value.as_str())
            .map(|(_, f)| {
                f.data_type()
                    .clone()
                    .try_into()
                    .expect("Failed to convert data type.")
            })
    }

    fn lookup_schema(&self, table_ref: &TableRef) -> Vec<(Ident, ColumnType)> {
        self.tables
            .get(table_ref)
            .expect("Table not found.")
            .schema()
            .fields()
            .iter()
            .map(|field| {
                (
                    Ident::new(field.name()),
                    field
                        .data_type()
                        .clone()
                        .try_into()
                        .expect("Failed to convert data type."),
                )
            })
            .collect()
    }
}
