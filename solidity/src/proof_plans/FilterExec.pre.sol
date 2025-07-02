// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";
import {VerificationBuilder} from "../builder/VerificationBuilder.pre.sol";

/// @title FilterExec
/// @dev Library for handling filter execution plans
library FilterExec {
    /// @notice Evaluates a filter execution plan
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// filter_exec_evaluate(plan_ptr, builder_ptr, accessor_ptr, one_evals) -> plan_ptr_out, evaluations_ptr, output_chi_eval
    /// ```
    /// ##### Parameters
    /// * `plan_ptr` - calldata pointer to the filter execution plan
    /// * `builder_ptr` - memory pointer to the verification builder
    /// ##### Return Values
    /// * `plan_ptr_out` - pointer to the remaining plan after consuming the filter execution plan
    /// * `evaluations_ptr` - pointer to the evaluations
    /// * `output_length` - the length of the column of ones
    /// * `output_chi_eval` - pointer to the evaluation of a column of 1s with same length as output
    /// @notice Evaluates two sub-expressions and produces identity constraints checking their equality
    /// @notice ##### Constraints
    /// * Inputs: \\(S=\texttt{selection}\\), \\(C_1,\ldots,C_\ell=\texttt{c}\\) with length \\(n\\), and thus \\(\chi_{[0,n)}=\texttt{input_chi_eval}\\).
    ///   Note: `proof_expr_evaluate` guarentees that the lengths of these inputs equals the lengths of \\(chi_{[0,n)}\\).
    ///         It also guarentees that the inputs have the correct evaluations.
    ///         So, because we are assuming the query is valid, we know that \\(S\\) must be a column of boolean values.
    /// * Outputs: \\(D_1,\ldots,D_\ell=\texttt{d}\\) with length \\(m\\), and thus \\(\chi_{[0,m)}=\texttt{output_chi_eval}\\).
    /// * Hints: \\(C^\star=\texttt{c_star}\\) and \\(D^\star=\texttt{d_star}\\)
    /// * Challenges: \\(\alpha=\texttt{alpha}\\), \\(\beta=\texttt{beta}\\)
    /// * Helpers: \\(\bar{C} \=\texttt{c_fold} :\equiv \sum_{i=1}^{\ell} C_i \beta^{\ell-i}\\) and \\(\bar{D} \=\texttt{d_fold} :\equiv \sum_{i=1}^{\ell} D_i \beta^{\ell-i}\\)
    /// * Constraints:
    /// \\[\begin{aligned}
    /// C^\star \cdot S - D^\star &\overset{\sum}{=} 0\\\\
    /// (1 + \alpha\cdot \bar{C})\cdot C^\star - \chi_{[0,n)} &\equiv 0\\\\
    /// (1 + \alpha\cdot \bar{D})\cdot D^\star - \chi_{[0,m)} &\equiv 0\\\\
    /// \alpha\cdot\bar{D}\cdot(\chi_{[0,m)}-1) &\equiv 0\\\\
    /// \end{aligned}\\]
    /// Note: the notation \\(A\overset{\sum}{=}B\\) is used to indicate the zero-sum constratin.
    /// That is, that the sum of the elements of \\(A\\) equals the sum of the elements in \\(B\\).
    /// @notice To satisfy these constraints, we have that \\[\begin{aligned}
    /// C^\star[i] &= \begin{cases} \frac{1}{1+\alpha\cdot \bar{C}[i]} & \text{ when } i < n\\\\ 0 & \text{else}\end{cases}\\\\
    /// D^\star[i] &= \begin{cases} \frac{1}{1+\alpha\cdot \bar{D}[i]} & \text{ when } i < m\\\\ 0 & \text{else}\end{cases}
    /// \end{aligned}\\]
    /// @notice ##### Proof of Correctness
    /// @notice **Theorem:** Consider columns \\(C_1,\ldots,C_\ell\\) and \\(S\\) of length \\(n\\), where \\(S[i]=0\text{ or }1\\) for all \\(i\\). Given columns \\(D_1,\ldots,D_\ell\\),
    /// we have that
    /// \\[\begin{aligned}
    /// \\{(D_1[i],\ldots,D_\ell[i])\mid i<m\\} &= \\{(C_1[i],\ldots,C_\ell[i])\mid S[i]=1\\}\\\\
    /// \end{aligned}\\]
    /// if and only if
    /// \\[\sum_{i<n}\frac{S[i]}{1+\alpha\left(\beta^{\ell-1}C_0[i]+\cdots+C_\ell[i]\right)}=\sum_{i<m}\frac{1}{1+\alpha\left(\beta^{\ell-1}D_0[i]+\cdots+D_\ell[i]\right)}\\]
    /// @notice **Completeness Proof:**
    /// TODO
    /// @notice **Soundness Proof:**
    /// TODO
    /// ##### Proof Plan Encoding
    /// The filter plan is encoded as follows:
    /// 1. The index of the table being read from (64 bit integer)
    /// 2. The selection/filtering condition expression
    /// 3. The number of input/output columns (64 bit integer)
    /// 4. The input column expressions, in order
    /// @dev Evaluates a filter execution plan by checking the filter condition on each row
    /// @param __plan The filter execution plan data
    /// @param __builder The verification builder
    /// @return __planOut The remaining plan after processing
    /// @return __builderOut The verification builder result
    /// @return __evaluationsPtr The evaluations pointer
    /// @return __outputLength The length of the output chi evaluation
    /// @return __outputChiEvaluation The output chi evaluation
    function __filterExecEvaluate( // solhint-disable-line gas-calldata-parameters
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
            function dequeue_uint512(queue_ptr) -> value {
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
            // IMPORT-YUL ../proof_exprs/CastExpr.pre.sol
            function cast_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_bit_distribution(builder_ptr) -> vary_mask, leading_bit_mask {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_chi_evaluation_with_length(builder_ptr) -> length, chi_eval {
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
            // IMPORT-YUL ../proof_gadgets/FoldUtil.pre.sol
            function fold_expr_evals(plan_ptr, builder_ptr, input_chi_eval, beta, column_count) -> plan_ptr_out, fold {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/FoldUtil.pre.sol
            function fold_final_round_mles(builder_ptr, beta, column_count) -> fold, evaluations_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/EvaluationUtil.pre.sol
            function evaluate_proof_exprs(plan_ptr, builder_ptr, input_chi_eval, column_count) ->
                plan_ptr_out,
                evaluations_ptr
            {
                revert(0, 0)
            }

            function compute_filter_folds(plan_ptr, builder_ptr, input_chi_eval, beta) ->
                plan_ptr_out,
                c_fold,
                d_fold,
                evaluations_ptr
            {
                let column_count := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                plan_ptr := add(plan_ptr, UINT64_SIZE)
                {
                    let expr_evaluations
                    plan_ptr, expr_evaluations :=
                        evaluate_proof_exprs(plan_ptr, builder_ptr, input_chi_eval, column_count)
                    c_fold := compute_fold(beta, expr_evaluations)
                }

                d_fold, evaluations_ptr := fold_final_round_mles(builder_ptr, beta, column_count)
                plan_ptr_out := plan_ptr
            }

            function verify_filter(builder_ptr, c_fold, d_fold, input_chi_eval, output_chi_eval, selection_eval) {
                let c_star := builder_consume_final_round_mle(builder_ptr)
                let d_star := builder_consume_final_round_mle(builder_ptr)

                builder_produce_identity_constraint(
                    builder_ptr, submod_bn254(mulmod_bn254(addmod_bn254(1, c_fold), c_star), input_chi_eval), 2
                )
                builder_produce_identity_constraint(
                    builder_ptr, submod_bn254(mulmod_bn254(addmod_bn254(1, d_fold), d_star), output_chi_eval), 2
                )
                builder_produce_zerosum_constraint(
                    builder_ptr, submod_bn254(mulmod_bn254(c_star, selection_eval), d_star), 2
                )
                builder_produce_identity_constraint(
                    builder_ptr, mulmod_bn254(d_fold, submod_bn254(output_chi_eval, 1)), 2
                )
            }

            // IMPORT-YUL TableExec.pre.sol
            function table_exec_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr, output_chi_eval {
                revert(0, 0)
            }

            function filter_exec_evaluate(plan_ptr, builder_ptr) ->
                plan_ptr_out,
                evaluations_ptr,
                output_length,
                output_chi_eval
            {
                let input_chi_eval, selection_eval, c_fold, d_fold
                {
                    let alpha := builder_consume_challenge(builder_ptr)
                    let beta := builder_consume_challenge(builder_ptr)

                    input_chi_eval :=
                        builder_get_table_chi_evaluation(builder_ptr, shr(UINT64_PADDING_BITS, calldataload(plan_ptr)))
                    plan_ptr := add(plan_ptr, UINT64_SIZE)

                    plan_ptr, selection_eval := proof_expr_evaluate(plan_ptr, builder_ptr, input_chi_eval)

                    plan_ptr, c_fold, d_fold, evaluations_ptr :=
                        compute_filter_folds(plan_ptr, builder_ptr, input_chi_eval, beta)
                    c_fold := mulmod_bn254(alpha, c_fold)
                    d_fold := mulmod_bn254(alpha, d_fold)
                }
                output_length, output_chi_eval := builder_consume_chi_evaluation_with_length(builder_ptr)

                verify_filter(builder_ptr, c_fold, d_fold, input_chi_eval, output_chi_eval, selection_eval)

                plan_ptr_out := plan_ptr
            }

            let __planOutOffset
            __planOutOffset, __evaluations, __outputLength, __outputChiEvaluation :=
                filter_exec_evaluate(__plan.offset, __builder)
            __planOut.offset := __planOutOffset
            // slither-disable-next-line write-after-write
            __planOut.length := sub(__plan.length, sub(__planOutOffset, __plan.offset))
        }
        __evaluationsPtr = __evaluations;
        __builderOut = __builder;
    }
}
