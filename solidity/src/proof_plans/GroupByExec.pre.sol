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
    function __groupByExecEvaluate( // solhint-disable-line gas-calldata-parameters
    bytes calldata __plan, VerificationBuilder.Builder memory __builder)
        external
        pure
        returns (
            bytes calldata __planOut,
            VerificationBuilder.Builder memory __builderOut,
            uint256[] memory __evaluationsPtr
        )
    {
        uint256[] memory __evaluations;
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Queue.pre.sol
            function dequeue(queue_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_challenge(builder_ptr) -> value {
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
            // IMPORT-YUL ../proof_exprs/ProofExpr.pre.sol
            function proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/DataType.pre.sol
            function read_entry(result_ptr, data_type_variant) -> result_ptr_out, entry {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/DataType.pre.sol
            function read_data_type(ptr) -> ptr_out, data_type {
                revert(0, 0)
            }
            // IMPORT-YUL FilterExec.pre.sol
            function compute_folds(plan_ptr, builder_ptr, input_chi_eval) ->
                plan_ptr_out,
                c_fold,
                d_fold,
                evaluations_ptr
            {
                revert(0, 0)
            }
            // IMPORT-YUL FilterExec.pre.sol
            function filter_exec_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_table_chi_evaluation(builder_ptr, table_num) -> value {
                revert(0, 0)
            }

            // Computes `result = sum (beta^(n-j) * vals[j]) for j in 0..vals.len()`
            function fold_vals(beta, vals_ptr, vals_length) -> result {
                result := 0
                for { let i := 0 } lt(i, vals_length) { i := add(i, 1) } {
                    let val := mload(vals_ptr)
                    vals_ptr := add(vals_ptr, WORD_SIZE)
                    result := addmod(mulmod(result, beta, MODULUS), val, MODULUS)
                }
            }

            function group_by_exec_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr {
                // Challenges
                let alpha := builder_consume_challenge(builder_ptr)
                let beta := builder_consume_challenge(builder_ptr)

                // Table input
                let input_chi_eval :=
                    builder_get_table_chi_evaluation(builder_ptr, shr(UINT64_PADDING_BITS, calldataload(plan_ptr)))
                plan_ptr := add(plan_ptr, UINT64_SIZE)

                // Where clause evaluation
                let selection_eval
                plan_ptr, selection_eval := proof_expr_evaluate(plan_ptr, builder_ptr, input_chi_eval)

                // Group by columns count and evaluations
                let group_by_count := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                plan_ptr := add(plan_ptr, UINT64_SIZE)

                // Allocate memory for group_by input evaluations
                let g_in_evals_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(g_in_evals_ptr, mul(add(group_by_count, 1), WORD_SIZE)))

                // Store the number of evaluations at the beginning
                mstore(g_in_evals_ptr, group_by_count)

                // Process group by expressions
                for { let i := 0 } lt(i, group_by_count) { i := add(i, 1) } {
                    let g_in_eval
                    plan_ptr, g_in_eval := proof_expr_evaluate(plan_ptr, builder_ptr, input_chi_eval)
                    mstore(add(g_in_evals_ptr, mul(add(i, 1), WORD_SIZE)), g_in_eval)
                }

                // Sum expressions count and evaluations
                let sum_count := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                plan_ptr := add(plan_ptr, UINT64_SIZE)

                // Allocate memory for sum input evaluations
                let sum_in_evals_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(sum_in_evals_ptr, mul(add(sum_count, 1), WORD_SIZE)))

                // Store the number of evaluations at the beginning
                mstore(sum_in_evals_ptr, sum_count)

                // Process sum expressions
                for { let i := 0 } lt(i, sum_count) { i := add(i, 1) } {
                    let sum_in_eval
                    plan_ptr, sum_in_eval := proof_expr_evaluate(plan_ptr, builder_ptr, input_chi_eval)
                    mstore(add(sum_in_evals_ptr, mul(add(i, 1), WORD_SIZE)), sum_in_eval)
                }

                // Skip the count alias (we don't need to parse it for verification)
                plan_ptr := add(plan_ptr, UINT64_SIZE)

                // Read result columns
                // Consume group by result columns evaluations
                let g_out_evals_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(g_out_evals_ptr, mul(add(group_by_count, 1), WORD_SIZE)))
                mstore(g_out_evals_ptr, group_by_count)

                for { let i := 0 } lt(i, group_by_count) { i := add(i, 1) } {
                    let g_out_eval := builder_consume_final_round_mle(builder_ptr)
                    mstore(add(g_out_evals_ptr, mul(add(i, 1), WORD_SIZE)), g_out_eval)
                }

                // Consume sum result columns evaluations
                let sum_out_evals_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(sum_out_evals_ptr, mul(add(sum_count, 1), WORD_SIZE)))
                mstore(sum_out_evals_ptr, sum_count)

                for { let i := 0 } lt(i, sum_count) { i := add(i, 1) } {
                    let sum_out_eval := builder_consume_final_round_mle(builder_ptr)
                    mstore(add(sum_out_evals_ptr, mul(add(i, 1), WORD_SIZE)), sum_out_eval)
                }

                // Consume count column evaluation
                let count_out_eval := builder_consume_final_round_mle(builder_ptr)

                // Get chi evaluation
                let output_chi_eval := builder_consume_chi_evaluation(builder_ptr)

                // Verify the group by operation
                // Compute g_in_fold = alpha * fold_vals(beta, g_in_evals)
                let g_in_fold :=
                    mulmod(alpha, fold_vals(beta, add(g_in_evals_ptr, WORD_SIZE), mload(g_in_evals_ptr)), MODULUS)

                // Compute g_out_fold = alpha * fold_vals(beta, g_out_evals)
                let g_out_fold :=
                    mulmod(alpha, fold_vals(beta, add(g_out_evals_ptr, WORD_SIZE), mload(g_out_evals_ptr)), MODULUS)

                // Compute sum_in_fold = input_chi_eval + beta * fold_vals(beta, sum_in_evals)
                let sum_in_fold :=
                    addmod(
                        input_chi_eval,
                        mulmod(beta, fold_vals(beta, add(sum_in_evals_ptr, WORD_SIZE), mload(sum_in_evals_ptr)), MODULUS),
                        MODULUS
                    )

                // Compute sum_out_fold = count_out_eval + beta * fold_vals(beta, sum_out_evals)
                let sum_out_fold :=
                    addmod(
                        count_out_eval,
                        mulmod(beta, fold_vals(beta, add(sum_out_evals_ptr, WORD_SIZE), mload(sum_out_evals_ptr)), MODULUS),
                        MODULUS
                    )

                // Get the g_in_star and g_out_star evaluations
                let g_in_star_eval := builder_consume_final_round_mle(builder_ptr)
                let g_out_star_eval := builder_consume_final_round_mle(builder_ptr)

                // First constraint: sum g_in_star * sel_in * sum_in_fold - g_out_star * sum_out_fold = 0
                builder_produce_zerosum_constraint(
                    builder_ptr,
                    addmod(
                        mulmod(mulmod(g_in_star_eval, selection_eval, MODULUS), sum_in_fold, MODULUS),
                        mulmod(mulmod(g_out_star_eval, sum_out_fold, MODULUS), MODULUS_MINUS_ONE, MODULUS),
                        MODULUS
                    ),
                    3
                )

                // Second constraint: g_in_star + g_in_star * g_in_fold - input_chi_eval = 0
                builder_produce_identity_constraint(
                    builder_ptr,
                    addmod(
                        addmod(g_in_star_eval, mulmod(g_in_star_eval, g_in_fold, MODULUS), MODULUS),
                        mulmod(input_chi_eval, MODULUS_MINUS_ONE, MODULUS),
                        MODULUS
                    ),
                    2
                )

                // Third constraint: g_out_star + g_out_star * g_out_fold - output_chi_eval = 0
                builder_produce_identity_constraint(
                    builder_ptr,
                    addmod(
                        addmod(g_out_star_eval, mulmod(g_out_star_eval, g_out_fold, MODULUS), MODULUS),
                        mulmod(output_chi_eval, MODULUS_MINUS_ONE, MODULUS),
                        MODULUS
                    ),
                    2
                )

                // Prepare result evaluations (group_by + sum + count columns)
                let evaluations_length := add(add(group_by_count, sum_count), 1)
                evaluations_ptr := mload(FREE_PTR)
                mstore(evaluations_ptr, evaluations_length)

                // Copy group by column evaluations
                for { let i := 0 } lt(i, group_by_count) { i := add(i, 1) } {
                    mstore(
                        add(add(evaluations_ptr, WORD_SIZE), mul(i, WORD_SIZE)),
                        mload(add(add(g_out_evals_ptr, WORD_SIZE), mul(i, WORD_SIZE)))
                    )
                }

                // Copy sum column evaluations
                for { let i := 0 } lt(i, sum_count) { i := add(i, 1) } {
                    mstore(
                        add(add(evaluations_ptr, WORD_SIZE), mul(add(i, group_by_count), WORD_SIZE)),
                        mload(add(add(sum_out_evals_ptr, WORD_SIZE), mul(i, WORD_SIZE)))
                    )
                }

                // Add count column evaluation
                mstore(
                    add(add(evaluations_ptr, WORD_SIZE), mul(add(group_by_count, sum_count), WORD_SIZE)), count_out_eval
                )

                // Update free memory pointer
                mstore(FREE_PTR, add(evaluations_ptr, mul(add(evaluations_length, 1), WORD_SIZE)))

                plan_ptr_out := plan_ptr
            }

            let __planOutOffset
            __planOutOffset, __evaluations := group_by_exec_evaluate(__plan.offset, __builder)
            __planOut.offset := __planOutOffset
            // slither-disable-next-line write-after-write
            __planOut.length := sub(__plan.length, sub(__planOutOffset, __plan.offset))
        }
        __evaluationsPtr = __evaluations;
        __builderOut = __builder;
    }
}
