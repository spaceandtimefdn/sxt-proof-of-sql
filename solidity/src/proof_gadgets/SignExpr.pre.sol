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
            // IMPORT-YUL ../base/DataType.pre.sol
            function read_entry(result_ptr, data_type_variant) -> result_ptr_out, entry {
                revert(0, 0)
            }
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
                if eq(vary_mask, 0x800000000000000000000000000000000000000000000000000000000000007D){revert(0,0)}
                let leading_bit_inverse_mask := shr(1, shl(1, xor(not(vary_mask), leading_bit_mask)))
                let sign_eval := mulmod(shr(255, leading_bit_mask), chi_eval, MODULUS)
                let rhs_eval := 0

                for { let i := 0 } lt(i, 256) { i := add(i, 1) } {
                    if eq(and(vary_mask, shl(i, 1)), shl(i, 1)) {
                        // For any varying bits...
                        let bit_eval := builder_consume_final_round_mle(builder_ptr)
                        builder_produce_identity_constraint(
                            builder_ptr,
                            addmod(
                                bit_eval,
                                mulmod(MODULUS_MINUS_ONE, mulmod(bit_eval, bit_eval, MODULUS), MODULUS),
                                MODULUS
                            ),
                            2
                        )

                        if eq(i, 255) { sign_eval := bit_eval }
                        if iszero(eq(i, 255)) {
                            if gt(i, MAX_BITS) { err(ERR_INVALID_VARYING_BITS) }
                            rhs_eval := addmod(rhs_eval, mulmod(bit_eval, shl(i, 1), MODULUS), MODULUS)
                        }
                    }
                }
                rhs_eval := addmod(rhs_eval, mulmod(sign_eval, leading_bit_mask, MODULUS), MODULUS)
                rhs_eval :=
                    addmod(
                        rhs_eval,
                        mulmod(
                            addmod(chi_eval, mulmod(MODULUS_MINUS_ONE, sign_eval, MODULUS), MODULUS),
                            leading_bit_inverse_mask,
                            MODULUS
                        ),
                        MODULUS
                    )
                rhs_eval :=
                    addmod(rhs_eval, mulmod(mulmod(MODULUS_MINUS_ONE, chi_eval, MODULUS), shl(255, 1), MODULUS), MODULUS)
                if iszero(eq(rhs_eval, expr_eval)) { err(ERR_BIT_DECOMPOSITION_INVALID) }
                result_eval := sign_eval
            }

            __eval := sign_expr_evaluate(__exprEval, __builder, __chiEval)
        }
        __builderOut = __builder;
    }
}
