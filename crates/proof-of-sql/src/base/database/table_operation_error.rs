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

    #[test]
    fn union_not_enough_tables_display() {
        let e = TableOperationError::UnionNotEnoughTables;
        let s = alloc::format!("{e}");
        assert!(s.contains("2") || s.contains("fewer"));
    }

    #[test]
    fn join_with_different_number_of_columns_display() {
        let e = TableOperationError::JoinWithDifferentNumberOfColumns {
            left_num_columns: 3,
            right_num_columns: 5,
        };
        let s = alloc::format!("{e}");
        assert!(s.contains("3") && s.contains("5"));
    }

    #[test]
    fn join_incompatible_types_display() {
        let e = TableOperationError::JoinIncompatibleTypes {
            left_type: ColumnType::BigInt,
            right_type: ColumnType::Boolean,
        };
        let s = alloc::format!("{e}");
        assert!(s.contains("incompatible") || s.contains("Incompatible"));
    }

    #[test]
    fn duplicate_column_display() {
        let e = TableOperationError::DuplicateColumn;
        let s = alloc::format!("{e}");
        assert!(s.contains("duplicated") || s.contains("duplicate"));
    }

    #[test]
    fn column_index_out_of_bounds_display() {
        let e = TableOperationError::ColumnIndexOutOfBounds { column_index: 42 };
        let s = alloc::format!("{e}");
        assert!(s.contains("42"));
    }

    #[test]
    fn union_not_enough_tables_equality() {
        assert_eq!(
            TableOperationError::UnionNotEnoughTables,
            TableOperationError::UnionNotEnoughTables
        );
    }

    #[test]
    fn duplicate_column_equality() {
        assert_eq!(
            TableOperationError::DuplicateColumn,
            TableOperationError::DuplicateColumn
        );
    }

    #[test]
    fn table_operation_error_is_debug_formattable() {
        let e = TableOperationError::DuplicateColumn;
        let s = alloc::format!("{e:?}");
        assert!(s.contains("DuplicateColumn"));
    }

    #[test]
    fn column_does_not_exist_display() {
        use sqlparser::ast::Ident;
        let e = TableOperationError::ColumnDoesNotExist {
            column_ident: Ident::new("my_col"),
        };
        let s = alloc::format!("{e}");
        assert!(s.contains("my_col") || s.contains("exist"));
    }
}
