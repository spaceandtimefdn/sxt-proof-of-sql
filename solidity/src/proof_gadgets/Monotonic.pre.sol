// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";
import {VerificationBuilder} from "../builder/VerificationBuilder.pre.sol";

/// @title Monotonic
/// @dev Library for verifying monotonic columns (increasing or decreasing, strictly or non-strictly)
library Monotonic {
    /// @notice Verifies that a column is monotonic (increasing or decreasing)
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// monotonic_verify(builder_ptr, alpha, beta, column_eval, chi_eval, strict, asc)
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the verification builder
    /// * `alpha` - a challenge
    /// * `beta` - a challenge
    /// * `column_eval` - the column evaluation
    /// * `chi_eval` - the chi evaluation with length same as the column
    /// * `strict` - whether monotonicity is strict (1) or non-strict (0)
    /// * `asc` - whether monotonicity is ascending (1) or descending (0)
    /// @param __builder The verification builder
    /// @param __alpha a challenge
    /// @param __beta a challenge
    /// @param __columnEval the column evaluation
    /// @param __chiEval The chi value for evaluation
    /// @param __strict Whether monotonicity is strict (1) or non-strict (0)
    /// @param __asc Whether monotonicity is ascending (1) or descending (0)
    /// @return __builderOut The verification builder result
    function __monotonicVerify( // solhint-disable-line gas-calldata-parameters
        VerificationBuilder.Builder memory __builder,
        uint256 __alpha,
        uint256 __beta,
        uint256 __columnEval,
        uint256 __chiEval,
        uint256 __strict,
        uint256 __asc
    ) internal pure returns (VerificationBuilder.Builder memory __builderOut) {
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
            function builder_consume_final_round_mle(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_bit_distribution(builder_ptr) -> vary_mask, leading_bit_mask {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_chi_evaluation(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_singleton_chi_evaluation(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_rho_evaluation(builder_ptr) -> value {
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
            // IMPORT-YUL ../base/MathUtil.pre.sol
            function mulmod_bn254(lhs, rhs) -> product {
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
            // IMPORT-YUL ./Shift.pre.sol
            function compute_shift_identity_constraint(star, chi_plus_one, fold) -> constraint {
                revert(0, 0)
            }
            // IMPORT-YUL ./Shift.pre.sol
            function compute_shift_fold(alpha, beta, eval, rho) -> fold {
                revert(0, 0)
            }
            // IMPORT-YUL ./Shift.pre.sol
            function shift_evaluate(builder_ptr, alpha, beta, expr_eval, chi_eval) ->
                shifted_expr_eval,
                chi_plus_one_eval
            {
                revert(0, 0)
            }
            // IMPORT-YUL ./SignExpr.pre.sol
            function sign_expr_evaluate(expr_eval, builder_ptr, chi_eval) -> result_eval {
                revert(0, 0)
            }

            function monotonic_verify(builder_ptr, alpha, beta, column_eval, chi_eval, strict, asc) {
                // 1. Verify that `shifted_column` is a shift of `column`
                let shifted_column_eval, shifted_chi_eval :=
                    shift_evaluate(builder_ptr, alpha, beta, column_eval, chi_eval)

                // 2. Compute indicator evaluation based on strictness and direction
                let ind_eval
                switch eq(strict, asc)
                case 1 {
                    // (strict && asc) || (!strict && !asc): ind = shifted_column - column
                    ind_eval := submod_bn254(shifted_column_eval, column_eval)
                }
                default {
                    // (!strict && asc) || (strict && !asc): ind = column - shifted_column
                    ind_eval := submod_bn254(column_eval, shifted_column_eval)
                }

                // 3. Verify the sign of `ind`
                let sign_eval := sign_expr_evaluate(ind_eval, builder_ptr, shifted_chi_eval)
                let singleton_chi_eval := builder_get_singleton_chi_evaluation(builder_ptr)

                // 4. Check if sign_eval is in allowed evaluations
                let is_valid := 0
                switch strict
                case 1 {
                    // Strict monotonicity: sign(ind) == 1 for all but first and last element
                    // Allowed evaluations: chi_eval, shifted_chi_eval - singleton_chi_eval, chi_eval - singleton_chi_eval
                    is_valid :=
                        or(
                            or(eq(sign_eval, chi_eval), eq(sign_eval, submod_bn254(shifted_chi_eval, singleton_chi_eval))),
                            eq(sign_eval, submod_bn254(chi_eval, singleton_chi_eval))
                        )
                }
                default {
                    // Non-strict monotonicity: sign(ind) == 0 for all but first and last element
                    // Allowed evaluations: singleton_chi_eval, shifted_chi_eval - chi_eval,
                    // singleton_chi_eval + shifted_chi_eval - chi_eval, 0
                    is_valid :=
                        or(
                            or(eq(sign_eval, singleton_chi_eval), eq(sign_eval, submod_bn254(shifted_chi_eval, chi_eval))),
                            or(
                                eq(sign_eval, submod_bn254(addmod_bn254(singleton_chi_eval, shifted_chi_eval), chi_eval)),
                                iszero(sign_eval)
                            )
                        )
                }

                if iszero(is_valid) { err(ERR_MONOTONY_CHECK_FAILED) }
            }

            monotonic_verify(__builder, __alpha, __beta, __columnEval, __chiEval, __strict, __asc)
        }
        __builderOut = __builder;
    }
}
