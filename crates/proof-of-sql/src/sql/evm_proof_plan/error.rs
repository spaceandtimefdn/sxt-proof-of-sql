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
    use super::*;
    use alloc::string::{String, ToString};

    #[test]
    fn we_can_display_evm_proof_plan_errors() {
        let cases = [
            (EVMProofPlanError::NotSupported, "plan not yet supported"),
            (EVMProofPlanError::ColumnNotFound, "column not found"),
            (EVMProofPlanError::TableNotFound, "table not found"),
            (
                EVMProofPlanError::InvalidTableName,
                "table name can not be parsed into TableRef",
            ),
            (
                EVMProofPlanError::InvalidOutputColumnName,
                "invalid or missing output column name",
            ),
            (
                EVMProofPlanError::InconsistentGroupByColumnCounts,
                "column counts in group by plans are inconsistent",
            ),
            (
                EVMProofPlanError::IncorrectScalingFactor,
                "incorrect scaling factor",
            ),
        ];

        for (error, expected) in cases {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn analyze_errors_are_transparently_wrapped() {
        let error = EVMProofPlanError::AnalyzeError {
            source: AnalyzeError::NotEnoughInputPlans,
        };

        assert_eq!(error.to_string(), "Not enough input plans");
    }

    #[test]
    fn evm_proof_plan_errors_convert_into_string() {
        let error = EVMProofPlanError::ColumnNotFound;
        let message: String = error.to_string();

        assert_eq!(message, "column not found");
    }
}
