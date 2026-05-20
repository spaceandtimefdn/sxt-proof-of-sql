use super::{EVMProofPlanError, EVMProofPlanResult};
use crate::sql::AnalyzeError;

#[test]
fn evm_proof_plan_errors_have_expected_display_messages() {
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
fn evm_proof_plan_error_preserves_transparent_analyze_error_display() {
    let error = EVMProofPlanError::AnalyzeError {
        source: AnalyzeError::NotEnoughInputPlans,
    };

    assert_eq!(error.to_string(), "Not enough input plans");
}

#[test]
fn evm_proof_plan_result_alias_uses_evm_error_type() {
    let result: EVMProofPlanResult<()> = Err(EVMProofPlanError::NotSupported);

    assert_eq!(result.unwrap_err(), EVMProofPlanError::NotSupported);
}
