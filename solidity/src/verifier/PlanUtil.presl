// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";

/// @title Plan Utility Library
/// @notice A library for handling utility functions related to plans.
library PlanUtil {
    /// @notice The Proof Plan is prefixed with metadata about the plan, primarily the names of the tables and columns.
    /// @notice This method skips over the names in a plan and returns the updated pointer.
    /// @dev The format of the plan is as follows:
    /// @dev * number of tables (uint64)
    /// @dev * table names
    /// @dev     * length of table name (uint64)
    /// @dev     * table name (variable length)
    /// @dev * number of columns (uint64)
    /// @dev * column names
    /// @dev     * index of the table the column belongs to (uint64)
    /// @dev     * length of column name (uint64)
    /// @dev     * column name (variable length)
    /// @dev     * column type (variable length)
    /// @dev * number of output columns (uint64)
    /// @dev * output column names
    /// @dev     * length of output column name (uint64)
    /// @dev     * output column name (variable length)
    /// @param __plan The calldata pointer to the plan.
    /// @return __planOut The updated pointer after skipping names which points to the actual plan itself. i.e., the AST without any names.
    function __skipPlanNames(bytes calldata __plan) external pure returns (bytes calldata __planOut) {
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/SwitchUtil.pre.sol
            function case_const(lhs, rhs) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/DataType.pre.sol
            function read_data_type(ptr) -> ptr_out, data_type {
                revert(0, 0)
            }
            function skip_plan_names(plan_ptr) -> plan_ptr_out {
                // skip over the table names
                let num_tables := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                plan_ptr := add(plan_ptr, UINT64_SIZE)
                for {} num_tables { num_tables := sub(num_tables, 1) } {
                    let name_len := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                    plan_ptr := add(plan_ptr, add(UINT64_SIZE, name_len))
                }
                // skip over the column names
                let num_columns := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                plan_ptr := add(plan_ptr, UINT64_SIZE)
                for {} num_columns { num_columns := sub(num_columns, 1) } {
                    plan_ptr := add(plan_ptr, UINT64_SIZE)
                    let name_len := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                    plan_ptr := add(plan_ptr, add(UINT64_SIZE, name_len))
                    let data_type
                    plan_ptr, data_type := read_data_type(plan_ptr)
                }
                // skip over the output column names
                let num_outputs := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                plan_ptr := add(plan_ptr, UINT64_SIZE)
                for {} num_outputs { num_outputs := sub(num_outputs, 1) } {
                    let name_len := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                    plan_ptr := add(plan_ptr, add(UINT64_SIZE, name_len))
                }

                plan_ptr_out := plan_ptr
            }

            let __planOutOffset := skip_plan_names(__plan.offset)
            __planOut.offset := __planOutOffset
            // slither-disable-next-line write-after-write
            __planOut.length := sub(__plan.length, sub(__planOutOffset, __plan.offset))
        }
    }
}
