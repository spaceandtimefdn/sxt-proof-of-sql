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
    use super::TableOperationError;
    use crate::base::database::ColumnType;
    use alloc::string::ToString;

    #[test]
    fn union_not_enough_tables_displays_correctly() {
        assert_eq!(
            TableOperationError::UnionNotEnoughTables.to_string(),
            "Cannot union fewer than 2 tables"
        );
    }

    #[test]
    fn join_with_different_number_of_columns_displays_counts() {
        let err = TableOperationError::JoinWithDifferentNumberOfColumns {
            left_num_columns: 3,
            right_num_columns: 5,
        };
        let msg = err.to_string();
        assert!(msg.contains("3"));
        assert!(msg.contains("5"));
        assert!(msg.contains("Cannot join tables with different numbers of columns"));
    }

    #[test]
    fn join_incompatible_types_displays_both_types() {
        let err = TableOperationError::JoinIncompatibleTypes {
            left_type: ColumnType::BigInt,
            right_type: ColumnType::Boolean,
        };
        let msg = err.to_string();
        assert!(msg.contains("BigInt"));
        assert!(msg.contains("Boolean"));
        assert!(msg.contains("Cannot join tables on columns with incompatible types"));
    }

    #[test]
    fn duplicate_column_displays_correctly() {
        assert_eq!(
            TableOperationError::DuplicateColumn.to_string(),
            "Some column is duplicated in table"
        );
    }

    #[test]
    fn column_index_out_of_bounds_displays_index() {
        let err = TableOperationError::ColumnIndexOutOfBounds { column_index: 7 };
        let msg = err.to_string();
        assert!(msg.contains("7"));
        assert!(msg.contains("Column index out of bounds"));
    }

    #[test]
    fn table_operation_errors_implement_partial_eq() {
        assert_eq!(TableOperationError::DuplicateColumn, TableOperationError::DuplicateColumn);
        assert_eq!(TableOperationError::UnionNotEnoughTables, TableOperationError::UnionNotEnoughTables);
    }

    #[test]
    fn table_operation_error_debug_contains_variant_name() {
        let debug = format!("{:?}", TableOperationError::DuplicateColumn);
        assert!(debug.contains("DuplicateColumn"));
    }
}
