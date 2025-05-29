// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";

/// @title LiteralExpr
/// @dev Library for handling literal expressions
library LiteralExpr {
    /// @notice Evaluates a literal expression
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// literal_expr_evaluate(expr_ptr, chi_eval) -> expr_ptr_out, eval
    /// ```
    /// ##### Parameters
    /// * `expr_ptr` - the calldata pointer to the beginning of the expression data
    /// * `chi_eval` - the chi value for evaluation
    /// ##### Return Values
    /// * `expr_ptr_out` - the pointer to the remaining expression after consuming the literal expression
    /// * `eval` - the evaluated result
    /// ##### Proof Plan Encoding
    /// The literal expression is encoded as follows:
    /// 1. The literal variant (as a uint32)
    /// 2. The literal value, which is variant-specific
    ///     a. BigInt: The literal value as a signed int64
    ///     b. Other variants are unsupported at this time
    /// @dev This function evaluates a literal expression by multiplying the literal value by chi_eval.
    /// This is because `chi_eval` is the evaluation of a column of ones of the appropriate length.
    /// @param __expr The literal expression data
    /// @param __chiEval The chi value for evaluation
    /// @return __exprOut The remaining expression data after processing
    /// @return __eval The evaluated result
    function __literalExprEvaluate(bytes calldata __expr, uint256 __chiEval)
        external
        pure
        returns (bytes calldata __exprOut, uint256 __eval)
    {
        assembly {
            // IMPORT-YUL ../base/MathUtil.sol
            function addmod_bn254(lhs, rhs) -> sum {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/MathUtil.sol
            function submod_bn254(lhs, rhs) -> difference {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/MathUtil.sol
            function mulmod_bn254(lhs, rhs) -> product {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/SwitchUtil.pre.sol
            function case_const(lhs, rhs) {
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

            function literal_expr_evaluate(expr_ptr, chi_eval) -> expr_ptr_out, eval {
                let literal_variant
                expr_ptr, literal_variant := read_data_type(expr_ptr)
                expr_ptr, eval := read_entry(expr_ptr, literal_variant)
                eval := mulmod_bn254(eval, chi_eval)
                expr_ptr_out := expr_ptr
            }
            let __exprOutOffset
            __exprOutOffset, __eval := literal_expr_evaluate(__expr.offset, __chiEval)
            __exprOut.offset := __exprOutOffset
            // slither-disable-next-line write-after-write
            __exprOut.length := sub(__expr.length, sub(__exprOutOffset, __expr.offset))
        }
    }
}
