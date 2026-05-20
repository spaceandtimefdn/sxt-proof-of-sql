use super::{PlaceholderError, PlaceholderResult, ProofError, ProofSizeMismatch};
use crate::base::database::ColumnType;

#[test]
fn proof_errors_have_expected_display_messages() {
    let cases = [
        (
            ProofError::VerificationError {
                error: "bad transcript",
            },
            "Verification error: bad transcript",
        ),
        (
            ProofError::UnsupportedQueryPlan { error: "aggregate" },
            "Unsupported query plan: aggregate",
        ),
        (
            ProofError::InvalidTypeCoercion,
            "Result does not match query: type mismatch",
        ),
        (
            ProofError::FieldNamesMismatch,
            "Result does not match query: field names mismatch",
        ),
        (
            ProofError::FieldCountMismatch,
            "Result does not match query: field count mismatch",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
    }
}

#[test]
fn proof_size_mismatch_errors_have_expected_display_messages() {
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
fn placeholder_errors_have_expected_display_messages() {
    let cases = [
        (
            PlaceholderError::InvalidPlaceholderIndex {
                index: 3,
                num_params: 2,
            },
            "Invalid placeholder index: 3, number of params: 2",
        ),
        (
            PlaceholderError::InvalidPlaceholderType {
                index: 1,
                expected: ColumnType::BigInt,
                actual: ColumnType::VarChar,
            },
            "Invalid placeholder type: 1, expected: BIGINT, actual: VARCHAR",
        ),
        (
            PlaceholderError::ZeroPlaceholderId,
            "Placeholder id must be greater than 0",
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
    }
}

#[test]
fn proof_error_transparently_displays_nested_errors() {
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
fn placeholder_result_alias_uses_placeholder_error_type() {
    let result: PlaceholderResult<()> = Err(PlaceholderError::ZeroPlaceholderId);

    assert_eq!(result.unwrap_err(), PlaceholderError::ZeroPlaceholderId);
}
