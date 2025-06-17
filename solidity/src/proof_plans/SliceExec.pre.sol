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
    /// slice_exec_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr, output_chi_eval
    /// ```
    /// ##### Parameters
    /// * `plan_ptr` - calldata pointer to the slice execution plan
    /// * `builder_ptr` - memory pointer to the verification builder
    /// ##### Return Values
    /// * `plan_ptr_out` - pointer to the remaining plan after consuming the slice execution plan
    /// * `evaluations_ptr` - pointer to the evaluations
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
    /// @return __outputChiEvaluation The output chi evaluation
    function __sliceExecEvaluate( // solhint-disable-line gas-calldata-parameters
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
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_table_chi_evaluation(builder_ptr, table_num) -> value {
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
            // IMPORT-YUL ProjectionExec.pre.sol
            function get_proof_expr_evaluations(plan_ptr, builder_ptr, input_chi_eval) -> plan_ptr_out, evaluations_ptr
            {
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
            // IMPORT-YUL FilterExec.pre.sol
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
            function filter_exec_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr, output_chi_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ProofPlan.pre.sol
            function proof_plan_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr, output_chi_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/MathUtil.pre.sol
            function compute_fold(beta, evals) -> fold {
                revert(0, 0)
            }
            function skip_unused_slice_fields(plan_ptr) -> plan_ptr_out {
                plan_ptr := add(plan_ptr, UINT64_SIZE)
                let is_populated := shr(BOOLEAN_PADDING_BITS, calldataload(plan_ptr))
                plan_ptr := add(plan_ptr, BOOLEAN_SIZE)
                if is_populated { plan_ptr := add(plan_ptr, UINT64_SIZE) }
                plan_ptr_out := plan_ptr
            }

            function slice_exec_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr, output_chi_eval {
                let input_evaluations, input_chi_evaluation
                plan_ptr, input_evaluations, input_chi_evaluation := proof_plan_evaluate(plan_ptr, builder_ptr)
                output_chi_eval := builder_consume_chi_evaluation(builder_ptr)
                let selection_eval := builder_consume_chi_evaluation(builder_ptr)
                selection_eval := submod_bn254(builder_consume_chi_evaluation(builder_ptr), selection_eval)

                let c_fold, d_fold
                {
                    let alpha := builder_consume_challenge(builder_ptr)
                    let beta := builder_consume_challenge(builder_ptr)
                    c_fold := mulmod_bn254(alpha, compute_fold(beta, input_evaluations))
                    d_fold, evaluations_ptr := fold_final_round_mles(builder_ptr, beta, mload(input_evaluations))
                    d_fold := mulmod_bn254(alpha, d_fold)
                }
                verify_filter(builder_ptr, c_fold, d_fold, input_chi_evaluation, output_chi_eval, selection_eval)
                plan_ptr_out := skip_unused_slice_fields(plan_ptr)
            }

            let __planOutOffset
            __planOutOffset, __evaluations, __outputChiEvaluation := slice_exec_evaluate(__plan.offset, __builder)
            __planOut.offset := __planOutOffset
            // slither-disable-next-line write-after-write
            __planOut.length := sub(__plan.length, sub(__planOutOffset, __plan.offset))
        }
        __evaluationsPtr = __evaluations;
        __builderOut = __builder;
    }
}
