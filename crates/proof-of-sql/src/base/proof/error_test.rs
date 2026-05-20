use super::{PlaceholderError, ProofError, ProofSizeMismatch};
use crate::base::database::ColumnType;

#[test]
fn proof_error_display_messages_are_stable() {
    assert_eq!(
        ProofError::VerificationError {
            error: "bad challenge"
        }
        .to_string(),
        "Verification error: bad challenge"
    );
    assert_eq!(
        ProofError::UnsupportedQueryPlan {
            error: "window functions"
        }
        .to_string(),
        "Unsupported query plan: window functions"
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
fn proof_size_mismatch_variants_display_expected_text() {
    assert_eq!(
        ProofSizeMismatch::SumcheckProofTooSmall.to_string(),
        "Sumcheck proof is too small"
    );
    assert_eq!(
        ProofSizeMismatch::TooFewMLEEvaluations.to_string(),
        "Proof has too few MLE evaluations"
    );
    assert_eq!(
        ProofSizeMismatch::PostResultCountMismatch.to_string(),
        "Post result challenge count mismatch"
    );
    assert_eq!(
        ProofSizeMismatch::ConstraintCountMismatch.to_string(),
        "Constraint count mismatch"
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
fn placeholder_error_display_messages_are_stable() {
    assert_eq!(
        PlaceholderError::InvalidPlaceholderIndex {
            index: 9,
            num_params: 2
        }
        .to_string(),
        "Invalid placeholder index: 9, number of params: 2"
    );
    assert_eq!(
        PlaceholderError::InvalidPlaceholderType {
            index: 1,
            expected: ColumnType::BigInt,
            actual: ColumnType::VarChar,
        }
        .to_string(),
        "Invalid placeholder type: 1, expected: BIGINT, actual: VARCHAR"
    );
    assert_eq!(
        PlaceholderError::ZeroPlaceholderId.to_string(),
        "Placeholder id must be greater than 0"
    );
}

#[test]
fn transparent_proof_error_variants_preserve_source_message() {
    let size_error = ProofError::ProofSizeMismatch {
        source: ProofSizeMismatch::TooFewRhoLengths,
    };
    assert_eq!(size_error.to_string(), "Proof has too few rho lengths");

    let placeholder_error = ProofError::PlaceholderError {
        source: PlaceholderError::ZeroPlaceholderId,
    };
    assert_eq!(
        placeholder_error.to_string(),
        "Placeholder id must be greater than 0"
    );
}
