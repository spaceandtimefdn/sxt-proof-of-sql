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
    use alloc::string::ToString;

    #[test]
    fn table_operation_errors_display_expected_messages() {
        assert_eq!(
            TableOperationError::UnionNotEnoughTables.to_string(),
            "Cannot union fewer than 2 tables"
        );
        assert_eq!(
            TableOperationError::JoinWithDifferentNumberOfColumns {
                left_num_columns: 2,
                right_num_columns: 3,
            }
            .to_string(),
            "Cannot join tables with different numbers of columns: 2 and 3"
        );
        assert_eq!(
            TableOperationError::JoinIncompatibleTypes {
                left_type: ColumnType::Boolean,
                right_type: ColumnType::BigInt,
            }
            .to_string(),
            "Cannot join tables on columns with incompatible types: Boolean and BigInt"
        );
        assert_eq!(
            TableOperationError::ColumnDoesNotExist {
                column_ident: "missing_column".into(),
            }
            .to_string(),
            r#"Column Ident { value: "missing_column", quote_style: None } does not exist in table"#
        );
        assert_eq!(
            TableOperationError::DuplicateColumn.to_string(),
            "Some column is duplicated in table"
        );
        assert_eq!(
            TableOperationError::ColumnIndexOutOfBounds { column_index: 9 }.to_string(),
            "Column index out of bounds: 9"
        );
    }

    #[test]
    fn table_operation_errors_transparently_wrap_column_operation_errors() {
        assert_eq!(
            TableOperationError::ColumnOperationError {
                source: ColumnOperationError::DivisionByZero,
            }
            .to_string(),
            "Division by zero"
        );
    }
}
