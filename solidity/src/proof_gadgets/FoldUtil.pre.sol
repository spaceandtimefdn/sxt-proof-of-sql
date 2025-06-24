// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";
import {VerificationBuilder} from "../builder/VerificationBuilder.pre.sol";

/// @title FoldUtil
/// @dev Library for handling fold operations
library FoldUtil {
    /// @notice Folds expression evaluations with beta challenge
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// fold_expr_evals(plan_ptr, builder_ptr, input_chi_eval, beta, column_count) -> plan_ptr_out, fold
    /// ```
    /// ##### Parameters
    /// * `plan_ptr` - calldata pointer to the plan
    /// * `builder_ptr` - memory pointer to the verification builder
    /// * `input_chi_eval` - input chi evaluation
    /// * `beta` - challenge value
    /// * `column_count` - number of columns to process
    /// ##### Return Values
    /// * `plan_ptr_out` - pointer to the remaining plan after consuming
    /// * `fold` - computed fold value
    /// @param __plan The plan data
    /// @param __builder The verification builder
    /// @param __inputChiEval The input chi evaluation
    /// @param __beta The beta challenge value
    /// @param __columnCount The number of columns
    /// @return __planOut The remaining plan after processing
    /// @return __builderOut The updated verification builder
    /// @return __fold The computed fold value
    function __foldExprEvals( // solhint-disable-line gas-calldata-parameters
        bytes calldata __plan,
        VerificationBuilder.Builder memory __builder,
        uint256 __inputChiEval,
        uint256 __beta,
        uint256 __columnCount
    )
        external
        pure
        returns (bytes calldata __planOut, VerificationBuilder.Builder memory __builderOut, uint256 __fold)
    {
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
            // IMPORT-YUL ../base/Queue.pre.sol
            function dequeue_uint512(queue_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/SwitchUtil.pre.sol
            function case_const(lhs, rhs) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_column_evaluation(builder_ptr, column_num) -> eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Array.pre.sol
            function get_array_element(arr_ptr, index) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/ColumnExpr.pre.sol
            function column_expr_evaluate(expr_ptr, builder_ptr) -> expr_ptr_out, eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/LiteralExpr.pre.sol
            function literal_expr_evaluate(expr_ptr, chi_eval) -> expr_ptr_out, eval {
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
            // IMPORT-YUL ../proof_exprs/EqualsExpr.pre.sol
            function equals_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/AddExpr.pre.sol
            function add_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/SubtractExpr.pre.sol
            function subtract_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/MultiplyExpr.pre.sol
            function multiply_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/AndExpr.pre.sol
            function and_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/OrExpr.pre.sol
            function or_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/NotExpr.pre.sol
            function not_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/CastExpr.pre.sol
            function cast_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_bit_distribution(builder_ptr) -> vary_mask, leading_bit_mask {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/SignExpr.pre.sol
            function sign_expr_evaluate(expr_eval, builder_ptr, chi_eval) -> result_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/InequalityExpr.pre.sol
            function inequality_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/InequalityExpr.pre.sol
            function placeholder_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                revert(0, 0)
            }
            // slither-disable-start cyclomatic-complexity
            // IMPORT-YUL ../proof_exprs/ProofExpr.pre.sol
            function proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                revert(0, 0)
            }
            // slither-disable-end cyclomatic-complexity
            function fold_expr_evals(plan_ptr, builder_ptr, input_chi_eval, beta, column_count) -> plan_ptr_out, fold {
                fold := 0
                for {} column_count { column_count := sub(column_count, 1) } {
                    let expr_eval
                    plan_ptr, expr_eval := proof_expr_evaluate(plan_ptr, builder_ptr, input_chi_eval)
                    fold := addmod_bn254(mulmod_bn254(fold, beta), expr_eval)
                }
                plan_ptr_out := plan_ptr
            }

            let __planOutOffset
            __planOutOffset, __fold := fold_expr_evals(__plan.offset, __builder, __inputChiEval, __beta, __columnCount)
            // slither-disable-start write-after-write
            __planOut.offset := __planOutOffset
            __planOut.length := sub(__plan.length, sub(__planOutOffset, __plan.offset))
            // slither-disable-end write-after-write
        }
        __builderOut = __builder;
    }

    /// @notice Folds final round MLEs with beta challenge
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// fold_final_round_mles(builder_ptr, column_count, beta) -> fold, evaluations_ptr
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the verification builder
    /// * `beta` - challenge value
    /// * `column_count` - number of columns to process
    /// ##### Return Values
    /// * `fold` - computed fold value
    /// * `evaluations_ptr` - pointer to the evaluations array
    /// @param __builder The verification builder
    /// @param __beta The beta challenge value
    /// @param __columnCount The number of columns
    /// @return __builderOut The updated verification builder
    /// @return __fold The computed fold value
    /// @return __evaluations The evaluations array
    function __foldFinalRoundMles( // solhint-disable-line gas-calldata-parameters
    VerificationBuilder.Builder memory __builder, uint256 __beta, uint256 __columnCount)
        external
        pure
        returns (VerificationBuilder.Builder memory __builderOut, uint256 __fold, uint256[] memory __evaluations)
    {
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

            function fold_final_round_mles(builder_ptr, beta, column_count) -> fold, evaluations_ptr {
                evaluations_ptr := mload(FREE_PTR)
                mstore(evaluations_ptr, column_count)
                evaluations_ptr := add(evaluations_ptr, WORD_SIZE)
                fold := 0
                for { let i := column_count } i { i := sub(i, 1) } {
                    let mle := builder_consume_final_round_mle(builder_ptr)
                    fold := addmod_bn254(mulmod_bn254(fold, beta), mle)
                    mstore(evaluations_ptr, mle)
                    evaluations_ptr := add(evaluations_ptr, WORD_SIZE)
                }
                evaluations_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(evaluations_ptr, add(WORD_SIZE, mul(column_count, WORD_SIZE))))
            }

            let __evaluationsPtr
            __fold, __evaluationsPtr := fold_final_round_mles(__builder, __beta, __columnCount)
            __evaluations := __evaluationsPtr
        }
        __builderOut = __builder;
    }

    /// @notice Folds column expression evaluations with beta challenge
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// fold_column_expr_evals(plan_ptr, builder_ptr, beta, column_count) -> plan_ptr_out, fold
    /// ```
    /// ##### Parameters
    /// * `plan_ptr` - calldata pointer to the plan
    /// * `builder_ptr` - memory pointer to the verification builder
    /// * `beta` - challenge value
    /// * `column_count` - number of columns to process
    /// ##### Return Values
    /// * `plan_ptr_out` - pointer to the remaining plan after consuming
    /// * `fold` - computed fold value
    /// @param __plan The plan data
    /// @param __builder The verification builder
    /// @param __beta The beta challenge value
    /// @param __columnCount The number of columns
    /// @return __planOut The remaining plan after processing
    /// @return __builderOut The updated verification builder
    /// @return __fold The computed fold value
    function __foldColumnExprEvals( // solhint-disable-line gas-calldata-parameters
    bytes calldata __plan, VerificationBuilder.Builder memory __builder, uint256 __beta, uint256 __columnCount)
        external
        pure
        returns (bytes calldata __planOut, VerificationBuilder.Builder memory __builderOut, uint256 __fold)
    {
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
            function mulmod_bn254(lhs, rhs) -> product {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Array.pre.sol
            function get_array_element(arr_ptr, index) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_column_evaluation(builder_ptr, column_num) -> eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/ColumnExpr.pre.sol
            function column_expr_evaluate(expr_ptr, builder_ptr) -> expr_ptr_out, eval {
                revert(0, 0)
            }

            function fold_column_expr_evals(plan_ptr, builder_ptr, beta, column_count) -> plan_ptr_out, fold {
                fold := 0
                for {} column_count { column_count := sub(column_count, 1) } {
                    let expr_eval
                    plan_ptr, expr_eval := column_expr_evaluate(plan_ptr, builder_ptr)
                    fold := addmod_bn254(mulmod_bn254(fold, beta), expr_eval)
                }
                plan_ptr_out := plan_ptr
            }

            let __planOutOffset
            __planOutOffset, __fold := fold_column_expr_evals(__plan.offset, __builder, __beta, __columnCount)
            // slither-disable-start write-after-write
            __planOut.offset := __planOutOffset
            __planOut.length := sub(__plan.length, sub(__planOutOffset, __plan.offset))
            // slither-disable-end write-after-write
        }
        __builderOut = __builder;
    }
}
