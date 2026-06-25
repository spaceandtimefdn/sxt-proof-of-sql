#[cfg(feature = "arrow")]
use crate::base::database::{ArrayRefExt, ArrowArrayToColumnConversionError};
use crate::base::{
    database::{Column, DataAccessor, MetadataAccessor, TableRef},
    scalar::Scalar,
    IndexMap,
};
#[cfg(feature = "arrow")]
use arrow::array::RecordBatch;
use bumpalo::Bump;
use sqlparser::ast::Ident;

/// The canonical implementation for the `DataAccessor` trait
pub struct DataAccessorImpl<'a, S: Scalar> {
    data_lookup: IndexMap<TableRef, TableDataAccessor<'a, S>>,
}

impl<'a, S: Scalar> DataAccessorImpl<'a, S> {
    /// Creates a new instance of `DataAccessorImpl`
    #[must_use]
    pub fn new(data_lookup: IndexMap<TableRef, TableDataAccessor<'a, S>>) -> Self {
        Self { data_lookup }
    }
}

/// An intermediate type for use by `DataAccessorImpl`
pub struct TableDataAccessor<'a, S: Scalar> {
    offset: usize,
    table_data: IndexMap<Ident, Column<'a, S>>,
}

impl<'a, S: Scalar> TableDataAccessor<'a, S> {
    /// Creates a new instance of `TableDataAccessor`
    #[must_use]
    pub fn new(offset: usize, table_data: IndexMap<Ident, Column<'a, S>>) -> Self {
        Self { offset, table_data }
    }

    /// Creates a new instance of `TableDataAccessor` using a `RecordBatch`
    #[cfg(feature = "arrow")]
    pub fn try_from_record_batch(
        record_batch: &'a RecordBatch,
        offset: usize,
        alloc: &'a Bump,
    ) -> Result<Self, ArrowArrayToColumnConversionError> {
        let range = 0..record_batch.num_rows();
        let columns = record_batch
            .schema()
            .fields()
            .iter()
            .zip(record_batch.columns())
            .map(|(f, col)| {
                col.to_column::<S>(alloc, &range, None)
                    .map(|col| (f.name().as_str().into(), col))
            })
            // Use collect to transform Iterator<Result<T, E>> into Result<Collection<T>, E>
            .collect::<Result<IndexMap<_, _>, _>>()?;
        Ok(Self {
            offset,
            table_data: columns,
        })
    }
}

impl<S: Scalar> MetadataAccessor for DataAccessorImpl<'_, S> {
    fn get_length(&self, table_ref: &TableRef) -> usize {
        self.data_lookup
            .get(table_ref)
            .expect("table does not exist")
            .table_data
            .len()
    }

    fn get_offset(&self, table_ref: &TableRef) -> usize {
        self.data_lookup
            .get(table_ref)
            .expect("table does not exist")
            .offset
    }
}

impl<S: Scalar> DataAccessor<S> for DataAccessorImpl<'_, S> {
    fn get_column(&self, table_ref: &TableRef, column_id: &Ident) -> Column<'_, S> {
        *self
            .data_lookup
            .get(table_ref)
            .expect("table does not exist")
            .table_data
            .get(column_id)
            .expect("column does not exist")
    }
}

#[cfg(test)]
mod tests {
    use crate::base::{
        database::{
            Column, DataAccessor, DataAccessorImpl, MetadataAccessor, TableDataAccessor, TableRef,
        },
        scalar::test_scalar::TestScalar,
    };
    use alloc::sync::Arc;
    #[cfg(feature = "arrow")]
    use arrow::array::{ArrayRef, BooleanArray, RecordBatch};
    use bumpalo::Bump;
    use sqlparser::ast::Ident;
    use std::str::FromStr;

    #[test]
    fn we_can_get_offset_and_length() {
        let column_id = Ident::from("test");
        let column = Column::<TestScalar>::BigInt(&[3i64]);
        let table_data_accessor =
            TableDataAccessor::new(2, [(column_id.clone(), column)].into_iter().collect());
        let table_ref = TableRef::from_names(Some("test"), "table");
        let data_accessor = DataAccessorImpl::new(
            [(table_ref.clone(), table_data_accessor)]
                .into_iter()
                .collect(),
        );
        assert_eq!(data_accessor.get_length(&table_ref), 1);
        assert_eq!(data_accessor.get_offset(&table_ref), 2);
        assert_eq!(data_accessor.get_column(&table_ref, &column_id), column);
    }

    #[cfg(feature = "arrow")]
    #[test]
    fn we_can_get_data_accessor_from_record_batch() {
        let rb = RecordBatch::try_from_iter([(
            "BOOLS",
            Arc::new(BooleanArray::from(vec![true, false])) as ArrayRef,
        )])
        .unwrap();

        let alloc = Bump::new();
        let table_ref = TableRef::from_str("test.table").unwrap();
        let table_data_accessor =
            TableDataAccessor::<TestScalar>::try_from_record_batch(&rb, 1, &alloc).unwrap();
        let data_accessor_impl = DataAccessorImpl::new(
            [(table_ref.clone(), table_data_accessor)]
                .into_iter()
                .collect(),
        );

        assert_eq!(data_accessor_impl.get_length(&table_ref), 1);
        assert_eq!(data_accessor_impl.get_offset(&table_ref), 1);
        assert_eq!(
            data_accessor_impl.get_column(&table_ref, &Ident::new("BOOLS")),
            Column::Boolean(&[true, false])
        );
    }

    #[test]
    fn we_can_access_multiple_tables() {
        let col_a = Ident::from("col_a");
        let col_b = Ident::from("col_b");
        let column_a = Column::<TestScalar>::BigInt(&[10i64, 20]);
        let column_b = Column::<TestScalar>::SmallInt(&[1i16, 2, 3]);
        let table_ref_a = TableRef::from_names(Some("schema"), "table_a");
        let table_ref_b = TableRef::from_names(Some("schema"), "table_b");
        let accessor_a = TableDataAccessor::new(0, [(col_a.clone(), column_a)].into_iter().collect());
        let accessor_b = TableDataAccessor::new(5, [(col_b.clone(), column_b)].into_iter().collect());
        let data_accessor = DataAccessorImpl::new(
            [
                (table_ref_a.clone(), accessor_a),
                (table_ref_b.clone(), accessor_b),
            ]
            .into_iter()
            .collect(),
        );
        assert_eq!(data_accessor.get_length(&table_ref_a), 1);
        assert_eq!(data_accessor.get_offset(&table_ref_a), 0);
        assert_eq!(data_accessor.get_column(&table_ref_a, &col_a), column_a);
        assert_eq!(data_accessor.get_length(&table_ref_b), 1);
        assert_eq!(data_accessor.get_offset(&table_ref_b), 5);
        assert_eq!(data_accessor.get_column(&table_ref_b, &col_b), column_b);
    }

    #[test]
    fn we_can_access_multiple_columns_in_a_table() {
        let col_int = Ident::from("int_col");
        let col_bool = Ident::from("bool_col");
        let col_small = Ident::from("small_col");
        let column_int = Column::<TestScalar>::Int(&[1i32, 2, 3]);
        let column_bool = Column::<TestScalar>::Boolean(&[true, false, true]);
        let column_small = Column::<TestScalar>::SmallInt(&[10i16, 20, 30]);
        let table_ref = TableRef::from_names(Some("ns"), "multi");
        let table_data = TableDataAccessor::new(
            7,
            [
                (col_int.clone(), column_int),
                (col_bool.clone(), column_bool),
                (col_small.clone(), column_small),
            ]
            .into_iter()
            .collect(),
        );
        let data_accessor = DataAccessorImpl::new([(table_ref.clone(), table_data)].into_iter().collect());
        assert_eq!(data_accessor.get_length(&table_ref), 3);
        assert_eq!(data_accessor.get_offset(&table_ref), 7);
        assert_eq!(data_accessor.get_column(&table_ref, &col_int), column_int);
        assert_eq!(data_accessor.get_column(&table_ref, &col_bool), column_bool);
        assert_eq!(data_accessor.get_column(&table_ref, &col_small), column_small);
    }

    #[test]
    fn we_can_create_table_data_accessor_with_zero_offset() {
        let column_id = Ident::from("value");
        let column = Column::<TestScalar>::Int(&[42i32]);
        let table_ref = TableRef::from_names(None, "t");
        let table_accessor = TableDataAccessor::new(0, [(column_id.clone(), column)].into_iter().collect());
        let data_accessor = DataAccessorImpl::new([(table_ref.clone(), table_accessor)].into_iter().collect());
        assert_eq!(data_accessor.get_offset(&table_ref), 0);
        assert_eq!(data_accessor.get_column(&table_ref, &column_id), column);
    }

    #[cfg(feature = "arrow")]
    #[test]
    fn we_can_get_data_accessor_from_record_batch_with_multiple_columns() {
        use arrow::array::{Int32Array, Int64Array};
        let rb = RecordBatch::try_from_iter([
            ("INTS", Arc::new(Int32Array::from(vec![1i32, 2, 3])) as ArrayRef),
            ("BIGS", Arc::new(Int64Array::from(vec![10i64, 20, 30])) as ArrayRef),
            ("FLAGS", Arc::new(BooleanArray::from(vec![true, false, true])) as ArrayRef),
        ])
        .unwrap();
        let alloc = Bump::new();
        let table_ref = TableRef::from_str("db.multi").unwrap();
        let table_data = TableDataAccessor::<TestScalar>::try_from_record_batch(&rb, 3, &alloc).unwrap();
        let data_accessor = DataAccessorImpl::new([(table_ref.clone(), table_data)].into_iter().collect());
        assert_eq!(data_accessor.get_length(&table_ref), 3);
        assert_eq!(data_accessor.get_offset(&table_ref), 3);
        assert_eq!(
            data_accessor.get_column(&table_ref, &Ident::new("INTS")),
            Column::Int(&[1i32, 2, 3])
        );
        assert_eq!(
            data_accessor.get_column(&table_ref, &Ident::new("BIGS")),
            Column::BigInt(&[10i64, 20, 30])
        );
        assert_eq!(
            data_accessor.get_column(&table_ref, &Ident::new("FLAGS")),
            Column::Boolean(&[true, false, true])
        );
    }

    #[cfg(feature = "arrow")]
    #[test]
    fn we_can_get_data_accessor_from_empty_record_batch() {
        use arrow::{datatypes::Schema, record_batch::RecordBatchOptions};
        let schema = alloc::sync::Arc::new(Schema::empty());
        let rb = RecordBatch::try_new_with_options(
            schema,
            vec![],
            &RecordBatchOptions::new().with_row_count(Some(0)),
        )
        .unwrap();
        let alloc = Bump::new();
        let table_ref = TableRef::from_str("test.empty").unwrap();
        let table_data = TableDataAccessor::<TestScalar>::try_from_record_batch(&rb, 0, &alloc).unwrap();
        let data_accessor = DataAccessorImpl::new([(table_ref.clone(), table_data)].into_iter().collect());
        assert_eq!(data_accessor.get_length(&table_ref), 0);
        assert_eq!(data_accessor.get_offset(&table_ref), 0);
    }

}