// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";

/// @title EmptyExec
/// @dev Library for handling empty execution plans
library EmptyExec {
    /// @notice Evaluates an empty execution plan
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// empty_exec_evaluate() -> evaluations_ptr
    /// ```
    /// ##### Return Values
    /// * `evaluations_ptr` - pointer to the evaluations
    /// @notice Evaluates an empty execution plan
    /// @dev Evaluates an empty execution plan by returning an empty collection of evaluations
    /// @return __evaluationsPtr The evaluations pointer
    function __emptyExecEvaluate() external pure returns (uint256[] memory __evaluationsPtr) {
        assembly {
            function empty_exec_evaluate() -> evaluations_ptr {
                evaluations_ptr := mload(FREE_PTR)
                mstore(evaluations_ptr, 0)
                mstore(FREE_PTR, add(FREE_PTR, WORD_SIZE))
            }

            __evaluationsPtr := empty_exec_evaluate()
        }
    }
}
