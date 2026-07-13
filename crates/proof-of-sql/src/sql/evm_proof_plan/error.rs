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
    use crate::{base::database::ColumnType, sql::AnalyzeError};

    #[test]
    fn we_display_basic_evm_proof_plan_errors() {
        assert_eq!(
            EVMProofPlanError::NotSupported.to_string(),
            "plan not yet supported"
        );
        assert_eq!(
            EVMProofPlanError::ColumnNotFound.to_string(),
            "column not found"
        );
        assert_eq!(
            EVMProofPlanError::TableNotFound.to_string(),
            "table not found"
        );
        assert_eq!(
            EVMProofPlanError::InvalidTableName.to_string(),
            "table name can not be parsed into TableRef"
        );
        assert_eq!(
            EVMProofPlanError::InvalidOutputColumnName.to_string(),
            "invalid or missing output column name"
        );
        assert_eq!(
            EVMProofPlanError::InconsistentGroupByColumnCounts.to_string(),
            "column counts in group by plans are inconsistent"
        );
        assert_eq!(
            EVMProofPlanError::IncorrectScalingFactor.to_string(),
            "incorrect scaling factor"
        );
    }

    #[test]
    fn we_transparently_display_analyze_errors() {
        let error = EVMProofPlanError::AnalyzeError {
            source: AnalyzeError::InvalidDataType {
                expr_type: ColumnType::VarBinary,
            },
        };

        assert_eq!(
            error.to_string(),
            "Expression has datatype BINARY, which was not valid"
        );
    }
}
