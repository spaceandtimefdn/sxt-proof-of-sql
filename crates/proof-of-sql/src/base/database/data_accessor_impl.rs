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
            .first()
            .map_or(0, |(_, col)| col.len())
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
    use crate::base::database::{
        Column, DataAccessor, DataAccessorImpl, MetadataAccessor, TableDataAccessor, TableRef,
    };
    use crate::base::map::IndexSet;
        
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
        let column = Column::<TestScalar>::BigInt(&[3i64, -2]);
        let table_data_accessor =
            TableDataAccessor::new(2, [(column_id.clone(), column)].into_iter().collect());
        let table_ref = TableRef::from_names(Some("test"), "table");
        let data_accessor = DataAccessorImpl::new(
            [(table_ref.clone(), table_data_accessor)]
                .into_iter()
                .collect(),
        );
        assert_eq!(data_accessor.get_length(&table_ref), 2);
        assert_eq!(data_accessor.get_offset(&table_ref), 2);
        assert_eq!(data_accessor.get_column(&table_ref, &column_id), column);
    }

    #[test]
    fn we_can_get_length_for_empty_tables() {
        let table_data_accessor = TableDataAccessor::<TestScalar>::new(2, [].into_iter().collect());
        let table_ref = TableRef::from_names(Some("test"), "table");
        let data_accessor = DataAccessorImpl::new(
            [(table_ref.clone(), table_data_accessor)]
                .into_iter()
                .collect(),
        );
        assert_eq!(data_accessor.get_length(&table_ref), 0);
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

    #[test]
    fn we_can_get_table_from_data_accessor() {
        let bump = Bump::new();
        let col_id1 = Ident::from("col1");
        let col_id2 = Ident::from("col2");
        let column1 = Column::<TestScalar>::Int(&[1, 2]);
        let column2 = Column::<TestScalar>::Int(&[3, 4]);
        let table_data_accessor =
            TableDataAccessor::new(0, [("col1".into(), column1), ("col2".into(), column2)].into_iter().collect());
        let table_ref = TableRef::from_names(Some("test"), "table");
        let data_accessor = DataAccessorImpl::new(
            [(table_ref.clone(), table_data_accessor)]
                .into_iter()
                .collect(),
        );

        // 1) Test with empty column ids
        let table_empty = data_accessor.get_table(&table_ref, &IndexSet::default());
        assert_eq!(table_empty.num_rows(), 2);
        assert_eq!(table_empty.num_columns(), 0);

        // 2) Test with specific column ids
        let mut column_ids = IndexSet::default();
        column_ids.insert(col_id1.clone());
        column_ids.insert(col_id2.clone());
        let table_full = data_accessor.get_table(&table_ref, &column_ids);
        assert_eq!(table_full.num_rows(), 2);
        assert_eq!(table_full.num_columns(), 2);
        assert_eq!(table_full.inner_table().get(&col_id1).unwrap(), &Column::Int(&[1, 2]));
        assert_eq!(table_full.inner_table().get(&col_id2).unwrap(), &Column::Int(&[3, 4]));
    }
        );

        assert_eq!(data_accessor_impl.get_length(&table_ref), 2);
        assert_eq!(data_accessor_impl.get_offset(&table_ref), 1);
        assert_eq!(
            data_accessor_impl.get_column(&table_ref, &Ident::new("BOOLS")),
            Column::Boolean(&[true, false])
        );
    }
}
