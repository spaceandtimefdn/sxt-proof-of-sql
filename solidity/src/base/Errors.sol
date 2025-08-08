// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

/// @dev Error code for when ECADD inputs are invalid.
uint32 constant ERR_INVALID_EC_ADD_INPUTS = 0x765bcba0;
/// @dev Error code for when ECMUL inputs are invalid.
uint32 constant ERR_INVALID_EC_MUL_INPUTS = 0xe32c7472;
/// @dev Error code for when ECPAIRING inputs are invalid.
uint32 constant ERR_INVALID_EC_PAIRING_INPUTS = 0x4385b511;
/// @dev Error code for when a boolean literal is invalid.
uint32 constant ERR_INVALID_BOOLEAN = 0xaf979eb5;
/// @dev Error code for commitment array having odd length which is impossible
/// since each commitment is 2 elements.
uint32 constant ERR_COMMITMENT_ARRAY_ODD_LENGTH = 0x88acadef;
/// @dev Error code for when the size of a sumcheck proof is incorrect.
uint32 constant ERR_INVALID_SUMCHECK_PROOF_SIZE = 0x3f889a17;
/// @dev Error code for when the evaluation of a round in a sumcheck proof does not match the expected value.
uint32 constant ERR_ROUND_EVALUATION_MISMATCH = 0x741f5c3f;
/// @dev Error code for when a dequeue attempt was made on an empty queue.
uint32 constant ERR_EMPTY_QUEUE = 0x31dcf2b5;
/// @dev Error code for when the HyperKZG proof has an inconsistent v.
uint32 constant ERR_HYPER_KZG_INCONSISTENT_V = 0x6a5ae827;
/// @dev Error code for when the HyperKZG proof has an empty x point.
uint32 constant ERR_HYPER_KZG_EMPTY_POINT = 0xf1c6069e;
/// @dev Error code for when the HyperKZG proof fails the pairing check.
uint32 constant ERR_HYPER_KZG_PAIRING_CHECK_FAILED = 0xa41148a3;
/// @dev Error code for when the produces constraint degree is higher than the provided proof.
uint32 constant ERR_CONSTRAINT_DEGREE_TOO_HIGH = 0x8568ae69;
/// @dev Error code for when the case literal in a switch statement is incorrect.
uint32 constant ERR_INCORRECT_CASE_CONST = 0x9324fb03;
/// @dev Error code for when an index is invalid.
uint32 constant ERR_INVALID_INDEX = 0x63df8171;
/// @dev Error code for when a proof expression variant is unsupported.
uint32 constant ERR_UNSUPPORTED_PROOF_EXPR_VARIANT = 0xb8a26620;
/// @dev Error code for when PCS batch lengths don't match.
uint32 constant ERR_PCS_BATCH_LENGTH_MISMATCH = 0x5a64ac85;
/// @dev Error code for when result column counts don't match.
uint32 constant ERR_RESULT_COLUMN_COUNT_MISMATCH = 0x4b08a100;
/// @dev Error code for when a result column name is invalid.
uint32 constant ERR_INVALID_RESULT_COLUMN_NAME = 0xc5a456b6;
/// @dev Error code for when result column lengths are inconsistent.
uint32 constant ERR_INCONSISTENT_RESULT_COLUMN_LENGTHS = 0x68c99843;
/// @dev Error code for when the result is incorrect.
uint32 constant ERR_INCORRECT_RESULT = 0x3ad072a3;
/// @dev Error code for when HyperKZG proof size doesn't match.
uint32 constant ERR_HYPER_KZG_PROOF_SIZE_MISMATCH = 0xbe285ccd;
/// @dev Error code for when aggregate evaluation doesn't match.
uint32 constant ERR_AGGREGATE_EVALUATION_MISMATCH = 0xf5c6cb38;
/// @dev Error code for when proof type is unsupported.
uint32 constant ERR_UNSUPPORTED_PROOF = 0x6f1c50d9;
/// @dev Error code for when a proof plan variant is unsupported.
uint32 constant ERR_UNSUPPORTED_PROOF_PLAN_VARIANT = 0xe5503cfa;
/// @dev Error code for when a data type variant is unsupported.
uint32 constant ERR_UNSUPPORTED_DATA_TYPE_VARIANT = 0xbd12560e;
/// @dev Error code for when the evaluation length is too large.
uint32 constant ERR_EVALUATION_LENGTH_TOO_LARGE = 0xb65e7142;
/// @dev Error code for when a bit decomposition is invalid.
uint32 constant ERR_BIT_DECOMPOSITION_INVALID = 0xda443b2b;
/// @dev Error code for when bits that shouldn't vary do vary.
uint32 constant ERR_INVALID_VARYING_BITS = 0x76d56a3d;
/// @dev Error code for when monotony check fails.
uint32 constant ERR_MONOTONY_CHECK_FAILED = 0x976f97b8;
/// @dev Error code for when offset and plan value mismatch in slice.
uint32 constant ERR_SLICE_OFFSET_PLAN_VALUE_MISMATCH = 0x86052984;
/// @dev Error code for when offset and selection size mismatch in slice.
uint32 constant ERR_SLICE_OFFSET_SELECTION_SIZE_MISMATCH = 0xd66498ee;
/// @dev Error code for when the max length is incorrect in slice.
uint32 constant ERR_SLICE_MAX_LENGTH_MISMATCH = 0xd2e2f4a8;
/// @dev Error code for when there are not enough input plans in a union.
uint32 constant ERR_UNION_NOT_ENOUGH_INPUT_PLANS = 0x22d85efe;
/// @dev Error thrown when the number of column counts in a union is invalid (e.g. zero or inconsistent).
uint32 constant ERR_UNION_INVALID_COLUMN_COUNTS = 0x2b150620;
/// @dev Error thrown when there is an internal error.
uint32 constant ERR_INTERNAL = 0xfe835e35;
/// @dev Error code for unprovable group by error.
uint32 constant ERR_UNPROVABLE_GROUP_BY = 0x6bd33da2;
/// @dev Error code for when a plan has a number of join columns other than one.
uint32 constant ERR_NUMBER_OF_JOIN_COLUMNS_NOT_ONE = 0xf81ffd2a;

library Errors {
    /// @notice Error thrown when the inputs to the ECADD precompile are invalid.
    error InvalidECAddInputs();
    /// @notice Error thrown when the inputs to the ECMUL precompile are invalid.
    error InvalidECMulInputs();
    /// @notice Error thrown when the inputs to the ECPAIRING precompile are invalid.
    error InvalidECPairingInputs();
    /// @notice Error thrown when a boolean literal is invalid.
    error InvalidBoolean();
    /// @notice Error code for commitment array having odd length which is impossible
    /// since each commitment is 2 elements.
    error CommitmentArrayOddLength();
    /// @notice Error thrown when the size of a sumcheck proof is incorrect.
    error InvalidSumcheckProofSize();
    /// @notice Error thrown when the evaluation of a round in a sumcheck proof does not match the expected value.
    error RoundEvaluationMismatch();
    /// @notice Error thrown when a dequeue attempt was made on an empty queue.
    error EmptyQueue();
    /// @notice Error thrown when the HyperKZG proof has an inconsistent v.
    error HyperKZGInconsistentV();
    /// @notice Error thrown when the HyperKZG proof has an empty x point.
    error HyperKZGEmptyPoint();
    /// @notice Error thrown when the HyperKZG proof fails the pairing check.
    error HyperKZGPairingCheckFailed();
    /// @notice Error thrown when the produces constraint degree is higher than the provided proof.
    error ConstraintDegreeTooHigh();
    /// @notice Error thrown when the case literal in a switch statement is incorrect.
    error IncorrectCaseConst();
    /// @notice Error thrown when an index is invalid.
    error InvalidIndex();
    /// @notice Error thrown when a proof expression variant is unsupported.
    error UnsupportedProofExprVariant();
    /// @notice Error thrown when PCS batch lengths don't match.
    error PCSBatchLengthMismatch();
    /// @notice Error thrown when result column counts don't match.
    error ResultColumnCountMismatch();
    /// @notice Error thrown when a result column name is invalid.
    error InvalidResultColumnName();
    /// @notice Error thrown when result column lengths are inconsistent.
    error InconsistentResultColumnLengths();
    /// @notice Error thrown when the result is incorrect.
    error IncorrectResult();
    /// @notice Error thrown when HyperKZG proof size doesn't match.
    error HyperKZGProofSizeMismatch();
    /// @notice Error thrown when aggregate evaluation doesn't match.
    error AggregateEvaluationMismatch();
    /// @notice Error thrown when proof type is unsupported.
    error UnsupportedProof();
    /// @notice Error thrown when a proof plan variant is unsupported.
    error UnsupportedProofPlanVariant();
    /// @notice Error thrown when a data type variant is unsupported.
    error UnsupportedDataTypeVariant();
    /// @notice Error thrown when the evaluation length is too large.
    error EvaluationLengthTooLarge();
    /// @notice Error thrown when a bit decomposition is invalid.
    error BitDecompositionInvalid();
    /// @notice Error thrown when bits that shouldn't vary do vary.
    error InvalidVaryingBits();
    /// @notice Error thrown when monotony check fails.
    error MonotonyCheckFailed();
    /// @notice Error code for when offset and plan value mismatch in slice.
    error SliceOffsetPlanValueMismatch();
    /// @notice Error thrown when offset and selection size mismatch in slice.
    error SliceOffsetSelectionSizeMismatch();
    /// @notice Error thrown when the max length is incorrect in slice.
    error SliceMaxLengthMismatch();
    /// @notice Error thrown when there are not enough input plans in a union.
    error UnionNotEnoughInputPlans();
    /// @notice Error thrown when the number of column counts in a union is invalid (e.g. zero or inconsistent).
    error UnionInvalidColumnCounts();
    /// @notice Error thrown when there is an internal error.
    error InternalError();
    /// @notice Error thrown for unprovable group by error.
    error UnprovableGroupBy();
    /// @notice Error thrown when a plan has a number of join columns other than one.
    error NumberOfJoinColumnsNotOne();

    function __err(uint32 __code) internal pure {
        assembly {
            function err(code) {
                mstore(0, code)
                revert(28, 4)
            }
            err(__code)
        }
    }
}
