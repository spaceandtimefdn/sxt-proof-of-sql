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
    fn we_can_format_union_schema_errors() {
        let error = TableOperationError::UnionIncompatibleSchemas {
            correct_schema: vec![ColumnField::new(Ident::new("id"), ColumnType::BigInt)],
            actual_schema: vec![ColumnField::new(Ident::new("id"), ColumnType::Int)],
        };
        let message = error.to_string();

        assert!(message.contains("Cannot union tables with incompatible schemas"));
        assert!(message.contains("BigInt"));
        assert!(message.contains("Int"));
    }

    #[test]
    fn we_can_format_join_errors() {
        let column_count_error = TableOperationError::JoinWithDifferentNumberOfColumns {
            left_num_columns: 2,
            right_num_columns: 3,
        };
        let type_error = TableOperationError::JoinIncompatibleTypes {
            left_type: ColumnType::BigInt,
            right_type: ColumnType::VarChar,
        };

        assert_eq!(
            column_count_error.to_string(),
            "Cannot join tables with different numbers of columns: 2 and 3"
        );
        assert_eq!(
            type_error.to_string(),
            "Cannot join tables on columns with incompatible types: BigInt and VarChar"
        );
    }

    #[test]
    fn we_can_format_column_lookup_errors() {
        let missing_column_error = TableOperationError::ColumnDoesNotExist {
            column_ident: Ident::new("missing_column"),
        };
        let index_error = TableOperationError::ColumnIndexOutOfBounds { column_index: 7 };

        let missing_column_message = missing_column_error.to_string();

        assert!(missing_column_message.starts_with("Column Ident"));
        assert!(missing_column_message.contains("missing_column"));
        assert!(missing_column_message.ends_with("does not exist in table"));
        assert_eq!(index_error.to_string(), "Column index out of bounds: 7");
    }

    #[test]
    fn we_can_format_simple_table_operation_errors() {
        assert_eq!(
            TableOperationError::UnionNotEnoughTables.to_string(),
            "Cannot union fewer than 2 tables"
        );
        assert_eq!(
            TableOperationError::DuplicateColumn.to_string(),
            "Some column is duplicated in table"
        );
    }
}
