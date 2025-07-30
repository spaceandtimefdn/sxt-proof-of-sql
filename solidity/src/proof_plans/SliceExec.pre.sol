// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";
import {VerificationBuilder} from "../builder/VerificationBuilder.pre.sol";

/// @title SliceExec
/// @dev Library for handling slice execution plans
library SliceExec {
    /// @notice Evaluates a slice execution plan
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// slice_exec_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr, output_length, output_chi_eval
    /// ```
    /// ##### Parameters
    /// * `plan_ptr` - calldata pointer to the slice execution plan
    /// * `builder_ptr` - memory pointer to the verification builder
    /// ##### Return Values
    /// * `plan_ptr_out` - pointer to the remaining plan after consuming the slice execution plan
    /// * `evaluations_ptr` - pointer to the evaluations
    /// * `output_length` - the number of rows of the output
    /// * `output_chi_eval` - pointer to the evaluation of a column of 1s with same length as output
    /// @notice Evaluates a slice execution plan
    /// ##### Proof Plan Encoding
    /// The slice plan is encoded as follows:
    /// 1. The input plan
    /// 2. The number of rows to skip
    /// 3. The number of rows to fetch
    /// @dev Evaluates a slice execution plan
    /// @param __plan The slice execution plan data
    /// @param __builder The verification builder
    /// @return __planOut The remaining plan after processing
    /// @return __builderOut The verification builder result
    /// @return __evaluationsPtr The evaluations pointer
    /// @return __outputLength The length of the output chi evaluation
    /// @return __outputChiEvaluation The output chi evaluation
    function __sliceExecEvaluate( // solhint-disable-line gas-calldata-parameters
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
            // IMPORT-YUL ../base/MathUtil.pre.sol
            function min(a, b) -> minimum {
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
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_challenge(builder_ptr) -> value {
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
            function builder_consume_chi_evaluation(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_chi_evaluation_with_length(builder_ptr) -> length, chi_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_rho_evaluation(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/FoldUtil.pre.sol
            function fold_column_expr_evals(plan_ptr, builder_ptr, beta, column_count) -> plan_ptr_out, fold {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_produce_zerosum_constraint(builder_ptr, evaluation, degree) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_produce_identity_constraint(builder_ptr, evaluation, degree) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/SwitchUtil.pre.sol
            function case_const(lhs, rhs) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Array.pre.sol
            function get_array_element(arr_ptr, index) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_column_evaluation(builder_ptr, column_num) -> value {
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
            // IMPORT-YUL ../proof_exprs/EqualsExpr.pre.sol
            function equals_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/AddExpr.pre.sol
            function add_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/SubtractExpr.pre.sol
            function subtract_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/MultiplyExpr.pre.sol
            function multiply_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/AndExpr.pre.sol
            function and_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/OrExpr.pre.sol
            function or_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/NotExpr.pre.sol
            function not_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/CastExpr.pre.sol
            function cast_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/ScalingCastExpr.pre.sol
            function scaling_cast_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
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
            // IMPORT-YUL ../proof_exprs/PlaceholderExpr.pre.sol
            function placeholder_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                revert(0, 0)
            }
            // slither-disable-start cyclomatic-complexity
            // IMPORT-YUL ../proof_exprs/ProofExpr.pre.sol
            function proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                revert(0, 0)
            }
            // slither-disable-end cyclomatic-complexity
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
            // IMPORT-YUL ../base/Array.pre.sol
            function get_uint512_array_element(arr_ptr, index) -> upper, lower {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_table_chi_evaluation(builder_ptr, table_num) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_table_chi_evaluation_with_length(builder_ptr, table_num) -> length, chi_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_placeholder_parameter(builder_ptr, index) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/Shift.pre.sol
            function compute_shift_identity_constraint(star, chi_plus_one, fold) -> constraint {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/Shift.pre.sol
            function compute_shift_fold(alpha, beta, eval, rho) -> fold {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/Shift.pre.sol
            function shift_evaluate(builder_ptr, alpha, beta, expr_eval, chi_eval) ->
                shifted_expr_eval,
                chi_plus_one_eval
            {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/Monotonic.pre.sol
            function monotonic_verify(builder_ptr, alpha, beta, column_eval, chi_eval, strict, asc) {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/FoldUtil.pre.sol
            function fold_expr_evals(plan_ptr, builder_ptr, input_chi_eval, beta, column_count) -> plan_ptr_out, fold {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/FoldUtil.pre.sol
            function fold_first_round_mles(builder_ptr, beta, column_count) -> fold, evaluations_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ProjectionExec.pre.sol
            function projection_exec_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr, output_chi_eval {
                revert(0, 0)
            }
            // IMPORT-YUL FilterExec.pre.sol
            function compute_filter_folds(plan_ptr, builder_ptr, input_chi_eval, beta) ->
                plan_ptr_out,
                c_fold,
                d_fold,
                evaluations_ptr
            {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/FilterBase.pre.sol
            function verify_filter(builder_ptr, c_fold, d_fold, input_chi_eval, output_chi_eval, selection_eval) {
                revert(0, 0)
            }
            // IMPORT-YUL TableExec.pre.sol
            function table_exec_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr, output_chi_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_singleton_chi_evaluation(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL EmptyExec.pre.sol
            function empty_exec_evaluate(builder_ptr) -> evaluations_ptr, output_chi_eval {
                revert(0, 0)
            }
            // IMPORT-YUL FilterExec.pre.sol
            function filter_exec_evaluate(plan_ptr, builder_ptr) ->
                plan_ptr_out,
                evaluations_ptr,
                output_length,
                output_chi_eval
            {
                revert(0, 0)
            }
            // IMPORT-YUL GroupByExec.pre.sol
            function compute_g_in_star_eval(plan_ptr, builder_ptr, alpha, beta, input_chi_eval) ->
                plan_ptr_out,
                g_in_fold,
                g_in_star_eval_times_selection_eval,
                num_group_by_columns
            {
                revert(0, 0)
            }
            // IMPORT-YUL GroupByExec.pre.sol
            function compute_sum_in_fold_eval(plan_ptr, builder_ptr, alpha, beta, input_chi_eval) ->
                plan_ptr_out,
                sum_in_fold_eval,
                num_sum_columns
            {
                revert(0, 0)
            }
            // IMPORT-YUL GroupByExec.pre.sol
            function compute_g_out_star_eval(builder_ptr, alpha, beta, output_chi_eval, evaluations_ptr) ->
                g_out_star_eval
            {
                revert(0, 0)
            }
            // IMPORT-YUL GroupByExec.pre.sol
            function compute_sum_out_fold_eval(
                builder_ptr, alpha, beta, output_chi_eval, num_sum_columns, evaluations_ptr
            ) -> sum_out_fold_eval {
                revert(0, 0)
            }
            // IMPORT-YUL GroupByExec.pre.sol
            function read_input_evals(plan_ptr, builder_ptr, alpha, beta) ->
                plan_ptr_out,
                partial_dlog_zero_sum_constraint_eval,
                num_group_by_columns,
                num_sum_columns
            {
                revert(0, 0)
            }
            // IMPORT-YUL GroupByExec.pre.sol
            function read_output_evals(
                builder_ptr, alpha, beta, partial_dlog_zero_sum_constraint_eval, num_group_by_columns, num_sum_columns
            ) -> evaluations_ptr, output_chi_eval {
                revert(0, 0)
            }
            // IMPORT-YUL GroupByExec.pre.sol
            function group_by_exec_evaluate(plan_ptr, builder_ptr) ->
                plan_ptr_out,
                evaluations_ptr,
                output_length,
                output_chi_eval
            {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/FoldLogExpr.pre.sol
            function fold_log_star_evaluate_from_fold(builder_ptr, fold, chi_eval) -> star {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/FoldLogExpr.pre.sol
            function fold_log_star_evaluate(builder_ptr, alpha, beta, column_evals, chi_eval) -> star {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/FoldLogExpr.pre.sol
            function fold_log_star_evaluate_from_column_exprs(
                plan_ptr, builder_ptr, alpha, beta, column_count, chi_eval
            ) -> plan_ptr_out, star {
                revert(0, 0)
            }
            // IMPORT-YUL UnionExec.pre.sol
            function union_input_evaluate(plan_ptr, builder_ptr, gamma, beta) ->
                plan_ptr_out,
                output_length,
                num_columns,
                zerosum_constraint
            {
                revert(0, 0)
            }
            // IMPORT-YUL UnionExec.pre.sol
            function union_exec_evaluate(plan_ptr, builder_ptr) ->
                plan_ptr_out,
                evaluations_ptr,
                output_length,
                output_chi_eval
            {
                revert(0, 0)
            }
            // IMPORT-YUL ProofPlan.pre.sol
            function proof_plan_evaluate(plan_ptr, builder_ptr) ->
                plan_ptr_out,
                evaluations_ptr,
                output_length,
                output_chi_eval
            {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/MathUtil.pre.sol
            function compute_fold(beta, evals) -> fold {
                revert(0, 0)
            }
            function get_and_verify_slice_length(plan_ptr, builder_ptr, input_length) ->
                plan_ptr_out,
                output_length,
                output_chi_eval,
                selection_eval
            {
                let expected_skip := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                plan_ptr := add(plan_ptr, UINT64_SIZE)
                let expected_max_length
                {
                    let is_fetch_populated := shr(BOOLEAN_PADDING_BITS, calldataload(plan_ptr))
                    plan_ptr := add(plan_ptr, BOOLEAN_SIZE)
                    switch is_fetch_populated
                    case 0 { expected_max_length := input_length }
                    default {
                        let fetch := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                        plan_ptr := add(plan_ptr, UINT64_SIZE)
                        expected_max_length := min(add(expected_skip, fetch), input_length)
                    }
                }

                output_length, output_chi_eval := builder_consume_chi_evaluation_with_length(builder_ptr)
                let actual_skip
                actual_skip, selection_eval := builder_consume_chi_evaluation_with_length(builder_ptr)
                let max_length, max_eval := builder_consume_chi_evaluation_with_length(builder_ptr)
                selection_eval := submod_bn254(max_eval, selection_eval)

                if sub(max_length, add(actual_skip, output_length)) { err(ERR_SLICE_OFFSET_SELECTION_SIZE_MISMATCH) }
                if sub(min(expected_skip, input_length), actual_skip) { err(ERR_SLICE_OFFSET_PLAN_VALUE_MISMATCH) }
                if sub(max_length, expected_max_length) { err(ERR_SLICE_MAX_LENGTH_MISMATCH) }

                plan_ptr_out := plan_ptr
            }

            function compute_slice_folds(builder_ptr, input_evaluations_ptr) -> c_fold, d_fold, evaluations_ptr {
                let alpha := builder_consume_challenge(builder_ptr)
                let beta := builder_consume_challenge(builder_ptr)
                c_fold := mulmod_bn254(alpha, compute_fold(beta, input_evaluations_ptr))
                d_fold, evaluations_ptr := fold_first_round_mles(builder_ptr, beta, mload(input_evaluations_ptr))
                d_fold := mulmod_bn254(alpha, d_fold)
            }

            function slice_exec_evaluate(plan_ptr, builder_ptr) ->
                plan_ptr_out,
                evaluations_ptr,
                output_length,
                output_chi_eval
            {
                let c_fold, d_fold, input_chi_eval, selection_eval
                {
                    let input_length, input_evaluations_ptr
                    plan_ptr, input_evaluations_ptr, input_length, input_chi_eval :=
                        proof_plan_evaluate(plan_ptr, builder_ptr)
                    plan_ptr_out, output_length, output_chi_eval, selection_eval :=
                        get_and_verify_slice_length(plan_ptr, builder_ptr, input_length)
                    c_fold, d_fold, evaluations_ptr := compute_slice_folds(builder_ptr, input_evaluations_ptr)
                }
                verify_filter(builder_ptr, c_fold, d_fold, input_chi_eval, output_chi_eval, selection_eval)
            }

            let __planOutOffset
            __planOutOffset, __evaluations, __outputLength, __outputChiEvaluation :=
                slice_exec_evaluate(__plan.offset, __builder)
            __planOut.offset := __planOutOffset
            // slither-disable-next-line write-after-write
            __planOut.length := sub(__plan.length, sub(__planOutOffset, __plan.offset))
        }
        __evaluationsPtr = __evaluations;
        __builderOut = __builder;
    }
}
