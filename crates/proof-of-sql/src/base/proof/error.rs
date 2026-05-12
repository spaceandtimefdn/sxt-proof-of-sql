use crate::base::database::ColumnType;
use snafu::Snafu;

#[derive(Snafu, Debug)]
/// These errors occur when a proof failed to verify.
pub enum ProofError {
    #[snafu(display("Verification error: {error}"))]
    /// This error occurs when a proof failed to verify.
    VerificationError { error: &'static str },
    /// This error occurs when a query plan is not supported.
    #[snafu(display("Unsupported query plan: {error}"))]
    UnsupportedQueryPlan { error: &'static str },
    /// This error occurs the type coercion of the result table failed.
    #[snafu(display("Result does not match query: type mismatch"))]
    InvalidTypeCoercion,
    /// This error occurs when the field names of the result table do not match the query.
    #[snafu(display("Result does not match query: field names mismatch"))]
    FieldNamesMismatch,
    /// This error occurs when the number of fields in the result table does not match the query.
    #[snafu(display("Result does not match query: field count mismatch"))]
    FieldCountMismatch,
    #[snafu(transparent)]
    ProofSizeMismatch { source: ProofSizeMismatch },
    #[snafu(transparent)]
    PlaceholderError { source: PlaceholderError },
}

#[derive(Snafu, Debug)]
/// These errors occur when the proof size does not match the expected size.
pub enum ProofSizeMismatch {
    /// This error occurs when the sumcheck proof doesn't have enough coefficients.
    #[snafu(display("Sumcheck proof is too small"))]
    SumcheckProofTooSmall,
    /// This error occurs when the proof has too few MLE evaluations.
    #[snafu(display("Proof has too few MLE evaluations"))]
    TooFewMLEEvaluations,
    /// This error occurs when the number of post result challenges in the proof plan doesn't match the number specified in the proof
    #[snafu(display("Post result challenge count mismatch"))]
    PostResultCountMismatch,
    /// This error occurs when the number of constraints in the proof plan doesn't match the number specified in the proof
    #[snafu(display("Constraint count mismatch"))]
    ConstraintCountMismatch,
    /// This error occurs when the proof has too few bit distributions.
    #[snafu(display("Proof has too few bit distributions"))]
    TooFewBitDistributions,
    /// This error occurs when the proof has too few one lengths.
    #[snafu(display("Proof has too few one lengths"))]
    TooFewChiLengths,
    /// This error occurs when the proof has too few rho lengths.
    #[snafu(display("Proof has too few rho lengths"))]
    TooFewRhoLengths,
    /// This error occurs when the proof has too few sumcheck variables.
    #[snafu(display("Proof has too few sumcheck variables"))]
    TooFewSumcheckVariables,
    /// This error occurs when a requested one length is not found.
    #[snafu(display("Proof doesn't have requested one length"))]
    ChiLengthNotFound,
    /// This error occurs when a requested rho length is not found.
    #[snafu(display("Proof doesn't have requested rho length"))]
    RhoLengthNotFound,
}

/// Errors related to placeholders
#[derive(Snafu, Debug, PartialEq, Eq)]
pub enum PlaceholderError {
    #[snafu(display("Invalid placeholder index: {index}, number of params: {num_params}"))]
    /// Placeholder id is invalid
    InvalidPlaceholderIndex {
        /// The invalid placeholder index
        index: usize,
        /// The number of parameters
        num_params: usize,
    },

    #[snafu(display("Invalid placeholder type: {index}, expected: {expected}, actual: {actual}"))]
    /// Placeholder type is invalid
    InvalidPlaceholderType {
        /// The invalid placeholder id
        index: usize,
        /// The expected type
        expected: ColumnType,
        /// The actual type
        actual: ColumnType,
    },

    #[snafu(display("Placeholder id must be greater than 0"))]
    /// Placeholder id is zero
    ZeroPlaceholderId,
}

/// Result type for placeholder errors
pub type PlaceholderResult<T> = Result<T, PlaceholderError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proof_errors_preserve_payloads() {
        let verification_error = ProofError::VerificationError {
            error: "bad opening",
        };
        assert!(matches!(
            verification_error,
            ProofError::VerificationError {
                error: "bad opening"
            }
        ));

        let unsupported_query_plan = ProofError::UnsupportedQueryPlan {
            error: "window function",
        };
        assert!(matches!(
            unsupported_query_plan,
            ProofError::UnsupportedQueryPlan {
                error: "window function"
            }
        ));
    }

    #[test]
    fn proof_errors_display_readable_messages() {
        assert_eq!(
            ProofError::VerificationError {
                error: "bad opening"
            }
            .to_string(),
            "Verification error: bad opening"
        );
        assert_eq!(
            ProofError::UnsupportedQueryPlan {
                error: "window function"
            }
            .to_string(),
            "Unsupported query plan: window function"
        );
        assert_eq!(
            ProofError::InvalidTypeCoercion.to_string(),
            "Result does not match query: type mismatch"
        );
        assert_eq!(
            ProofError::FieldNamesMismatch.to_string(),
            "Result does not match query: field names mismatch"
        );
        assert_eq!(
            ProofError::FieldCountMismatch.to_string(),
            "Result does not match query: field count mismatch"
        );
    }

    #[test]
    fn proof_size_errors_display_readable_messages() {
        assert_eq!(
            ProofSizeMismatch::SumcheckProofTooSmall.to_string(),
            "Sumcheck proof is too small"
        );
        assert_eq!(
            ProofSizeMismatch::TooFewMLEEvaluations.to_string(),
            "Proof has too few MLE evaluations"
        );
        assert_eq!(
            ProofSizeMismatch::TooFewBitDistributions.to_string(),
            "Proof has too few bit distributions"
        );
        assert_eq!(
            ProofSizeMismatch::TooFewChiLengths.to_string(),
            "Proof has too few one lengths"
        );
        assert_eq!(
            ProofSizeMismatch::TooFewRhoLengths.to_string(),
            "Proof has too few rho lengths"
        );
        assert_eq!(
            ProofSizeMismatch::TooFewSumcheckVariables.to_string(),
            "Proof has too few sumcheck variables"
        );
        assert_eq!(
            ProofSizeMismatch::ChiLengthNotFound.to_string(),
            "Proof doesn't have requested one length"
        );
        assert_eq!(
            ProofSizeMismatch::RhoLengthNotFound.to_string(),
            "Proof doesn't have requested rho length"
        );
    }

    #[test]
    fn placeholder_errors_display_readable_messages() {
        assert_eq!(
            PlaceholderError::InvalidPlaceholderIndex {
                index: 3,
                num_params: 2
            }
            .to_string(),
            "Invalid placeholder index: 3, number of params: 2"
        );
        assert_eq!(
            PlaceholderError::InvalidPlaceholderType {
                index: 1,
                expected: ColumnType::Int,
                actual: ColumnType::VarChar
            }
            .to_string(),
            "Invalid placeholder type: 1, expected: INT, actual: VARCHAR"
        );
        assert_eq!(
            PlaceholderError::ZeroPlaceholderId.to_string(),
            "Placeholder id must be greater than 0"
        );
    }
}
