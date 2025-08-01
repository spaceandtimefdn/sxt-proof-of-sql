// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";
import {VerificationBuilder} from "../builder/VerificationBuilder.pre.sol";

/// @title FilterBase
/// @dev Library for handling common filter functionality
library FilterBase {
    /// @notice Verifies the filtered columns
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// verify_filter(builder_ptr, c_fold, d_fold, input_chi_eval, output_chi_eval, selection_eval)
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the verification builder
    /// * `c_fold` - the folded value of the input columns
    /// * `d_fold` - the folded value of the filtered columns
    /// * `input_chi_eval` - the column of ones with same length as the input columns
    /// * `output_chi_eval` - column of ones with same length as the filtered columns
    /// * `selection_eval` - column of ones and zeroes which indicate which rows to include in the result
    /// @param __builder The verification builder
    /// @param __cFold The folded value of the input columns
    /// @param __dFold The folded value of the filtered columns
    /// @param __inputChiEval The column of ones with same length as the input columns
    /// @param __outputChiEval The column of ones with same length as the filtered columns
    /// @param __selectionEval The column of ones and zeroes which indicate which rows to include in the result
    /// @return __builderOut The updated verification builder
    function __filterBaseEvaluate( // solhint-disable-line gas-calldata-parameters
        VerificationBuilder.Builder memory __builder,
        uint256 __cFold,
        uint256 __dFold,
        uint256 __inputChiEval,
        uint256 __outputChiEval,
        uint256 __selectionEval
    ) external pure returns (VerificationBuilder.Builder memory __builderOut) {
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/MathUtil.pre.sol
            function addmod_bn254(lhs, rhs) -> sum {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/MathUtil.pre.sol
            function submod_bn254(lhs, rhs) -> difference {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/MathUtil.pre.sol
            function mulmod_bn254(lhs, rhs) -> product {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Queue.pre.sol
            function dequeue(queue_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_final_round_mle(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_produce_identity_constraint(builder_ptr, evaluation, degree) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_produce_zerosum_constraint(builder_ptr, evaluation, degree) {
                revert(0, 0)
            }
            function verify_filter(builder_ptr, c_fold, d_fold, input_chi_eval, output_chi_eval, selection_eval) {
                let c_star := builder_consume_final_round_mle(builder_ptr)
                let d_star := builder_consume_final_round_mle(builder_ptr)

                builder_produce_identity_constraint(
                    builder_ptr, submod_bn254(mulmod_bn254(addmod_bn254(1, c_fold), c_star), input_chi_eval), 2
                )
                builder_produce_identity_constraint(
                    builder_ptr, submod_bn254(mulmod_bn254(addmod_bn254(1, d_fold), d_star), output_chi_eval), 2
                )
                builder_produce_zerosum_constraint(
                    builder_ptr, submod_bn254(mulmod_bn254(c_star, selection_eval), d_star), 2
                )
                builder_produce_identity_constraint(
                    builder_ptr, mulmod_bn254(d_fold, submod_bn254(output_chi_eval, 1)), 2
                )
            }

            verify_filter(__builder, __cFold, __dFold, __inputChiEval, __outputChiEval, __selectionEval)
        }
        __builderOut = __builder;
    }
}
