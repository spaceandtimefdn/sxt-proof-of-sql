// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";
import {VerificationBuilder} from "../builder/VerificationBuilder.pre.sol";

/// @title UnionExec
/// @dev Library for handling union execution plans
library TableExec {
    /// @notice Evaluates a union execution plan
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// union_exec_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr, output_chi_eval
    /// ```
    /// ##### Parameters
    /// * `plan_ptr` - calldata pointer to the union execution plan
    /// * `builder_ptr` - memory pointer to the verification builder
    /// ##### Return Values
    /// * `plan_ptr_out` - pointer to the remaining plan after consuming the union execution plan
    /// * `evaluations_ptr` - pointer to the evaluations
    /// * `output_chi_eval` - pointer to the evaluation of a column of 1s with same length as output
    /// @notice Evaluates a union execution plan
    /// ##### Proof Plan Encoding
    /// The filter plan is encoded as follows:
    /// 1. The length of the input plan array
    /// 2. The input plans, in order
    /// @notice ##### Union execution plan
    /// The union execution plan is a representation of a union, such as `SELECT col from A UNION ALL SELECT col from B`.
    /// The plan accesses a union and returns its evaluations.
    /// @dev Evaluates a union execution plan
    /// @param __plan The union execution plan data
    /// @param __builder The verification builder
    /// @return __planOut The remaining plan after processing
    /// @return __builderOut The verification builder result
    /// @return __evaluationsPtr The evaluations pointer
    /// @return __outputChiEvaluation The output chi evaluation
    function __unionExecEvaluate( // solhint-disable-line gas-calldata-parameters
    bytes calldata __plan, VerificationBuilder.Builder memory __builder)
        external
        pure
        returns (
            bytes calldata __planOut,
            VerificationBuilder.Builder memory __builderOut,
            uint256[] memory __evaluationsPtr,
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
            // slither-disable-start cyclomatic-complexity
            // IMPORT-YUL ../proof_exprs/ProofExpr.pre.sol
            function proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                revert(0, 0)
            }
            // slither-disable-end cyclomatic-complexity
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_table_chi_evaluation(builder_ptr, table_num) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/FoldUtil.pre.sol
            function fold_expr_evals(plan_ptr, builder_ptr, input_chi_eval, beta, column_count) -> plan_ptr_out, fold {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/FoldUtil.pre.sol
            function fold_final_round_mles(builder_ptr, column_count, beta) -> fold, evaluations_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ProofPlan.pre.sol
            function proof_plan_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr, output_chi_eval {
                revert(0, 0)
            }
            // IMPORT-YUL EmptyExec.pre.sol
            function empty_exec_evaluate(builder_ptr) -> evaluations_ptr, output_chi_eval {
                revert(0, 0)
            }
            // IMPORT-YUL FilterExec.pre.sol
            function filter_exec_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr, output_chi_eval {
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
            // IMPORT-YUL TableExec.pre.sol
            function table_exec_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr, output_chi_eval {
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
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_singleton_chi_evaluation(builder_ptr) -> value {
                revert(0, 0)
            }

            function evaluate_plan_and_apply_fold_constraint_for_union(plan_ptr, builder_ptr, gamma, beta) ->
                plan_ptr_out,
                c_star,
                num_evaluations
            {
                let evaluations
                let output_chi
                plan_ptr_out, evaluations, output_chi := proof_plan_evaluate(plan_ptr, builder_ptr)
                c_star := builder_consume_final_round_mle(builder_ptr)
                num_evaluations := mload(evaluations)
                let c_fold := mulmod_bn254(gamma, compute_fold(beta, evaluations))

                // c_star + c_fold * c_star - chi_n_i = 0
                builder_produce_identity_constraint(
                    builder_ptr, addmod_bn254(c_star, submod_bn254(mulmod_bn254(c_fold, c_star), output_chi)), 2
                )
            }

            function union_exec_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr, output_chi_eval {
                let gamma := builder_consume_challenge(builder_ptr)
                let beta := builder_consume_challenge(builder_ptr)
                let star_sum
                let d_bar_fold

                {
                    let plan_count := shr(UINT64_PADDING_BITS, mload(plan_ptr))
                    plan_ptr := add(plan_ptr, UINT64_SIZE)

                    // TODO: Add error type
                    if lt(plan_count, 2) { revert(0, 0) }

                    let num_evaluations
                    plan_ptr, star_sum, num_evaluations :=
                        evaluate_plan_and_apply_fold_constraint_for_union(plan_ptr, builder_ptr, gamma, beta)

                    for { plan_count := sub(plan_count, 1) } plan_count { plan_count := sub(plan_count, 1) } {
                        let c_star
                        let num_evaluations_to_compare
                        plan_ptr, c_star, num_evaluations_to_compare :=
                            evaluate_plan_and_apply_fold_constraint_for_union(plan_ptr, builder_ptr, gamma, beta)
                        // TODO: Add error type
                        if sub(num_evaluations, num_evaluations_to_compare) { revert(0, 0) }
                        star_sum := addmod_bn254(star_sum, c_star)
                    }

                    let output_evaluations
                    d_bar_fold, output_evaluations := fold_final_round_mles(builder_ptr, num_evaluations, beta)
                }
                {
                    let d_star := builder_consume_final_round_mle(builder_ptr)

                    let chi_m_eval := builder_consume_chi_evaluation(builder_ptr)

                    // d_star + d_bar_fold * d_star - chi_m = 0
                    builder_produce_identity_constraint(
                        builder_ptr, addmod_bn254(d_star, submod_bn254(mulmod_bn254(d_bar_fold, d_star), chi_m_eval)), 2
                    )

                    // sum (sum c_star) - d_star = 0
                    builder_produce_zerosum_constraint(builder_ptr, submod_bn254(star_sum, d_star), 1)
                }
            }

            let __planOutOffset
            __planOutOffset, __evaluations, __outputChiEvaluation := union_exec_evaluate(__plan.offset, __builder)
            __planOut.offset := __planOutOffset
            // slither-disable-next-line write-after-write
            __planOut.length := sub(__plan.length, sub(__planOutOffset, __plan.offset))
        }
        __evaluationsPtr = __evaluations;
        __builderOut = __builder;
    }
}
