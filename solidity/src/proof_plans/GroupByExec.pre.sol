// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";
import {VerificationBuilder} from "../builder/VerificationBuilder.pre.sol";

/// @title GroupByExec
/// @dev Library for handling group by execution plans
library GroupByExec {
    /// @notice Evaluates a group by execution plan
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// group_by_exec_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr
    /// ```
    /// ##### Parameters
    /// * `plan_ptr` - calldata pointer to the group by execution plan
    /// * `builder_ptr` - memory pointer to the verification builder
    /// ##### Return Values
    /// * `plan_ptr_out` - pointer to the remaining plan after consuming the group by execution plan
    /// * `evaluations_ptr` - pointer to the evaluations
    /// * `output_length` - the length of the column of ones
    /// * `output_chi_eval` - pointer to the evaluation of a column of 1s with same length as output
    /// @notice Evaluates expressions for group by operation and verifies the aggregation constraints
    /// @notice ##### Constraints
    /// * Inputs: \\(G_1,\ldots,G_\ell=\texttt{g_in}\\), \\(A_1,\ldots,A_m=\texttt{sum_in}\\),
    ///   and \\(S=\texttt{sel_in}\\) with input characteristic \\(\chi_{[0,n)}=\texttt{input_chi_eval}\\).
    /// * Outputs: \\(G_1',\ldots,G_\ell'=\texttt{g_out}\\), \\(A_1',\ldots,A_m'=\texttt{sum_out}\\),
    ///   and \\(C=\texttt{count_out}\\) with output characteristic \\(\chi_{[0,m')}=\texttt{output_chi_eval}\\).
    /// * Challenges: \\(\alpha=\texttt{alpha}\\), \\(\beta=\texttt{beta}\\)
    /// * Helpers:
    ///   \\(g_{in,fold} \=\texttt{g_in_fold} :\equiv \alpha \sum_{j=1}^{\ell} G_j \beta^{\ell-j}\\)
    ///   \\(g_{out,fold} \=\texttt{g_out_fold} :\equiv \alpha \sum_{j=1}^{\ell} G_j' \beta^{\ell-j}\\)
    ///   \\(sum_{in,fold} \=\texttt{sum_in_fold} :\equiv \chi_{[0,n)} + \beta \sum_{j=1}^{m} A_j \beta^{m-j}\\)
    ///   \\(sum_{out,fold} \=\texttt{sum_out_fold} :\equiv C + \beta \sum_{j=1}^{m} A_j' \beta^{m-j}\\)
    ///   \\(G^\star = \texttt{g_in_star}\\) and \\(G^{\star'} = \texttt{g_out_star}\\)
    /// * Constraints:
    /// \\[\begin{aligned}
    /// G^\star \cdot S \cdot sum_{in,fold} - G^{\star'} \cdot sum_{out,fold} &\overset{\sum}{=} 0\\\\
    /// G^\star + G^\star \cdot g_{in,fold} - \chi_{[0,n)} &\equiv 0\\\\
    /// G^{\star'} + G^{\star'} \cdot g_{out,fold} - \chi_{[0,m')} &\equiv 0\\\\
    /// \end{aligned}\\]
    /// Note: the notation \\(A\overset{\sum}{=}B\\) is used to indicate the zero-sum constraint.
    /// @dev Evaluates a group by execution plan
    /// @param __plan The group by execution plan data
    /// @param __builder The verification builder
    /// @return __planOut The remaining plan after processing
    /// @return __builderOut The verification builder result
    /// @return __evaluationsPtr The evaluations pointer
    /// @return __outputLength The length of the output chi evaluation
    /// @return __outputChiEvaluation The output chi evaluation
    function __groupByExecEvaluate( // solhint-disable-line gas-calldata-parameters
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
            function builder_produce_zerosum_constraint(builder_ptr, evaluation, degree) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_produce_identity_constraint(builder_ptr, evaluation, degree) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_bit_distribution(builder_ptr) -> vary_mask, leading_bit_mask {
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
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_singleton_chi_evaluation(builder_ptr) -> value {
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
            // IMPORT-YUL ../proof_exprs/InequalityExpr.pre.sol
            function inequality_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
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
            // IMPORT-YUL ../proof_exprs/PlaceholderExpr.pre.sol
            function placeholder_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/ScalingCastExpr.pre.sol
            function scaling_cast_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
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
            // IMPORT-YUL ../proof_gadgets/FoldUtil.pre.sol
            function fold_expr_evals(plan_ptr, builder_ptr, input_chi_eval, beta, column_count) -> plan_ptr_out, fold {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/FoldUtil.pre.sol
            function fold_column_expr_evals(plan_ptr, builder_ptr, beta, column_count) -> plan_ptr_out, fold {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/FoldLogExpr.pre.sol
            function fold_log_star_evaluate_from_fold(builder_ptr, fold, chi_eval) -> star {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/FoldLogExpr.pre.sol
            function fold_log_star_evaluate_from_column_exprs(
                plan_ptr, builder_ptr, alpha, beta, column_count, chi_eval
            ) -> plan_ptr_out, star {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/FoldUtil.pre.sol
            function fold_first_round_mles(builder_ptr, column_count, beta) -> fold, evaluations_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL FilterExec.pre.sol
            function compute_filter_folds(plan_ptr, builder_ptr, input_chi_eval) ->
                plan_ptr_out,
                c_fold,
                d_fold,
                evaluations_ptr
            {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_plans/FilterExec.pre.sol
            function verify_filter(builder_ptr, c_fold, d_fold, input_chi_eval, output_chi_eval, selection_eval) {
                revert(0, 0)
            }
            // IMPORT-YUL FilterExec.pre.sol
            function filter_exec_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
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
            // IMPORT-YUL ../proof_gadgets/Monotonic.pre.sol
            function monotonic_verify(builder_ptr, alpha, beta, column_eval, chi_eval, strict, asc) {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/SignExpr.pre.sol
            function sign_expr_evaluate(expr_eval, builder_ptr, chi_eval) -> result_eval {
                revert(0, 0)
            }

            function compute_g_in_star_eval(plan_ptr, builder_ptr, alpha, beta, input_chi_eval) ->
                plan_ptr_out,
                g_in_star_eval_times_selection_eval
            {
                let num_group_by_columns := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                // We can not prove uniqueness for multiple columns yet
                if sub(num_group_by_columns, 1) { err(ERR_UNPROVABLE_GROUP_BY) }
                plan_ptr := add(plan_ptr, UINT64_SIZE)

                // Process group by columns
                let g_in_star_eval
                plan_ptr, g_in_star_eval :=
                    fold_log_star_evaluate_from_column_exprs(
                        plan_ptr, builder_ptr, alpha, beta, num_group_by_columns, input_chi_eval
                    )

                let selection_eval
                plan_ptr, selection_eval := proof_expr_evaluate(plan_ptr, builder_ptr, input_chi_eval)
                g_in_star_eval_times_selection_eval := mulmod_bn254(g_in_star_eval, selection_eval)
                plan_ptr_out := plan_ptr
            }

            function compute_sum_in_fold_eval(plan_ptr, builder_ptr, alpha, beta, input_chi_eval) ->
                plan_ptr_out,
                sum_in_fold_eval,
                num_sum_columns
            {
                num_sum_columns := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                plan_ptr := add(plan_ptr, UINT64_SIZE)
                plan_ptr, sum_in_fold_eval :=
                    fold_expr_evals(plan_ptr, builder_ptr, input_chi_eval, beta, num_sum_columns)
                sum_in_fold_eval := addmod_bn254(mulmod_bn254(sum_in_fold_eval, beta), input_chi_eval)
                plan_ptr_out := plan_ptr
            }
            function compute_g_out_star_eval(builder_ptr, alpha, beta, output_chi_eval, evaluations_ptr) ->
                g_out_star_eval
            {
                let mle := builder_consume_first_round_mle(builder_ptr)
                mstore(evaluations_ptr, mle)
                evaluations_ptr := add(evaluations_ptr, WORD_SIZE)
                let g_out_fold := mulmod_bn254(mle, alpha)
                g_out_star_eval := builder_consume_final_round_mle(builder_ptr)
                // Second constraint: g_out_star + g_out_star * g_out_fold - output_chi_eval = 0
                builder_produce_identity_constraint(
                    builder_ptr,
                    submod_bn254(
                        addmod_bn254(g_out_star_eval, mulmod_bn254(g_out_star_eval, g_out_fold)), output_chi_eval
                    ),
                    2
                )
                // Uniqueness constraint, currently only for single group by column using monotonicity
                monotonic_verify(builder_ptr, alpha, beta, mle, output_chi_eval, 1, 1)
            }
            function compute_sum_out_fold_eval(
                builder_ptr, alpha, beta, output_chi_eval, num_sum_columns, evaluations_ptr
            ) -> sum_out_fold_eval {
                sum_out_fold_eval := 0
                for {} num_sum_columns { num_sum_columns := sub(num_sum_columns, 1) } {
                    let mle := builder_consume_first_round_mle(builder_ptr)
                    sum_out_fold_eval := addmod_bn254(mulmod_bn254(sum_out_fold_eval, beta), mle)
                    mstore(evaluations_ptr, mle)
                    evaluations_ptr := add(evaluations_ptr, WORD_SIZE)
                }
                // Consume count column evaluation
                let count_out_eval := builder_consume_first_round_mle(builder_ptr)
                mstore(evaluations_ptr, count_out_eval)
                sum_out_fold_eval := addmod_bn254(mulmod_bn254(sum_out_fold_eval, beta), count_out_eval)
            }

            function read_input_evals(plan_ptr, builder_ptr, alpha, beta) ->
                plan_ptr_out,
                partial_dlog_zero_sum_constraint_eval,
                num_sum_columns
            {
                // Read input chi evaluation
                let input_chi_eval
                {
                    let table_num := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                    input_chi_eval := builder_get_table_chi_evaluation(builder_ptr, table_num)
                    plan_ptr := add(plan_ptr, UINT64_SIZE)
                }

                // Read/eval group by inputs, selection inputs, and fold and dlog them
                let g_in_star_eval_times_selection_eval
                plan_ptr, g_in_star_eval_times_selection_eval :=
                    compute_g_in_star_eval(plan_ptr, builder_ptr, alpha, beta, input_chi_eval)

                // Read/eval sum inputs and fold them
                let sum_in_fold_eval
                plan_ptr, sum_in_fold_eval, num_sum_columns :=
                    compute_sum_in_fold_eval(plan_ptr, builder_ptr, alpha, beta, input_chi_eval)

                partial_dlog_zero_sum_constraint_eval :=
                    mulmod_bn254(g_in_star_eval_times_selection_eval, sum_in_fold_eval)

                // Read count alias
                {
                    let count_alias
                    plan_ptr, count_alias := read_binary(plan_ptr)
                }

                plan_ptr_out := plan_ptr
            }

            function read_output_evals(
                builder_ptr, alpha, beta, partial_dlog_zero_sum_constraint_eval, num_group_by_columns, num_sum_columns
            ) -> evaluations_ptr, output_length, output_chi_eval {
                // Allocate memory for evaluations
                {
                    let free_ptr := mload(FREE_PTR)
                    evaluations_ptr := free_ptr
                    let num_evals := add(num_group_by_columns, add(num_sum_columns, 1))
                    mstore(free_ptr, num_evals)
                    free_ptr := add(free_ptr, WORD_SIZE)
                    free_ptr := add(free_ptr, mul(num_evals, WORD_SIZE))
                    mstore(FREE_PTR, free_ptr)
                }

                output_length, output_chi_eval := builder_consume_chi_evaluation_with_length(builder_ptr)

                let g_out_star_eval :=
                    compute_g_out_star_eval(builder_ptr, alpha, beta, output_chi_eval, add(evaluations_ptr, WORD_SIZE))

                let sum_out_fold_eval :=
                    compute_sum_out_fold_eval(
                        builder_ptr,
                        alpha,
                        beta,
                        output_chi_eval,
                        num_sum_columns,
                        add(evaluations_ptr, mul(add(num_group_by_columns, 1), WORD_SIZE))
                    )

                builder_produce_zerosum_constraint(
                    builder_ptr,
                    submod_bn254(
                        partial_dlog_zero_sum_constraint_eval, mulmod_bn254(g_out_star_eval, sum_out_fold_eval)
                    ),
                    3
                )
            }

            function group_by_exec_evaluate(plan_ptr, builder_ptr) ->
                plan_ptr_out,
                evaluations_ptr,
                output_length,
                output_chi_eval
            {
                let alpha := builder_consume_challenge(builder_ptr)
                let beta := builder_consume_challenge(builder_ptr)

                let partial_dlog_zero_sum_constraint_eval, num_sum_columns
                plan_ptr_out, partial_dlog_zero_sum_constraint_eval, num_sum_columns :=
                    read_input_evals(plan_ptr, builder_ptr, alpha, beta)

                // Read output
                // For now, we can assume the number of group by columns is 1,
                // because the function would have errored by this point otherwise
                evaluations_ptr, output_length, output_chi_eval :=
                    read_output_evals(builder_ptr, alpha, beta, partial_dlog_zero_sum_constraint_eval, 1, num_sum_columns)
            }

            let __planOutOffset
            __planOutOffset, __evaluations, __outputLength, __outputChiEvaluation :=
                group_by_exec_evaluate(__plan.offset, __builder)
            __planOut.offset := __planOutOffset
            // slither-disable-next-line write-after-write
            __planOut.length := sub(__plan.length, sub(__planOutOffset, __plan.offset))
        }
        __evaluationsPtr = __evaluations;
        __builderOut = __builder;
    }
}
