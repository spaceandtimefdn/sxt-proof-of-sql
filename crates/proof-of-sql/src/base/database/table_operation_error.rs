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
    use alloc::{string::ToString, vec::Vec};

    #[test]
    fn we_can_display_table_operation_errors() {
        let cases = [
            (
                TableOperationError::UnionIncompatibleSchemas {
                    correct_schema: Vec::new(),
                    actual_schema: Vec::new(),
                },
                "Cannot union tables with incompatible schemas: [] and []",
            ),
            (
                TableOperationError::UnionNotEnoughTables,
                "Cannot union fewer than 2 tables",
            ),
            (
                TableOperationError::JoinWithDifferentNumberOfColumns {
                    left_num_columns: 1,
                    right_num_columns: 2,
                },
                "Cannot join tables with different numbers of columns: 1 and 2",
            ),
            (
                TableOperationError::JoinIncompatibleTypes {
                    left_type: ColumnType::BigInt,
                    right_type: ColumnType::VarChar,
                },
                "Cannot join tables on columns with incompatible types: BigInt and VarChar",
            ),
            (
                TableOperationError::DuplicateColumn,
                "Some column is duplicated in table",
            ),
            (
                TableOperationError::ColumnOperationError {
                    source: ColumnOperationError::DivisionByZero,
                },
                "Division by zero",
            ),
            (
                TableOperationError::ColumnIndexOutOfBounds { column_index: 5 },
                "Column index out of bounds: 5",
            ),
        ];

        for (error, expected) in cases {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn we_can_display_missing_column_errors() {
        let error = TableOperationError::ColumnDoesNotExist {
            column_ident: Ident::new("missing_column"),
        };
        let message = error.to_string();

        assert!(message.starts_with("Column Ident { value: \"missing_column\""));
        assert!(message.ends_with(" does not exist in table"));
    }
}
