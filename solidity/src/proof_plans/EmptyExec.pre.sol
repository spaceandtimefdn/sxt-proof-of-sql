// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import {VerificationBuilder} from "../builder/VerificationBuilder.pre.sol";

/// @title EmptyExec
/// @dev Library for handling empty execution plans
library EmptyExec {
    /// @notice Evaluates an empty execution plan
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// empty_exec_evaluate(builder_ptr) -> evaluations_ptr, output_chi_eval
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the verification builder
    /// ##### Return Values
    /// * `evaluations_ptr` - pointer to the evaluations
    /// * `output_chi_eval` - pointer to the evaluation of a column of 1s with same length as output
    /// @notice Evaluates an empty execution plan
    /// @dev Evaluates an empty execution plan by returning an empty collection of evaluations
    /// @param __builder The verification builder
    /// @return __builderOut The verification builder result
    /// @return __evaluationsPtr The evaluations pointer
    /// @return __outputChiEvaluation The output chi evaluation
    function __emptyExecEvaluate(VerificationBuilder.Builder memory __builder)
        external
        pure
        returns (
            VerificationBuilder.Builder memory __builderOut,
            uint256[] memory __evaluationsPtr,
            uint256 __outputChiEvaluation
        )
    {
        assembly {
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_singleton_chi_evaluation(builder_ptr) -> value {
                revert(0, 0)
            }
            function empty_exec_evaluate(builder_ptr) -> evaluations_ptr, output_chi_eval {
                evaluations_ptr := mload(FREE_PTR)
                mstore(evaluations_ptr, 0)
                mstore(FREE_PTR, add(evaluations_ptr, WORD_SIZE))
                output_chi_eval := builder_get_singleton_chi_evaluation(builder_ptr)
            }

            __evaluationsPtr, __outputChiEvaluation := empty_exec_evaluate(__builder)
        }
        __builderOut = __builder;
    }
}
