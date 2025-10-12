// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";
import {VerificationBuilder} from "../builder/VerificationBuilder.pre.sol";

/// @title TableExec
/// @dev Library for handling table execution plans
library TableExec {
    /// @notice Evaluates a table execution plan
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// table_exec_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr, output_chi_eval
    /// ```
    /// ##### Parameters
    /// * `plan_ptr` - calldata pointer to the table execution plan
    /// * `builder_ptr` - memory pointer to the verification builder
    /// ##### Return Values
    /// * `plan_ptr_out` - pointer to the remaining plan after consuming the table execution plan
    /// * `evaluations_ptr` - pointer to the evaluations
    /// * `output_length` - the length of the column of ones
    /// * `output_chi_eval` - pointer to the evaluation of a column of 1s with same length as output
    /// @notice Evaluates a table execution plan
    /// @notice ##### Table execution plan
    /// The table execution plan is a representation of a table query source, such as `SELECT col from tab`.
    /// The plan accesses a table directly and returns its evaluations.
    /// @dev Evaluates a table execution plan by loading the specified table
    /// @param __plan The table execution plan data
    /// @param __builder The verification builder
    /// @return __planOut The remaining plan after processing
    /// @return __builderOut The verification builder result
    /// @return __evaluationsPtr The evaluations pointer
    /// @return __outputLength The length of the output chi evaluation
    /// @return __outputChiEvaluation The output chi evaluation
    function __tableExecEvaluate( // solhint-disable-line gas-calldata-parameters
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
        uint256[] memory __evaluations;
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Queue.pre.sol
            function dequeue(queue_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_table_chi_evaluation(builder_ptr, table_num) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Array.pre.sol
            function get_array_element(arr_ptr, index) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Array.pre.sol
            function get_uint512_array_element(arr_ptr, index) -> upper, lower {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_column_evaluation(builder_ptr, column_num) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_table_chi_evaluation_with_length(builder_ptr, table_num) -> length, chi_eval {
                revert(0, 0)
            }

            function table_exec_evaluate(plan_ptr, builder_ptr) ->
                plan_ptr_out,
                evaluations_ptr,
                output_length,
                output_chi_eval
            {
                let table_number := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                plan_ptr := add(plan_ptr, UINT64_SIZE)
                output_length, output_chi_eval :=
                    builder_get_table_chi_evaluation_with_length(builder_ptr, table_number)

                // Get the number of columns in the schema
                let column_count := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                let copy_size := add(WORD_SIZE, mul(column_count, WORD_SIZE))
                plan_ptr := add(plan_ptr, UINT64_SIZE)

                // Initialize evaluations array to store column evaluations
                evaluations_ptr := mload(FREE_PTR)
                mstore(evaluations_ptr, column_count)

                // Read column evaluations for each field in the schema
                for {} column_count { column_count := sub(column_count, 1) } {
                    evaluations_ptr := add(evaluations_ptr, WORD_SIZE)
                    // For each column in schema, get its column number/index
                    let column_num := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                    plan_ptr := add(plan_ptr, UINT64_SIZE)

                    // Get the column evaluation from the builder
                    let column_eval := builder_get_column_evaluation(builder_ptr, column_num)

                    // Store the column evaluation in the result
                    mstore(evaluations_ptr, column_eval)
                }

                // Reset evaluations_ptr to the beginning of the array
                evaluations_ptr := mload(FREE_PTR)
                // Update free memory pointer
                mstore(FREE_PTR, add(evaluations_ptr, copy_size))

                plan_ptr_out := plan_ptr
            }

            let __planOutOffset
            __planOutOffset, __evaluations, __outputLength, __outputChiEvaluation :=
                table_exec_evaluate(__plan.offset, __builder)
            __planOut.offset := __planOutOffset
            // slither-disable-next-line write-after-write
            __planOut.length := sub(__plan.length, sub(__planOutOffset, __plan.offset))
        }
        __evaluationsPtr = __evaluations;
        __builderOut = __builder;
    }
}
