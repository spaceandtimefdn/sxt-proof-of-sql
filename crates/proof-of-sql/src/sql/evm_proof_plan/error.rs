use crate::sql::AnalyzeError;
use snafu::Snafu;

/// Represents errors that can occur in the EVM proof plan module.
#[derive(Snafu, Debug, PartialEq)]
pub(crate) enum EVMProofPlanError {
    /// Error indicating that the plan is not supported.
    #[snafu(display("plan not yet supported"))]
    NotSupported,
    /// Error indicating that the column was not found.
    #[snafu(display("column not found"))]
    ColumnNotFound,
    /// Error indicating that the table was not found.
    #[snafu(display("table not found"))]
    TableNotFound,
    /// Error indicating that table name can not be parsed into `TableRef`.
    #[snafu(display("table name can not be parsed into TableRef"))]
    InvalidTableName,
    /// Error indicating that the output column name is invalid or missing.
    #[snafu(display("invalid or missing output column name"))]
    InvalidOutputColumnName,
    /// Error indicating that the column counts in group by plans are inconsistent.
    #[snafu(display("column counts in group by plans are inconsistent"))]
    InconsistentGroupByColumnCounts,
    /// Analyze error
    #[snafu(transparent)]
    AnalyzeError {
        /// The underlying source error
        source: AnalyzeError,
    },
    /// Incorrect scaling factor
    #[snafu(display("incorrect scaling factor"))]
    IncorrectScalingFactor,
}

/// Result type for EVM proof plan operations.
pub(crate) type EVMProofPlanResult<T> = core::result::Result<T, EVMProofPlanError>;

#[cfg(test)]
mod tests {
    use super::EVMProofPlanError;

    #[test]
    fn not_supported_display() {
        let e = EVMProofPlanError::NotSupported;
        assert_eq!(alloc::format!("{e}"), "plan not yet supported");
    }

    #[test]
    fn column_not_found_display() {
        let e = EVMProofPlanError::ColumnNotFound;
        assert_eq!(alloc::format!("{e}"), "column not found");
    }

    #[test]
    fn table_not_found_display() {
        let e = EVMProofPlanError::TableNotFound;
        assert_eq!(alloc::format!("{e}"), "table not found");
    }

    #[test]
    fn invalid_table_name_display() {
        let e = EVMProofPlanError::InvalidTableName;
        assert_eq!(alloc::format!("{e}"), "table name can not be parsed into TableRef");
    }

    #[test]
    fn invalid_output_column_name_display() {
        let e = EVMProofPlanError::InvalidOutputColumnName;
        assert_eq!(alloc::format!("{e}"), "invalid or missing output column name");
    }

    #[test]
    fn incorrect_scaling_factor_display() {
        let e = EVMProofPlanError::IncorrectScalingFactor;
        assert_eq!(alloc::format!("{e}"), "incorrect scaling factor");
    }

    #[test]
    fn not_supported_debug() {
        let e = EVMProofPlanError::NotSupported;
        assert!(alloc::format!("{e:?}").contains("NotSupported"));
    }

    #[test]
    fn not_supported_equality() {
        assert_eq!(EVMProofPlanError::NotSupported, EVMProofPlanError::NotSupported);
    }

    #[test]
    fn different_variants_not_equal() {
        assert_ne!(EVMProofPlanError::NotSupported, EVMProofPlanError::ColumnNotFound);
    }
}
