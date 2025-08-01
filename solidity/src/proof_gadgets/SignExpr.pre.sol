// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";
import {VerificationBuilder} from "../builder/VerificationBuilder.pre.sol";

/// @title SignExpr
/// @dev Library for handling the sign of an evaluation
library SignExpr {
    /// @notice Evaluates a sign expression by finding the sign of an expression evaluation
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// sign_expr_evaluate(expr_eval, builder_ptr, chi_eval) -> eval
    /// ```
    /// ##### Parameters
    /// * `expr_eval` - the expression evaluation
    /// * `builder_ptr` - memory pointer to the verification builder
    /// * `chi_eval` - the chi value for evaluation
    /// ##### Return Values
    /// * `eval` - the evaluation result from the builder's final round MLE (or bit distribution)
    /// @notice Evaluates the sign of an expression
    /// ##### Proof Plan Encoding
    /// The sign expression is encoded as follows:
    /// The expression
    /// @param __exprEval The expression evaluation
    /// @param __builder The verification builder
    /// @param __chiEval The chi value for evaluation
    /// @return __builderOut The verification builder result
    /// @return __eval The evaluated result
    function __signExprEvaluate( // solhint-disable-line gas-calldata-parameters
    uint256 __exprEval, VerificationBuilder.Builder memory __builder, uint256 __chiEval)
        internal
        pure
        returns (VerificationBuilder.Builder memory __builderOut, uint256 __eval)
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
            function dequeue_uint512(queue_ptr) -> value {
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
            // IMPORT-YUL ../base/SwitchUtil.pre.sol
            function case_const(lhs, rhs) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_final_round_mle(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Array.pre.sol
            function get_array_element(arr_ptr, index) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_bit_distribution(builder_ptr) -> vary_mask, leading_bit_mask {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_produce_identity_constraint(builder_ptr, evaluation, degree) {
                revert(0, 0)
            }
            // slither-disable-start cyclomatic-complexity
            // IMPORT-YUL ../base/DataType.pre.sol
            function read_entry(result_ptr, data_type_variant) -> result_ptr_out, entry {
                revert(0, 0)
            }
            // slither-disable-end cyclomatic-complexity
            // IMPORT-YUL ../base/DataType.pre.sol
            function read_binary(result_ptr) -> result_ptr_out, entry {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/DataType.pre.sol
            function read_data_type(ptr) -> ptr_out, data_type {
                revert(0, 0)
            }

            function sign_expr_evaluate(expr_eval, builder_ptr, chi_eval) -> result_eval {
                let vary_mask
                let leading_bit_mask
                vary_mask, leading_bit_mask := builder_consume_bit_distribution(builder_ptr)

                // Other than the lead bit, no bit should vary past some max bit position, depending on the field
                if and(vary_mask, MODULUS_INVALID_VARY_MASK) { err(ERR_INVALID_VARYING_BITS) }

                // The lead bit of the leading_bit_mask dictates the sign, if it's constant sign.
                // So this will be the value if sign is constant. Otherwise, it will be overwritten
                let sign_eval := mulmod_bn254(shr(255, leading_bit_mask), chi_eval)

                // For future computations, leading_bit_mask should have a 1 in the lead bit
                leading_bit_mask := or(leading_bit_mask, shl(255, 1))

                // leading_bit_inverse_mask identifies columns that match the inverse of the lead bit column
                // So !vary_mask ^ leading_bit_mask, with a lead bit of zero.
                let leading_bit_inverse_mask := shr(1, shl(1, xor(not(vary_mask), leading_bit_mask)))

                // sum_eval should ultimately add up to the original column of data
                // It will effectively be a recomposition of the bit decomposition
                let sum_eval := 0

                for { let i := 0 } vary_mask {
                    i := add(i, 1)
                    vary_mask := shr(1, vary_mask)
                } {
                    if and(vary_mask, 1) {
                        // For any varying bits...
                        let bit_eval := builder_consume_final_round_mle(builder_ptr)

                        // Verify that every eval is a bit
                        // bit_eval - bit_eval * bit_eval = 0
                        builder_produce_identity_constraint(
                            builder_ptr, submod_bn254(bit_eval, mulmod_bn254(bit_eval, bit_eval)), 2
                        )

                        switch i
                        // If the lead bit varies, that we get the sign from the mles.
                        case 255 { sign_eval := bit_eval }
                        // For varying non lead bits,
                        // we add bit_eval * 2ⁱ to the sum in order to recompose the original value of the column
                        default { sum_eval := addmod_bn254(sum_eval, mulmod_bn254(bit_eval, shl(i, 1))) }
                    }
                }

                result_eval := submod_bn254(chi_eval, sign_eval)

                // For constant and lead bits...
                // sum += sign_eval * leading_bit_mask + (sign_eval - chi_eval) * leading_bit_inverse_mask - chi_eval * (1 << 255)
                sum_eval :=
                    submod_bn254(
                        addmod_bn254(
                            addmod_bn254(sum_eval, mulmod_bn254(sign_eval, leading_bit_mask)),
                            mulmod_bn254(result_eval, leading_bit_inverse_mask)
                        ),
                        mulmod_bn254(chi_eval, shl(255, 1))
                    )

                // Verify the bit recomposition matches the original column evaluation
                if sub(sum_eval, expr_eval) { err(ERR_BIT_DECOMPOSITION_INVALID) }
            }

            __eval := sign_expr_evaluate(__exprEval, __builder, __chiEval)
        }
        __builderOut = __builder;
    }
}
