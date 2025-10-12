// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";
import {VerificationBuilder} from "../builder/VerificationBuilder.pre.sol";

/// @title Shift
/// @dev Library for handling column shifts
library Shift {
    /// @notice verifies that a shifted column is evaluated correctly
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// shift_evaluate(builder_ptr, alpha, beta, expr_eval, shifted_expr_eval, chi_eval, chi_plus_one_eval)
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the verification builder
    /// * `alpha` - a challenge
    /// * `beta` - a challenge
    /// * `expr_eval` - the expression evaluation
    /// * `chi_eval` - the chi evaluation with length same as the column
    /// ##### Return Values
    /// * `shifted_expr_eval` - the evaluation of the shifted expression
    /// * `chi_plus_one_eval` - the chi evaluation with length one longer than the column
    /// @notice verifies that a shifted column is evaluated correctly
    /// @param __builder The verification builder
    /// @param __alpha a challenge
    /// @param __beta a challenge
    /// @param __exprEval the expression evaluation
    /// @param __chiEval The chi value for evaluation
    /// @return __builderOut The verification builder result
    /// @return __shiftedExprEval the evaluation of the shifted expression
    /// @return __chiPlusOneEval The chi plus one value for evaluation
    function __shiftEvaluate( // solhint-disable-line gas-calldata-parameters
        VerificationBuilder.Builder memory __builder,
        uint256 __alpha,
        uint256 __beta,
        uint256 __exprEval,
        uint256 __chiEval
    )
        internal
        pure
        returns (VerificationBuilder.Builder memory __builderOut, uint256 __shiftedExprEval, uint256 __chiPlusOneEval)
    {
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Queue.pre.sol
            function dequeue(queue_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Queue.pre.sol
            function dequeue_uint512(queue_ptr) -> upper, lower {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Array.pre.sol
            function get_array_element(arr_ptr, index) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_first_round_mle(builder_ptr) -> value {
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
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_rho_evaluation(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_chi_evaluation(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/MathUtil.pre.sol
            function mulmod_bn254(lhs, rhs) -> product {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/MathUtil.pre.sol
            function addmod_bn254(lhs, rhs) -> difference {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/MathUtil.pre.sol
            function submod_bn254(lhs, rhs) -> product {
                revert(0, 0)
            }

            function compute_shift_identity_constraint(star, chi_plus_one, fold) -> constraint {
                constraint := addmod_bn254(submod_bn254(star, chi_plus_one), mulmod_bn254(fold, star))
            }

            function compute_shift_fold(alpha, beta, eval, rho) -> fold {
                fold := mulmod_bn254(alpha, addmod_bn254(mulmod_bn254(beta, rho), eval))
            }

            function shift_evaluate(builder_ptr, alpha, beta, expr_eval, chi_eval) ->
                shifted_expr_eval,
                chi_plus_one_eval
            {
                chi_plus_one_eval := builder_consume_chi_evaluation(builder_ptr)
                shifted_expr_eval := builder_consume_first_round_mle(builder_ptr)
                let rho_eval := builder_consume_rho_evaluation(builder_ptr)
                let rho_plus_one_eval := builder_consume_rho_evaluation(builder_ptr)
                let c_star_eval := builder_consume_final_round_mle(builder_ptr)
                let d_star_eval := builder_consume_final_round_mle(builder_ptr)
                // sum c_star - d_star = 0
                builder_produce_zerosum_constraint(builder_ptr, submod_bn254(c_star_eval, d_star_eval), 1)
                // c_star + c_fold * c_star - chi_n_plus_1 = 0
                {
                    let c_fold := compute_shift_fold(alpha, beta, expr_eval, addmod_bn254(rho_eval, chi_eval))
                    builder_produce_identity_constraint(
                        builder_ptr, compute_shift_identity_constraint(c_star_eval, chi_plus_one_eval, c_fold), 2
                    )
                }
                // d_star + d_fold * d_star - chi_n_plus_1 = 0
                {
                    let d_fold := compute_shift_fold(alpha, beta, shifted_expr_eval, rho_plus_one_eval)
                    builder_produce_identity_constraint(
                        builder_ptr, compute_shift_identity_constraint(d_star_eval, chi_plus_one_eval, d_fold), 2
                    )
                }
            }

            __shiftedExprEval, __chiPlusOneEval := shift_evaluate(__builder, __alpha, __beta, __exprEval, __chiEval)
        }
        __builderOut = __builder;
    }
}
