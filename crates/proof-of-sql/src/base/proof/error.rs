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
    use alloc::format;

    // ProofError Display tests
    #[test]
    fn proof_error_verification_displays_correctly() {
        let err = ProofError::VerificationError {
            error: "checksum mismatch",
        };
        assert_eq!(format!("{err}"), "Verification error: checksum mismatch");
    }

    #[test]
    fn proof_error_unsupported_plan_displays_correctly() {
        let err = ProofError::UnsupportedQueryPlan {
            error: "nested subquery",
        };
        assert_eq!(
            format!("{err}"),
            "Unsupported query plan: nested subquery"
        );
    }

    #[test]
    fn proof_error_invalid_type_coercion_displays_correctly() {
        let err = ProofError::InvalidTypeCoercion;
        assert_eq!(
            format!("{err}"),
            "Result does not match query: type mismatch"
        );
    }

    #[test]
    fn proof_error_field_names_mismatch_displays_correctly() {
        let err = ProofError::FieldNamesMismatch;
        assert_eq!(
            format!("{err}"),
            "Result does not match query: field names mismatch"
        );
    }

    #[test]
    fn proof_error_field_count_mismatch_displays_correctly() {
        let err = ProofError::FieldCountMismatch;
        assert_eq!(
            format!("{err}"),
            "Result does not match query: field count mismatch"
        );
    }

    // ProofSizeMismatch Display tests
    #[test]
    fn proof_size_mismatch_sumcheck_too_small_displays_correctly() {
        let err = ProofSizeMismatch::SumcheckProofTooSmall;
        assert_eq!(format!("{err}"), "Sumcheck proof is too small");
    }

    #[test]
    fn proof_size_mismatch_too_few_mle_displays_correctly() {
        let err = ProofSizeMismatch::TooFewMLEEvaluations;
        assert_eq!(format!("{err}"), "Proof has too few MLE evaluations");
    }

    #[test]
    fn proof_size_mismatch_post_result_count_displays_correctly() {
        let err = ProofSizeMismatch::PostResultCountMismatch;
        assert_eq!(format!("{err}"), "Post result challenge count mismatch");
    }

    #[test]
    fn proof_size_mismatch_constraint_count_displays_correctly() {
        let err = ProofSizeMismatch::ConstraintCountMismatch;
        assert_eq!(format!("{err}"), "Constraint count mismatch");
    }

    #[test]
    fn proof_size_mismatch_too_few_bit_distributions_displays_correctly() {
        let err = ProofSizeMismatch::TooFewBitDistributions;
        assert_eq!(format!("{err}"), "Proof has too few bit distributions");
    }

    #[test]
    fn proof_size_mismatch_too_few_chi_lengths_displays_correctly() {
        let err = ProofSizeMismatch::TooFewChiLengths;
        assert_eq!(format!("{err}"), "Proof has too few one lengths");
    }

    #[test]
    fn proof_size_mismatch_too_few_rho_lengths_displays_correctly() {
        let err = ProofSizeMismatch::TooFewRhoLengths;
        assert_eq!(format!("{err}"), "Proof has too few rho lengths");
    }

    #[test]
    fn proof_size_mismatch_too_few_sumcheck_variables_displays_correctly() {
        let err = ProofSizeMismatch::TooFewSumcheckVariables;
        assert_eq!(format!("{err}"), "Proof has too few sumcheck variables");
    }

    #[test]
    fn proof_size_mismatch_chi_length_not_found_displays_correctly() {
        let err = ProofSizeMismatch::ChiLengthNotFound;
        assert_eq!(format!("{err}"), "Proof doesn't have requested one length");
    }

    #[test]
    fn proof_size_mismatch_rho_length_not_found_displays_correctly() {
        let err = ProofSizeMismatch::RhoLengthNotFound;
        assert_eq!(
            format!("{err}"),
            "Proof doesn't have requested rho length"
        );
    }

    #[test]
    fn proof_size_mismatch_converts_to_proof_error() {
        let size_err = ProofSizeMismatch::SumcheckProofTooSmall;
        let proof_err: ProofError = size_err.into();
        assert!(matches!(proof_err, ProofError::ProofSizeMismatch { .. }));
    }

    // PlaceholderError tests
    #[test]
    fn placeholder_error_invalid_index_displays_correctly() {
        let err = PlaceholderError::InvalidPlaceholderIndex {
            index: 5,
            num_params: 3,
        };
        let msg = format!("{err}");
        assert!(msg.contains("5"));
        assert!(msg.contains("3"));
    }

    #[test]
    fn placeholder_error_invalid_type_displays_correctly() {
        let err = PlaceholderError::InvalidPlaceholderType {
            index: 1,
            expected: ColumnType::Int,
            actual: ColumnType::VarChar,
        };
        let msg = format!("{err}");
        assert!(msg.contains("1"));
        assert!(msg.contains("INT"));
        assert!(msg.contains("VARCHAR"));
    }

    #[test]
    fn placeholder_error_zero_id_displays_correctly() {
        let err = PlaceholderError::ZeroPlaceholderId;
        assert_eq!(
            format!("{err}"),
            "Placeholder id must be greater than 0"
        );
    }

    #[test]
    fn placeholder_errors_with_same_data_are_equal() {
        let a = PlaceholderError::InvalidPlaceholderIndex {
            index: 1,
            num_params: 2,
        };
        let b = PlaceholderError::InvalidPlaceholderIndex {
            index: 1,
            num_params: 2,
        };
        assert_eq!(a, b);
    }

    #[test]
    fn placeholder_error_converts_to_proof_error() {
        let placeholder_err = PlaceholderError::ZeroPlaceholderId;
        let proof_err: ProofError = placeholder_err.into();
        assert!(matches!(proof_err, ProofError::PlaceholderError { .. }));
    }
}
