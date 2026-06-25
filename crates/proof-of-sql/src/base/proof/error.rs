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
    use super::{PlaceholderError, ProofError, ProofSizeMismatch};
    use crate::base::database::ColumnType;

    #[test]
    fn proof_error_verification_error_displays_message() {
        let err = ProofError::VerificationError { error: "bad proof" };
        assert_eq!(err.to_string(), "Verification error: bad proof");
    }

    #[test]
    fn proof_error_invalid_type_coercion_displays_correctly() {
        assert_eq!(
            ProofError::InvalidTypeCoercion.to_string(),
            "Result does not match query: type mismatch"
        );
    }

    #[test]
    fn proof_error_field_names_mismatch_displays_correctly() {
        assert_eq!(
            ProofError::FieldNamesMismatch.to_string(),
            "Result does not match query: field names mismatch"
        );
    }

    #[test]
    fn proof_error_field_count_mismatch_displays_correctly() {
        assert_eq!(
            ProofError::FieldCountMismatch.to_string(),
            "Result does not match query: field count mismatch"
        );
    }

    #[test]
    fn proof_size_mismatch_sumcheck_too_small_displays_correctly() {
        assert_eq!(
            ProofSizeMismatch::SumcheckProofTooSmall.to_string(),
            "Sumcheck proof is too small"
        );
    }

    #[test]
    fn proof_size_mismatch_too_few_mle_evaluations_displays_correctly() {
        assert_eq!(
            ProofSizeMismatch::TooFewMLEEvaluations.to_string(),
            "Proof has too few MLE evaluations"
        );
    }

    #[test]
    fn proof_size_mismatch_chi_length_not_found_displays_correctly() {
        assert_eq!(
            ProofSizeMismatch::ChiLengthNotFound.to_string(),
            "Proof doesn't have requested one length"
        );
    }

    #[test]
    fn placeholder_error_zero_id_displays_correctly() {
        assert_eq!(
            PlaceholderError::ZeroPlaceholderId.to_string(),
            "Placeholder id must be greater than 0"
        );
    }

    #[test]
    fn placeholder_error_invalid_index_displays_index_and_params() {
        let err = PlaceholderError::InvalidPlaceholderIndex { index: 5, num_params: 3 };
        let msg = err.to_string();
        assert!(msg.contains("5"));
        assert!(msg.contains("3"));
    }

    #[test]
    fn placeholder_error_invalid_type_displays_types() {
        let err = PlaceholderError::InvalidPlaceholderType {
            index: 1,
            expected: ColumnType::BigInt,
            actual: ColumnType::Boolean,
        };
        let msg = err.to_string();
        assert!(msg.contains("BigInt"));
        assert!(msg.contains("Boolean"));
    }

    #[test]
    fn placeholder_errors_implement_partial_eq() {
        assert_eq!(PlaceholderError::ZeroPlaceholderId, PlaceholderError::ZeroPlaceholderId);
        assert_ne!(PlaceholderError::ZeroPlaceholderId, PlaceholderError::InvalidPlaceholderIndex { index: 0, num_params: 0 });
    }
}

#[cfg(test)]
mod tests {
    use super::{PlaceholderError, ProofError};
    use crate::base::database::ColumnType;
    use alloc::string::ToString;

    #[test]
    fn verification_error_displays_message() {
        let err = ProofError::VerificationError { error: "bad hash" };
        assert_eq!(err.to_string(), "Verification error: bad hash");
    }

    #[test]
    fn unsupported_query_plan_displays_message() {
        let err = ProofError::UnsupportedQueryPlan { error: "join not supported" };
        assert_eq!(err.to_string(), "Unsupported query plan: join not supported");
    }

    #[test]
    fn invalid_type_coercion_displays_message() {
        let err = ProofError::InvalidTypeCoercion;
        assert_eq!(err.to_string(), "Result does not match query: type mismatch");
    }

    #[test]
    fn field_names_mismatch_displays_message() {
        let err = ProofError::FieldNamesMismatch;
        assert_eq!(err.to_string(), "Result does not match query: field names mismatch");
    }

    #[test]
    fn field_count_mismatch_displays_message() {
        let err = ProofError::FieldCountMismatch;
        assert_eq!(err.to_string(), "Result does not match query: field count mismatch");
    }

    #[test]
    fn proof_error_debug_contains_variant() {
        assert!(format!("{:?}", ProofError::FieldCountMismatch).contains("FieldCountMismatch"));
    }

    #[test]
    fn placeholder_invalid_index_displays_index_and_count() {
        let err = PlaceholderError::InvalidPlaceholderIndex { index: 3, num_params: 2 };
        let s = err.to_string();
        assert!(s.contains("3"));
        assert!(s.contains("2"));
    }

    #[test]
    fn placeholder_invalid_type_displays_types() {
        let err = PlaceholderError::InvalidPlaceholderType {
            index: 1,
            expected: ColumnType::BigInt,
            actual: ColumnType::Boolean,
        };
        let s = err.to_string();
        assert!(s.contains("BigInt"));
        assert!(s.contains("Boolean"));
    }

    #[test]
    fn placeholder_zero_id_displays_message() {
        let err = PlaceholderError::ZeroPlaceholderId;
        assert_eq!(err.to_string(), "Placeholder id must be greater than 0");
    }

    #[test]
    fn placeholder_errors_implement_partial_eq() {
        assert_eq!(PlaceholderError::ZeroPlaceholderId, PlaceholderError::ZeroPlaceholderId);
        assert_ne!(
            PlaceholderError::ZeroPlaceholderId,
            PlaceholderError::InvalidPlaceholderIndex { index: 0, num_params: 0 }
        );
    }
}
