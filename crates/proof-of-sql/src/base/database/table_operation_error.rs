use super::{ColumnField, ColumnOperationError, ColumnType};
use alloc::vec::Vec;
use core::result::Result;
use snafu::Snafu;
use sqlparser::ast::Ident;

/// Errors from operations on tables.
#[derive(Snafu, Debug, PartialEq, Eq)]
pub enum TableOperationError {
    /// Errors related to unioning tables with incompatible schemas.
    #[snafu(display(
        "Cannot union tables with incompatible schemas: {correct_schema:?} and {actual_schema:?}"
    ))]
    UnionIncompatibleSchemas {
        /// The correct data type
        correct_schema: Vec<ColumnField>,
        /// The schema of the table that caused the error
        actual_schema: Vec<ColumnField>,
    },
    /// Errors related to unioning fewer than 2 tables.
    #[snafu(display("Cannot union fewer than 2 tables"))]
    UnionNotEnoughTables,
    /// Errors related to joining tables with different numbers of columns.
    #[snafu(display(
        "Cannot join tables with different numbers of columns: {left_num_columns} and {right_num_columns}"
    ))]
    JoinWithDifferentNumberOfColumns {
        /// The number of columns in the left-hand table
        left_num_columns: usize,
        /// The number of columns in the right-hand table
        right_num_columns: usize,
    },
    /// Errors related to joining tables on columns with incompatible types.
    #[snafu(display(
        "Cannot join tables on columns with incompatible types: {left_type:?} and {right_type:?}"
    ))]
    JoinIncompatibleTypes {
        /// The left-hand side data type
        left_type: ColumnType,
        /// The right-hand side data type
        right_type: ColumnType,
    },
    /// Errors related to a column that does not exist in a table.
    #[snafu(display("Column {column_ident:?} does not exist in table"))]
    ColumnDoesNotExist {
        /// The nonexistent column identifier
        column_ident: Ident,
    },
    /// Errors related to duplicate columns in a table.
    #[snafu(display("Some column is duplicated in table"))]
    DuplicateColumn,
    /// Errors due to bad column operations.
    #[snafu(transparent)]
    ColumnOperationError {
        /// The underlying `ColumnOperationError`
        source: ColumnOperationError,
    },
    /// Errors related to column index out of bounds.
    #[snafu(display("Column index out of bounds: {column_index}"))]
    ColumnIndexOutOfBounds {
        /// The column index that is out of bounds
        column_index: usize,
    },
}

/// Result type for table operations
pub type TableOperationResult<T> = Result<T, TableOperationError>;

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{string::ToString, vec};

    #[test]
    fn we_can_display_table_operation_errors() {
        let id = Ident::new("id");
        let left_schema = vec![ColumnField::new(id.clone(), ColumnType::Int)];
        let right_schema = vec![ColumnField::new(id.clone(), ColumnType::BigInt)];

        let cases = [
            (
                TableOperationError::UnionIncompatibleSchemas {
                    correct_schema: left_schema.clone(),
                    actual_schema: right_schema,
                },
                "Cannot union tables with incompatible schemas: [ColumnField { name: Ident { value: \"id\", quote_style: None }, data_type: Int }] and [ColumnField { name: Ident { value: \"id\", quote_style: None }, data_type: BigInt }]".to_string(),
            ),
            (
                TableOperationError::UnionNotEnoughTables,
                "Cannot union fewer than 2 tables".to_string(),
            ),
            (
                TableOperationError::JoinWithDifferentNumberOfColumns {
                    left_num_columns: 2,
                    right_num_columns: 3,
                },
                "Cannot join tables with different numbers of columns: 2 and 3".to_string(),
            ),
            (
                TableOperationError::JoinIncompatibleTypes {
                    left_type: ColumnType::VarChar,
                    right_type: ColumnType::VarBinary,
                },
                "Cannot join tables on columns with incompatible types: VarChar and VarBinary"
                    .to_string(),
            ),
            (
                TableOperationError::ColumnDoesNotExist { column_ident: id },
                "Column Ident { value: \"id\", quote_style: None } does not exist in table"
                    .to_string(),
            ),
            (
                TableOperationError::DuplicateColumn,
                "Some column is duplicated in table".to_string(),
            ),
            (
                TableOperationError::ColumnIndexOutOfBounds { column_index: 5 },
                "Column index out of bounds: 5".to_string(),
            ),
        ];

        for (error, expected) in cases {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn we_can_display_transparent_column_operation_error() {
        let error = TableOperationError::ColumnOperationError {
            source: ColumnOperationError::DifferentColumnLength { len_a: 1, len_b: 2 },
        };

        assert_eq!(error.to_string(), "Columns have different lengths: 1 != 2");
    }

    #[test]
    fn we_can_use_table_operation_result_alias() {
        let result: TableOperationResult<()> = Err(TableOperationError::DuplicateColumn);

        assert!(matches!(result, Err(TableOperationError::DuplicateColumn)));
    }
}
