// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";
import {VerificationBuilder} from "../builder/VerificationBuilder.pre.sol";

/// @title EmptyExec
/// @dev Library for handling sort merge join execution plans
library SortMergeJoinExec {
    /// @notice Evaluates a sort merge join plan
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// sort_merge_join_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr, output_length, output_chi_eval
    /// ```
    /// ##### Parameters
    /// * `plan_ptr` - calldata pointer to the sort merge join execution plan data
    /// * `builder_ptr` - memory pointer to the verification builder
    /// ##### Return Values
    /// * `plan_ptr_out` - pointer to the remaining plan after consuming the sort merge join execution plan data
    /// * `evaluations_ptr` - pointer to the evaluations
    /// * `output_length` - the length of the column of ones
    /// * `output_chi_eval` - pointer to the evaluation of a column of 1s with same length as output
    /// @notice Evaluates a sort merge join execution plan
    /// @dev Evaluates a sort merge join execution plan
    /// @param __plan The sort merge join execution plan data
    /// @param __builder The verification builder
    /// @return __planOut The remaining plan after processing
    /// @return __builderOut The verification builder result
    /// @return __evaluationsPtr The evaluations pointer
    /// @return __outputLength The length of the output chi evaluation
    /// @return __outputChiEvaluation The output chi evaluation
    function __sortMergeJoinEvaluate( // solhint-disable-line gas-calldata-parameters
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
            function compute_fold(beta, evals) -> fold {
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
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_challenge(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_bit_distribution(builder_ptr) -> vary_mask, leading_bit_mask {
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
            function builder_get_singleton_chi_evaluation(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_rho_evaluation(builder_ptr) -> value {
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
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_placeholder_parameter(builder_ptr, index) -> value {
                revert(0, 0)
            }
            // slither-disable-start cyclomatic-complexity
            // IMPORT-YUL ../proof_exprs/ProofExpr.pre.sol
            function proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Array.pre.sol
            function get_uint512_array_element(arr_ptr, index) -> upper, lower {
                revert(0, 0)
            }
            // slither-disable-end cyclomatic-complexity
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_table_chi_evaluation(builder_ptr, table_num) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_chi_evaluation_with_length(builder_ptr) -> length, chi_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_table_chi_evaluation_with_length(builder_ptr, table_num) -> length, chi_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/FoldUtil.pre.sol
            function fold_expr_evals(plan_ptr, builder_ptr, input_chi_eval, beta, column_count) -> plan_ptr_out, fold {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/FoldUtil.pre.sol
            function fold_column_expr_evals(plan_ptr, builder_ptr, beta, column_count) -> plan_ptr_out, fold {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/FoldUtil.pre.sol
            function fold_first_round_mles(builder_ptr, beta, column_count) -> fold, evaluations_ptr {
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
            // IMPORT-YUL FilterExec.pre.sol
            function verify_filter(builder_ptr, c_fold, d_fold, input_chi_eval, output_chi_eval, selection_eval) {
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
            // IMPORT-YUL EmptyExec.pre.sol
            function empty_exec_evaluate(builder_ptr) -> evaluations_ptr, output_length, output_chi_eval {
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
            // IMPORT-YUL ProjectionExec.pre.sol
            function projection_exec_evaluate(plan_ptr, builder_ptr) ->
                plan_ptr_out,
                evaluations_ptr,
                output_length,
                output_chi_eval
            {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/Monotonic.pre.sol
            function monotonic_verify(builder_ptr, alpha, beta, column_eval, chi_eval, strict, asc) {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/MembershipCheck.pre.sol
            function membership_check_evaluate(
                builder_ptr, alpha, beta, chi_n_eval, chi_m_eval, column_evals, candidate_evals
            ) -> multiplicity_eval {
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
            function compute_g_out_star_eval(builder_ptr, alpha, beta, g_in_fold, output_chi_eval, evaluations_ptr) ->
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
            // IMPORT-YUL TableExec.pre.sol
            function table_exec_evaluate(plan_ptr, builder_ptr) ->
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
            function evaluate_input_plans(plan_ptr, builder_ptr) -> plan_ptr_out, hat_evals, join_evals, chi_eval {
                // Evaluate input plan
                let evaluations, length
                plan_ptr, evaluations, length, chi_eval := proof_plan_evaluate(plan_ptr, builder_ptr)

                // Determine total number of evaluations
                let num_columns := mload(evaluations)
                evaluations := add(evaluations, WORD_SIZE)

                // Determine number of columns to join on
                let num_join_columns := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                plan_ptr := add(plan_ptr, UINT64_SIZE)

                // We need a collection to record which indices are not join indices.
                // The total number of evaluations from the input should be the number of entries in this collection.
                let other_indices := mload(FREE_PTR)
                hat_evals := add(other_indices, mul(num_columns, WORD_SIZE))

                // There should be an extra entry in the hat collection for rho
                mstore(hat_evals, add(num_columns, 1))
                let target_hat_evals := add(hat_evals, WORD_SIZE)
                join_evals := add(target_hat_evals, mul(add(num_columns, 1), WORD_SIZE))

                // The join evals should have the same number as the length of the join index collection
                mstore(join_evals, num_join_columns)
                let target_join_evals := add(join_evals, WORD_SIZE)
                mstore(FREE_PTR, add(target_join_evals, mul(num_join_columns, WORD_SIZE)))

                // Populate the non join index collection with 1s.
                for { let i := num_columns } i {} {
                    i := sub(i, 1)
                    mstore(add(other_indices, mul(i, WORD_SIZE)), 1)
                }

                // We need to update each of our collections for each join column
                for { let i := num_join_columns } i { i := sub(i, 1) } {
                    // Get the index of the join column from the plan
                    let join_column_index := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                    plan_ptr := add(plan_ptr, UINT64_SIZE)

                    // We set to 0 any indices that are covered by the join columns
                    mstore(add(other_indices, mul(join_column_index, WORD_SIZE)), 0)

                    // Get the evaluation of the column from the input evaluations
                    let join_column_eval := mload(add(evaluations, mul(join_column_index, WORD_SIZE)))

                    // Both hat and join evaluations should store the join column evaluation
                    mstore(target_hat_evals, join_column_eval)
                    target_hat_evals := add(target_hat_evals, WORD_SIZE)
                    mstore(target_join_evals, join_column_eval)
                    target_join_evals := add(target_join_evals, WORD_SIZE)
                }

                // We need to iterate through each index to determine which were not covered by join columns
                for { let i := 0 } lt(i, num_columns) { i := add(i, 1) } {
                    // We only need to worry about the indices that were not join indices
                    if mload(add(other_indices, mul(i, WORD_SIZE))) {
                        // We store each column evaluation in the hat collection
                        mstore(target_hat_evals, mload(add(evaluations, mul(i, WORD_SIZE))))
                        target_hat_evals := add(target_hat_evals, WORD_SIZE)
                    }
                }
                // The last entry in the hat collection is the rho evaluation
                mstore(target_hat_evals, builder_consume_rho_evaluation(builder_ptr))
                plan_ptr_out := plan_ptr
            }
            function evaluate_u_column_with_monotony_check(builder_ptr, alpha, beta) -> u_column_eval_array, u_chi_eval
            {
                // The length of the u eval collection is 1
                u_column_eval_array := mload(FREE_PTR)
                mstore(FREE_PTR, add(u_column_eval_array, WORDX2_SIZE))
                mstore(u_column_eval_array, 1)

                // We run our monotony check on u eval, before wrapping it in the collection
                u_chi_eval := builder_consume_chi_evaluation(builder_ptr)
                let u_column_eval := builder_consume_final_round_mle(builder_ptr)
                monotonic_verify(builder_ptr, alpha, beta, u_column_eval, u_chi_eval, 1, 1)

                // Store the u eval in the collection
                mstore(add(u_column_eval_array, WORD_SIZE), u_column_eval)
            }
            function consume_and_membership_check_left_column_evals(
                builder_ptr, alpha, beta, hat_evals, res_chi_eval, chi_eval
            ) -> res_column_evals, rho_eval {
                // Initially we set the length of res_column_evals to include the rho eval
                // This will allow us to check the left evals against the hat evals without needing to load a second,
                // almost identical collection
                let num_columns := mload(hat_evals)
                res_column_evals := mload(FREE_PTR)
                mstore(res_column_evals, num_columns)
                mstore(FREE_PTR, add(res_column_evals, mul(add(num_columns, 1), WORD_SIZE)))

                // We decrement num_columns for convenience.
                num_columns := sub(num_columns, 1)

                let target_ptr := add(res_column_evals, WORD_SIZE)
                for { let i := num_columns } i { i := sub(i, 1) } {
                    let eval := builder_consume_final_round_mle(builder_ptr)
                    mstore(target_ptr, eval)
                    target_ptr := add(target_ptr, WORD_SIZE)
                }
                rho_eval := builder_consume_final_round_mle(builder_ptr)
                mstore(target_ptr, rho_eval)
                pop(
                    membership_check_evaluate(
                        builder_ptr, alpha, beta, chi_eval, res_chi_eval, hat_evals, res_column_evals
                    )
                )
                // We now shorten the length of the collection. Rho is still in the same place in memory,
                // but the length effectively indicates that it isn't actually a part of the collection anymore
                mstore(res_column_evals, num_columns)
            }
            function populate_right_evals(eval, right_column_evals, res_column_evals_out) {
                // We store the output in two collections. One output collection and one to compare with the hat evals
                mstore(right_column_evals, eval)
                mstore(res_column_evals_out, eval)
            }
            function evaluate_and_membership_check_right_column_evals(
                builder_ptr, alpha, beta, num_join_columns, hat_evals, res_chi_eval, chi_eval, res_column_evals
            ) -> res_column_evals_out, rho_eval {
                // Load length of incoming res_column_evals
                let res_column_length := mload(res_column_evals)

                // The length of hat_evals is the length of the collection which will be used in the membership check
                let num_columns := mload(hat_evals)
                let right_column_evals := mload(FREE_PTR)
                mstore(right_column_evals, num_columns)
                res_column_evals_out := add(right_column_evals, mul(add(num_columns, 1), WORD_SIZE))

                // The length of the outgoing collection should be
                // the incoming length plus the number of right hat evals less the common columns
                mstore(FREE_PTR, add(res_column_evals_out, add(res_column_length, sub(num_columns, num_join_columns))))
                num_columns := sub(num_columns, 1)
                mstore(res_column_evals_out, add(res_column_length, sub(num_columns, num_join_columns)))

                // The first num_join_columns entries in the res_column_evals collection are the common evals.
                // Both res_column_evals_out and right_column_evals need them.
                // We can use rho_eval as an indexing value until it's time to consume. This saves a local variable.
                for { rho_eval := num_join_columns } rho_eval { rho_eval := sub(rho_eval, 1) } {
                    // We increment before reading and writing because we did not increment after handling the lenghts.
                    res_column_evals := add(res_column_evals, WORD_SIZE)
                    res_column_evals_out := add(res_column_evals_out, WORD_SIZE)
                    right_column_evals := add(right_column_evals, WORD_SIZE)
                    populate_right_evals(mload(res_column_evals), right_column_evals, res_column_evals_out)
                }
                // We copy over the remaining res_column_evals to res_column_evals_out
                for { rho_eval := sub(res_column_length, num_join_columns) } rho_eval { rho_eval := sub(rho_eval, 1) } {
                    res_column_evals := add(res_column_evals, WORD_SIZE)
                    res_column_evals_out := add(res_column_evals_out, WORD_SIZE)
                    mstore(res_column_evals_out, mload(res_column_evals))
                }
                // We consume the non rho, non common right evals and populate right_column_evals and res_column_evals_out
                for { rho_eval := sub(num_columns, num_join_columns) } rho_eval { rho_eval := sub(rho_eval, 1) } {
                    right_column_evals := add(right_column_evals, WORD_SIZE)
                    res_column_evals_out := add(res_column_evals_out, WORD_SIZE)
                    populate_right_evals(
                        builder_consume_final_round_mle(builder_ptr), right_column_evals, res_column_evals_out
                    )
                }
                // We populate the final entry in right_column_evals with rho_eval
                right_column_evals := add(right_column_evals, WORD_SIZE)
                rho_eval := builder_consume_final_round_mle(builder_ptr)
                mstore(right_column_evals, rho_eval)

                // We drop the pointers back to their starting places
                right_column_evals := sub(right_column_evals, mul(add(num_columns, 1), WORD_SIZE))
                res_column_evals_out :=
                    sub(res_column_evals_out, mul(add(res_column_length, sub(num_columns, num_join_columns)), WORD_SIZE))

                // Finally, we can do our membership check
                pop(
                    membership_check_evaluate(
                        builder_ptr, alpha, beta, chi_eval, res_chi_eval, hat_evals, right_column_evals
                    )
                )
            }
            function evaluate_consume_and_check_left_column_evals(plan_ptr, builder_ptr, alpha, beta, res_chi_eval) ->
                plan_ptr_out,
                join_evals,
                chi_eval,
                res_column_evals,
                i_eval
            {
                let hat_evals
                plan_ptr_out, hat_evals, join_evals, chi_eval := evaluate_input_plans(plan_ptr, builder_ptr)
                res_column_evals, i_eval :=
                    consume_and_membership_check_left_column_evals(
                        builder_ptr, alpha, beta, hat_evals, res_chi_eval, chi_eval
                    )
            }
            function evaluate_and_check_right_join_evals(
                plan_ptr, builder_ptr, alpha, beta, res_chi_eval, res_column_evals
            ) -> plan_ptr_out, join_evals, chi_eval, res_column_evals_out, i_eval {
                {
                    let hat_evals
                    plan_ptr, hat_evals, join_evals, chi_eval := evaluate_input_plans(plan_ptr, builder_ptr)
                    res_column_evals, i_eval :=
                        evaluate_and_membership_check_right_column_evals(
                            builder_ptr, alpha, beta, mload(join_evals), hat_evals, res_chi_eval, chi_eval, res_column_evals
                        )
                }
                res_column_evals_out := res_column_evals
                plan_ptr_out := plan_ptr
            }
            function evaluate_and_check_left_side(
                plan_ptr, builder_ptr, alpha, beta, u_column_eval_array, u_chi_eval, res_chi_eval
            ) -> plan_ptr_out, res_column_evals, i_eval, w_eval {
                let join_evals, chi_eval
                plan_ptr, join_evals, chi_eval, res_column_evals, i_eval :=
                    evaluate_consume_and_check_left_column_evals(plan_ptr, builder_ptr, alpha, beta, res_chi_eval)
                w_eval :=
                    membership_check_evaluate(
                        builder_ptr, alpha, beta, u_chi_eval, chi_eval, u_column_eval_array, join_evals
                    )
                plan_ptr_out := plan_ptr
            }

            function evaluate_and_check_right_side(
                plan_ptr, builder_ptr, alpha, beta, u_column_eval_array, u_chi_eval, res_chi_eval, res_column_evals
            ) -> plan_ptr_out, res_column_evals_out, i_eval, w_eval {
                let join_evals
                // res_column_evals_out is being used to hold chi_eval, for lack of variable space
                plan_ptr, join_evals, res_column_evals_out, res_column_evals, i_eval :=
                    evaluate_and_check_right_join_evals(plan_ptr, builder_ptr, alpha, beta, res_chi_eval, res_column_evals)
                w_eval :=
                    membership_check_evaluate(
                        builder_ptr, alpha, beta, u_chi_eval, res_column_evals_out, u_column_eval_array, join_evals
                    )
                res_column_evals_out := res_column_evals
                plan_ptr_out := plan_ptr
            }

            function evaluate_with_all_checks(plan_ptr, builder_ptr, alpha, beta, output_chi_eval) ->
                plan_ptr_out,
                res_column_evals
            {
                let i_eval, w_eval
                {
                    // We need u eval and u chi in order to run the checks on the left and right sides
                    let u_column_eval_array, u_chi_eval :=
                        evaluate_u_column_with_monotony_check(builder_ptr, alpha, beta)

                    // We run all our checks on the left side.
                    plan_ptr, res_column_evals, i_eval, w_eval :=
                        evaluate_and_check_left_side(
                            plan_ptr, builder_ptr, alpha, beta, u_column_eval_array, u_chi_eval, output_chi_eval
                        )

                    // We run all our checks on the right side.
                    // Because we are running out of local variables, we use u_chi_eval instead of new variable w_right_eval
                    // We can use u_chi_eval now because it is no longer needed at this point
                    let i_right_eval
                    plan_ptr, res_column_evals, i_right_eval, u_chi_eval :=
                        evaluate_and_check_right_side(
                            plan_ptr,
                            builder_ptr,
                            alpha,
                            beta,
                            u_column_eval_array,
                            u_chi_eval,
                            output_chi_eval,
                            res_column_evals
                        )
                    i_eval := addmod_bn254(mulmod_bn254(i_eval, shl(64, 1)), i_right_eval)
                    w_eval := mulmod_bn254(w_eval, u_chi_eval)
                }
                monotonic_verify(builder_ptr, alpha, beta, i_eval, output_chi_eval, 1, 1)
                builder_produce_zerosum_constraint(builder_ptr, submod_bn254(w_eval, output_chi_eval), 2)
                plan_ptr_out := plan_ptr
            }

            function sort_merge_join_evaluate(plan_ptr, builder_ptr) ->
                plan_ptr_out,
                evaluations_ptr,
                output_length,
                output_chi_eval
            {
                let alpha := builder_consume_challenge(builder_ptr)
                let beta := builder_consume_challenge(builder_ptr)
                output_length, output_chi_eval := builder_consume_chi_evaluation_with_length(builder_ptr)
                // We don't use output length for any of our checks
                plan_ptr_out, evaluations_ptr :=
                    evaluate_with_all_checks(plan_ptr, builder_ptr, alpha, beta, output_chi_eval)
            }

            let __planOutOffset
            __planOutOffset, __evaluations, __outputLength, __outputChiEvaluation :=
                sort_merge_join_evaluate(__plan.offset, __builder)
            __planOut.offset := __planOutOffset
            // slither-disable-next-line write-after-write
            __planOut.length := sub(__plan.length, sub(__planOutOffset, __plan.offset))
        }
        __evaluationsPtr = __evaluations;
        __builderOut = __builder;
    }
}
