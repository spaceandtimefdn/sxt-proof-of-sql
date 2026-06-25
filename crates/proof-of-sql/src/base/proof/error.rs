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
    fn verification_error_display_contains_message() {
        let e = ProofError::VerificationError { error: "bad proof" };
        assert!(alloc::format!("{e}").contains("bad proof"));
    }

    #[test]
    fn unsupported_query_plan_display_contains_message() {
        let e = ProofError::UnsupportedQueryPlan { error: "no plan" };
        assert!(alloc::format!("{e}").contains("no plan"));
    }

    #[test]
    fn invalid_type_coercion_display() {
        let e = ProofError::InvalidTypeCoercion;
        assert!(alloc::format!("{e}").contains("type mismatch"));
    }

    #[test]
    fn field_names_mismatch_display() {
        let e = ProofError::FieldNamesMismatch;
        assert!(alloc::format!("{e}").contains("field names"));
    }

    #[test]
    fn field_count_mismatch_display() {
        let e = ProofError::FieldCountMismatch;
        assert!(alloc::format!("{e}").contains("field count"));
    }

    #[test]
    fn proof_error_verification_debug() {
        let e = ProofError::VerificationError { error: "err" };
        assert!(alloc::format!("{e:?}").contains("VerificationError"));
    }

    #[test]
    fn proof_size_mismatch_sumcheck_too_small_display() {
        let e = ProofSizeMismatch::SumcheckProofTooSmall;
        assert!(alloc::format!("{e}").contains("small") || alloc::format!("{e}").contains("Sumcheck"));
    }

    #[test]
    fn proof_size_mismatch_too_few_mle_display() {
        let e = ProofSizeMismatch::TooFewMLEEvaluations;
        assert!(alloc::format!("{e}").contains("MLE") || alloc::format!("{e}").contains("few"));
    }

    #[test]
    fn proof_size_mismatch_chi_length_not_found_display() {
        let e = ProofSizeMismatch::ChiLengthNotFound;
        assert!(alloc::format!("{e}").contains("one length") || alloc::format!("{e}").contains("not found"));
    }

    #[test]
    fn placeholder_error_invalid_index_display() {
        let e = PlaceholderError::InvalidPlaceholderIndex { index: 5, num_params: 3 };
        let s = alloc::format!("{e}");
        assert!(s.contains("5") && s.contains("3"));
    }

    #[test]
    fn placeholder_error_invalid_type_display() {
        let e = PlaceholderError::InvalidPlaceholderType {
            index: 0,
            expected: ColumnType::BigInt,
            actual: ColumnType::Boolean,
        };
        let s = alloc::format!("{e}");
        assert!(s.contains("0"));
    }

    #[test]
    fn placeholder_error_zero_placeholder_id_display() {
        let e = PlaceholderError::ZeroPlaceholderId;
        assert!(alloc::format!("{e}").contains("0") || alloc::format!("{e}").contains("zero"));
    }

    #[test]
    fn placeholder_error_invalid_index_equality() {
        let e1 = PlaceholderError::InvalidPlaceholderIndex { index: 1, num_params: 2 };
        let e2 = PlaceholderError::InvalidPlaceholderIndex { index: 1, num_params: 2 };
        assert_eq!(e1, e2);
    }

    #[test]
    fn placeholder_error_zero_debug() {
        let e = PlaceholderError::ZeroPlaceholderId;
        assert!(alloc::format!("{e:?}").contains("ZeroPlaceholderId"));
    }
}
