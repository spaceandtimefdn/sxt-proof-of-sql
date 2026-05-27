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
    use alloc::string::ToString;

    #[test]
    fn proof_error_display_includes_static_context() {
        let verification_error = ProofError::VerificationError {
            error: "sumcheck failed",
        };
        let unsupported_query_plan = ProofError::UnsupportedQueryPlan {
            error: "window function",
        };

        assert_eq!(
            verification_error.to_string(),
            "Verification error: sumcheck failed"
        );
        assert_eq!(
            unsupported_query_plan.to_string(),
            "Unsupported query plan: window function"
        );
    }

    #[test]
    fn proof_error_transparent_variants_delegate_display_to_the_source() {
        let proof_size_error = ProofError::ProofSizeMismatch {
            source: ProofSizeMismatch::TooFewRhoLengths,
        };
        let placeholder_error = ProofError::PlaceholderError {
            source: PlaceholderError::ZeroPlaceholderId,
        };

        assert_eq!(
            proof_size_error.to_string(),
            "Proof has too few rho lengths"
        );
        assert_eq!(
            placeholder_error.to_string(),
            "Placeholder id must be greater than 0"
        );
    }

    #[test]
    fn proof_size_mismatch_display_messages_are_stable() {
        let cases = [
            (
                ProofSizeMismatch::SumcheckProofTooSmall,
                "Sumcheck proof is too small",
            ),
            (
                ProofSizeMismatch::TooFewMLEEvaluations,
                "Proof has too few MLE evaluations",
            ),
            (
                ProofSizeMismatch::PostResultCountMismatch,
                "Post result challenge count mismatch",
            ),
            (
                ProofSizeMismatch::ConstraintCountMismatch,
                "Constraint count mismatch",
            ),
            (
                ProofSizeMismatch::TooFewBitDistributions,
                "Proof has too few bit distributions",
            ),
            (
                ProofSizeMismatch::TooFewChiLengths,
                "Proof has too few one lengths",
            ),
            (
                ProofSizeMismatch::TooFewRhoLengths,
                "Proof has too few rho lengths",
            ),
            (
                ProofSizeMismatch::TooFewSumcheckVariables,
                "Proof has too few sumcheck variables",
            ),
            (
                ProofSizeMismatch::ChiLengthNotFound,
                "Proof doesn't have requested one length",
            ),
            (
                ProofSizeMismatch::RhoLengthNotFound,
                "Proof doesn't have requested rho length",
            ),
        ];

        for (error, expected) in cases {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn placeholder_error_display_and_equality_track_the_rejected_placeholder() {
        let invalid_index = PlaceholderError::InvalidPlaceholderIndex {
            index: 4,
            num_params: 3,
        };
        let invalid_type = PlaceholderError::InvalidPlaceholderType {
            index: 2,
            expected: ColumnType::Int,
            actual: ColumnType::VarChar,
        };

        assert_eq!(
            invalid_index.to_string(),
            "Invalid placeholder index: 4, number of params: 3"
        );
        assert_eq!(
            invalid_type.to_string(),
            "Invalid placeholder type: 2, expected: INT, actual: VARCHAR"
        );
        assert_eq!(
            PlaceholderError::ZeroPlaceholderId.to_string(),
            "Placeholder id must be greater than 0"
        );
        assert_eq!(
            invalid_index,
            PlaceholderError::InvalidPlaceholderIndex {
                index: 4,
                num_params: 3
            }
        );
        assert_ne!(
            invalid_index,
            PlaceholderError::InvalidPlaceholderIndex {
                index: 3,
                num_params: 3
            }
        );
    }
}
