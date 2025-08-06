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
    /// function verify_filter_from_column_evals(builder_ptr, alpha, beta, column_evals, input_chi_eval, output_chi_eval, selection_eval) -> filtered_columns
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the verification builder
    /// * `column_evals` - the input columns
    /// * `input_chi_eval` - the column of ones with same length as the input columns
    /// * `output_chi_eval` - column of ones with same length as the filtered columns
    /// * `selection_eval` - column of ones and zeroes which indicate which rows to include in the result
    /// ##### Return Values
    /// * `filtered_columns` - The output columns
    /// @param __builder The verification builder
    /// @param __columnEvals The input columns
    /// @param __inputChiEval The column of ones with same length as the input columns
    /// @param __outputChiEval The column of ones with same length as the filtered columns
    /// @param __selectionEval The column of ones and zeroes which indicate which rows to include in the result
    /// @return __builderOut The updated verification builder
    /// @return __filteredColumns The output column
    function __filterBaseEvaluateFromColumnEvals( // solhint-disable-line gas-calldata-parameters
        VerificationBuilder.Builder memory __builder,
        uint256[] memory __columnEvals,
        uint256 __inputChiEval,
        uint256 __outputChiEval,
        uint256 __selectionEval
    ) external pure returns (VerificationBuilder.Builder memory __builderOut, uint256[] memory __filteredColumns) {
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
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_challenge(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL FoldLogExpr.pre.sol
            function fold_log_star_evaluate_from_fold(builder_ptr, fold, chi_eval) -> star {
                revert(0, 0)
            }
            // IMPORT-YUL FoldLogExpr.pre.sol
            function fold_log_star_evaluate(builder_ptr, alpha, beta, column_evals, chi_eval) -> star {
                revert(0, 0)
            }
            // IMPORT-YUL FoldUtil.pre.sol
            function fold_first_round_mles(builder_ptr, beta, column_count) -> fold, evaluations_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_first_round_mle(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/MathUtil.pre.sol
            function compute_fold(beta, evals) -> fold {
                revert(0, 0)
            }
            // IMPORT-YUL FoldLogExpr.pre.sol
            function fold_log_evaluate_from_mles(builder_ptr, alpha, beta, column_count, chi_eval) ->
                fold,
                star,
                evaluations_ptr
            {
                revert(0, 0)
            }
            function verify_filter_from_column_evals(
                builder_ptr, column_evals, input_chi_eval, output_chi_eval, selection_eval
            ) -> filtered_columns {
                let alpha := builder_consume_challenge(builder_ptr)
                let beta := builder_consume_challenge(builder_ptr)
                let c_star := fold_log_star_evaluate(builder_ptr, alpha, beta, column_evals, input_chi_eval)
                let d_fold, d_star
                d_fold, d_star, filtered_columns :=
                    fold_log_evaluate_from_mles(builder_ptr, alpha, beta, mload(column_evals), output_chi_eval)

                builder_produce_zerosum_constraint(
                    builder_ptr, submod_bn254(mulmod_bn254(c_star, selection_eval), d_star), 2
                )
                builder_produce_identity_constraint(
                    builder_ptr, mulmod_bn254(d_fold, submod_bn254(output_chi_eval, 1)), 2
                )
            }

            __filteredColumns :=
                verify_filter_from_column_evals(__builder, __columnEvals, __inputChiEval, __outputChiEval, __selectionEval)
        }
        __builderOut = __builder;
    }

    /// @notice Verifies the filtered columns
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// function verify_filter_from_expr_evals(plan_ptr, builder_ptr, num_columns, input_chi_eval, output_chi_eval, selection_eval) -> plan_ptr_out, filtered_columns
    /// ```
    /// ##### Parameters
    /// * `plan_ptr` - calldata pointer to the plan
    /// * `builder_ptr` - memory pointer to the verification builder
    /// * `num_columns` - the number of columns
    /// * `input_chi_eval` - the column of ones with same length as the input columns
    /// * `output_chi_eval` - column of ones with same length as the filtered columns
    /// * `selection_eval` - column of ones and zeroes which indicate which rows to include in the result
    /// ##### Return Values
    /// * `plan_ptr_out` - pointer to the remaining plan after consuming
    /// * `filtered_columns` - consumed star value
    /// @param __plan The plan data
    /// @param __builder The verification builder
    /// @param __numColumns The number of columns
    /// @param __inputChiEval The column of ones with same length as the input columns
    /// @param __outputChiEval The column of ones with same length as the filtered columns
    /// @param __selectionEval The column of ones and zeroes which indicate which rows to include in the result
    /// @return __builderOut The updated verification builder
    /// @return __filteredColumns The output column
    function __filterBaseEvaluateFromExprEvals( // solhint-disable-line gas-calldata-parameters
        bytes calldata __plan,
        VerificationBuilder.Builder memory __builder,
        uint256 __numColumns,
        uint256 __inputChiEval,
        uint256 __outputChiEval,
        uint256 __selectionEval
    ) external pure returns (VerificationBuilder.Builder memory __builderOut, uint256[] memory __filteredColumns) {
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
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_challenge(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL FoldLogExpr.pre.sol
            function fold_log_star_evaluate_from_fold(builder_ptr, fold, chi_eval) -> star {
                revert(0, 0)
            }
            // IMPORT-YUL FoldLogExpr.pre.sol
            function fold_log_star_evaluate_from_expr_evals(
                plan_ptr, builder_ptr, input_chi_eval, alpha, beta, column_count
            ) -> plan_ptr_out, star {
                revert(0, 0)
            }
            // IMPORT-YUL FoldUtil.pre.sol
            function fold_first_round_mles(builder_ptr, beta, column_count) -> fold, evaluations_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_first_round_mle(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/MathUtil.pre.sol
            function compute_fold(beta, evals) -> fold {
                revert(0, 0)
            }
            // IMPORT-YUL FoldLogExpr.pre.sol
            function fold_log_evaluate_from_mles(builder_ptr, alpha, beta, column_count, chi_eval) ->
                fold,
                star,
                evaluations_ptr
            {
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
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_placeholder_parameter(builder_ptr, index) -> value {
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
            // IMPORT-YUL ../proof_exprs/PlaceholderExpr.pre.sol
            function placeholder_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/ScalingCastExpr.pre.sol
            function scaling_cast_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                revert(0, 0)
            }
            // slither-disable-start cyclomatic-complexity
            // IMPORT-YUL ../proof_exprs/ProofExpr.pre.sol
            function proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                revert(0, 0)
            }
            // slither-disable-end cyclomatic-complexity
            // IMPORT-YUL FoldUtil.pre.sol
            function fold_expr_evals(plan_ptr, builder_ptr, input_chi_eval, beta, column_count) -> plan_ptr_out, fold {
                revert(0, 0)
            }
            function verify_filter_from_expr_evals(
                plan_ptr, builder_ptr, num_columns, input_chi_eval, output_chi_eval, selection_eval
            ) -> plan_ptr_out, filtered_columns {
                let c_star, d_fold, d_star
                {
                    let alpha := builder_consume_challenge(builder_ptr)
                    let beta := builder_consume_challenge(builder_ptr)

                    plan_ptr, c_star :=
                        fold_log_star_evaluate_from_expr_evals(
                            plan_ptr, builder_ptr, input_chi_eval, alpha, beta, num_columns
                        )
                    d_fold, d_star, filtered_columns :=
                        fold_log_evaluate_from_mles(builder_ptr, alpha, beta, num_columns, output_chi_eval)
                }

                builder_produce_zerosum_constraint(
                    builder_ptr, submod_bn254(mulmod_bn254(c_star, selection_eval), d_star), 2
                )
                builder_produce_identity_constraint(
                    builder_ptr, mulmod_bn254(d_fold, submod_bn254(output_chi_eval, 1)), 2
                )
                plan_ptr_out := plan_ptr
            }
            // We don't keep the plan out, because it isn't needed for testing.
            let __planOutOffset
            __planOutOffset, __filteredColumns :=
                verify_filter_from_expr_evals(
                    __plan.offset, __builder, __numColumns, __inputChiEval, __outputChiEval, __selectionEval
                )
        }
        __builderOut = __builder;
    }
}
