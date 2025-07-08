// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import {VerificationBuilder} from "../builder/VerificationBuilder.pre.sol";

/// @title EmptyExec
/// @dev Library for handling sort merge join execution plans
library SortMergeJoinExec {
    /// @notice Evaluates a sort merge join plan
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// sort_merge_join_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr, output_length, output_chi_eval
    /// ```
    /// ##### Parameters
    /// * `plan_ptr` - calldata pointer to the sort merge join execution plan data
    /// * `builder_ptr` - memory pointer to the verification builder
    /// ##### Return Values
    /// * `plan_ptr_out` - pointer to the remaining plan after consuming the sort merge join execution plan data
    /// * `evaluations_ptr` - pointer to the evaluations
    /// * `output_length` - the length of the column of ones
    /// * `output_chi_eval` - pointer to the evaluation of a column of 1s with same length as output
    /// @notice Evaluates a sort merge join execution plan
    /// @dev Evaluates a sort merge join execution plan
    /// @param __plan The sort merge join execution plan data
    /// @param __builder The verification builder
    /// @return __planOut The remaining plan after processing
    /// @return __builderOut The verification builder result
    /// @return __evaluationsPtr The evaluations pointer
    /// @return __outputLength The length of the output chi evaluation
    /// @return __outputChiEvaluation The output chi evaluation
    function __sortMergeJoinEvaluate( // solhint-disable-line gas-calldata-parameters
    bytes calldata __plan, VerificationBuilder.Builder memory __builder)
        external
        pure
        returns (
            bytes calldata __planOut,
            VerificationBuilder.Builder memory __builderOut,
            uint256[] memory __evaluationsPtr,
            uint256 __outputLength,
            uint256 __outputChiEvaluation
        )
    {
        assembly {
            function sort_merge_join_evaluate(plan_ptr, builder_ptr) ->
                plan_ptr_out,
                evaluations_ptr,
                output_length,
                output_chi_eval
            {}

            let __planOutOffset
            __planOutOffset, __evaluationsPtr, __outputLength, __outputChiEvaluation :=
                sort_merge_join_evaluate(__plan.offset, __builder)
            __planOut.offset := __planOutOffset
            // slither-disable-next-line write-after-write
            __planOut.length := sub(__plan.length, sub(__planOutOffset, __plan.offset))
        }
        __builderOut = __builder;
    }
}
