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
    fn proof_error_verification_display() {
        let e = ProofError::VerificationError { error: "bad proof" };
        let s = alloc::format!("{e}");
        assert!(s.contains("bad proof"));
    }

    #[test]
    fn proof_error_unsupported_query_plan_display() {
        let e = ProofError::UnsupportedQueryPlan { error: "no cross join" };
        let s = alloc::format!("{e}");
        assert!(s.contains("no cross join"));
    }

    #[test]
    fn proof_error_invalid_type_coercion_display() {
        let e = ProofError::InvalidTypeCoercion;
        let s = alloc::format!("{e}");
        assert!(s.contains("type") || s.contains("mismatch"));
    }

    #[test]
    fn proof_error_field_names_mismatch_display() {
        let e = ProofError::FieldNamesMismatch;
        let s = alloc::format!("{e}");
        assert!(s.contains("field") || s.contains("mismatch"));
    }

    #[test]
    fn proof_error_field_count_mismatch_display() {
        let e = ProofError::FieldCountMismatch;
        let s = alloc::format!("{e}");
        assert!(s.contains("field") || s.contains("count") || s.contains("mismatch"));
    }

    #[test]
    fn proof_error_is_debug_formattable() {
        let e = ProofError::FieldCountMismatch;
        let s = alloc::format!("{e:?}");
        assert!(s.contains("FieldCountMismatch"));
    }

    #[test]
    fn proof_size_mismatch_sumcheck_too_small_display() {
        let e = ProofSizeMismatch::SumcheckProofTooSmall;
        let s = alloc::format!("{e}");
        assert!(s.contains("Sumcheck") || s.contains("small"));
    }

    #[test]
    fn proof_size_mismatch_too_few_mle_evaluations_display() {
        let e = ProofSizeMismatch::TooFewMLEEvaluations;
        let s = alloc::format!("{e}");
        assert!(s.contains("MLE") || s.contains("few"));
    }

    #[test]
    fn proof_size_mismatch_constraint_count_display() {
        let e = ProofSizeMismatch::ConstraintCountMismatch;
        let s = alloc::format!("{e}");
        assert!(s.contains("Constraint") || s.contains("mismatch"));
    }

    #[test]
    fn proof_size_mismatch_is_debug_formattable() {
        let e = ProofSizeMismatch::TooFewBitDistributions;
        let s = alloc::format!("{e:?}");
        assert!(s.contains("TooFewBitDistributions"));
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
            index: 2,
            expected: ColumnType::BigInt,
            actual: ColumnType::Boolean,
        };
        let s = alloc::format!("{e}");
        assert!(s.contains("2"));
    }

    #[test]
    fn placeholder_error_zero_id_display() {
        let e = PlaceholderError::ZeroPlaceholderId;
        let s = alloc::format!("{e}");
        assert!(s.contains("0") || s.contains("greater") || s.contains("zero"));
    }

    #[test]
    fn placeholder_error_zero_id_equality() {
        assert_eq!(
            PlaceholderError::ZeroPlaceholderId,
            PlaceholderError::ZeroPlaceholderId
        );
    }

    #[test]
    fn placeholder_error_invalid_index_equality() {
        let a = PlaceholderError::InvalidPlaceholderIndex { index: 1, num_params: 1 };
        let b = PlaceholderError::InvalidPlaceholderIndex { index: 1, num_params: 1 };
        assert_eq!(a, b);
    }

    #[test]
    fn placeholder_error_is_debug_formattable() {
        let e = PlaceholderError::ZeroPlaceholderId;
        let s = alloc::format!("{e:?}");
        assert!(s.contains("ZeroPlaceholderId"));
    }
}
