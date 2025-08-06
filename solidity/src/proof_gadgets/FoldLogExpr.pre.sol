// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";
import {VerificationBuilder} from "../builder/VerificationBuilder.pre.sol";

/// @title FoldLogExpr
/// @dev Library for handling the frequently used fold constraint
library FoldLogExpr {
    /// @notice Folds expression evaluations with beta challenge
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// fold_log_star_evaluate(builder_ptr, alpha, beta, column_evals, chi_eval) -> star
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the verification builder
    /// * `alpha` - challenge value
    /// * `beta` - challenge value
    /// * `column_evals` - columns to fold
    /// * `chi_eval` - column of ones with same length is the column evaluations
    /// ##### Return Values
    /// * `star` - consumed star value
    /// @param __builder The verification builder
    /// @param __alpha The alpha challenge value
    /// @param __beta The beta challenge value
    /// @param __columnEvals The columns to fold
    /// @param __chiEval The column of ones with same length is the column evaluations
    /// @return __builderOut The updated verification builder
    /// @return __star The consumed star value
    function __foldLogStarEvaluate( // solhint-disable-line gas-calldata-parameters
        VerificationBuilder.Builder memory __builder,
        uint256 __alpha,
        uint256 __beta,
        uint256[] memory __columnEvals,
        uint256 __chiEval
    ) external pure returns (VerificationBuilder.Builder memory __builderOut, uint256 __star) {
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
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_final_round_mle(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_produce_identity_constraint(builder_ptr, evaluation, degree) {
                revert(0, 0)
            }
            function fold_log_star_evaluate_from_fold(builder_ptr, fold, chi_eval) -> star {
                star := builder_consume_final_round_mle(builder_ptr)
                // star + fold * star - chi = 0
                builder_produce_identity_constraint(
                    builder_ptr, submod_bn254(addmod_bn254(star, mulmod_bn254(fold, star)), chi_eval), 2
                )
            }
            function fold_log_star_evaluate(builder_ptr, alpha, beta, column_evals, chi_eval) -> star {
                let fold := mulmod_bn254(alpha, compute_fold(beta, column_evals))
                star := fold_log_star_evaluate_from_fold(builder_ptr, fold, chi_eval)
            }

            __star := fold_log_star_evaluate(__builder, __alpha, __beta, __columnEvals, __chiEval)
        }
        __builderOut = __builder;
    }

    /// @notice Folds expression evaluations coming from column evaluations with beta challenge
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// fold_log_star_evaluate_from_column_exprs(plan_ptr, builder_ptr, alpha, beta, column_count, chi_eval) -> plan_ptr_out, star
    /// ```
    /// ##### Parameters
    /// * `plan_ptr` - memory pointer to the plan data
    /// * `builder_ptr` - memory pointer to the verification builder
    /// * `alpha` - challenge value
    /// * `beta` - challenge value
    /// * `column_count` - number of columns to evaluate and fold
    /// * `chi_eval` - column of ones with same length as the column evaluations
    /// ##### Return Values
    /// * `plan_ptr_out` - pointer to the remaining plan after consuming
    /// * `star` - consumed star value
    /// @param __plan The plan data
    /// @param __builder The verification builder
    /// @param __alpha The alpha challenge value
    /// @param __beta The beta challenge value
    /// @param __columnCount The number of columns to evaluate and fold
    /// @param __chiEval The column of ones with same length as the column evaluations
    /// @return __builderOut The updated verification builder
    /// @return __star The consumed star value
    function __foldLogStarEvaluateFromColumnExprs( // solhint-disable-line gas-calldata-parameters
        bytes calldata __plan,
        VerificationBuilder.Builder memory __builder,
        uint256 __alpha,
        uint256 __beta,
        uint256 __columnCount,
        uint256 __chiEval
    ) external pure returns (VerificationBuilder.Builder memory __builderOut, uint256 __star) {
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
            // IMPORT-YUL FoldLogExpr.pre.sol
            function fold_log_star_evaluate_from_fold(builder_ptr, fold, chi_eval) -> star {
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
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_column_evaluation(builder_ptr, column_num) -> eval {
                revert(0, 0)
            }
            // IMPORT-YUL FoldUtil.pre.sol
            function fold_column_expr_evals(plan_ptr, builder_ptr, beta, column_count) -> plan_ptr_out, fold {
                revert(0, 0)
            }
            function fold_log_star_evaluate_from_column_exprs(
                plan_ptr, builder_ptr, alpha, beta, column_count, chi_eval
            ) -> plan_ptr_out, star {
                let fold
                plan_ptr_out, fold := fold_column_expr_evals(plan_ptr, builder_ptr, beta, column_count)
                fold := mulmod_bn254(alpha, fold)
                star := fold_log_star_evaluate_from_fold(builder_ptr, fold, chi_eval)
            }

            let __planOutOffset
            // We don't need the plan out for testing, so we leave it off to avoid local variable limits
            __planOutOffset, __star :=
                fold_log_star_evaluate_from_column_exprs(
                    __plan.offset, __builder, __alpha, __beta, __columnCount, __chiEval
                )
        }
        __builderOut = __builder;
    }

    /// @notice Folds expression evaluations coming from mles with beta challenge
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// function fold_log_star_evaluate_from_mles(builder_ptr, alpha, beta, column_count, chi_eval) -> star, evaluations_ptr
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the verification builder
    /// * `alpha` - challenge value
    /// * `beta` - challenge value
    /// * `column_count` - number of columns to evaluate and fold
    /// * `chi_eval` - column of ones with same length as the column evaluations
    /// ##### Return Values
    /// * `star` - consumed star value
    /// * `evaluations_ptr` - the resulting mles
    /// @param __builder The verification builder
    /// @param __alpha The alpha challenge value
    /// @param __beta The beta challenge value
    /// @param __columnCount The number of columns to evaluate and fold
    /// @param __chiEval The column of ones with same length as the column evaluations
    /// @return __builderOut The updated verification builder
    /// @return __star The consumed star value
    /// @return __evaluations The resulting mles
    function __foldLogStarEvaluateFromMLEs( // solhint-disable-line gas-calldata-parameters
        VerificationBuilder.Builder memory __builder,
        uint256 __alpha,
        uint256 __beta,
        uint256 __columnCount,
        uint256 __chiEval
    )
        external
        pure
        returns (VerificationBuilder.Builder memory __builderOut, uint256 __star, uint256[] memory __evaluations)
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
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_first_round_mle(builder_ptr) -> value {
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
            // IMPORT-YUL FoldLogExpr.pre.sol
            function fold_log_star_evaluate_from_fold(builder_ptr, fold, chi_eval) -> star {
                revert(0, 0)
            }
            // IMPORT-YUL FoldUtil.pre.sol
            function fold_first_round_mles(builder_ptr, beta, column_count) -> fold, evaluations_ptr {
                revert(0, 0)
            }
            function fold_log_evaluate_from_mles(builder_ptr, alpha, beta, column_count, chi_eval) ->
                fold,
                star,
                evaluations_ptr
            {
                fold, evaluations_ptr := fold_first_round_mles(builder_ptr, beta, column_count)
                fold := mulmod_bn254(alpha, fold)
                star := fold_log_star_evaluate_from_fold(builder_ptr, fold, chi_eval)
            }
            function fold_log_star_evaluate_from_mles(builder_ptr, alpha, beta, column_count, chi_eval) ->
                star,
                evaluations_ptr
            {
                let fold
                fold, star, evaluations_ptr :=
                    fold_log_evaluate_from_mles(builder_ptr, alpha, beta, column_count, chi_eval)
            }

            __star, __evaluations :=
                fold_log_star_evaluate_from_mles(__builder, __alpha, __beta, __columnCount, __chiEval)
        }
        __builderOut = __builder;
    }

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
    /// @param __alpha The beta challenge value
    /// @param __beta The beta challenge value
    /// @param __columnCount The number of columns
    /// @return __builderOut The updated verification builder
    /// @return __star The consumed star value
    function __foldLogStarEvaluateFromExprEvals( // solhint-disable-line gas-calldata-parameters
        bytes calldata __plan,
        VerificationBuilder.Builder memory __builder,
        uint256 __inputChiEval,
        uint256 __alpha,
        uint256 __beta,
        uint256 __columnCount
    ) external pure returns (VerificationBuilder.Builder memory __builderOut, uint256 __star) {
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
            // IMPORT-YUL FoldLogExpr.pre.sol
            function fold_log_star_evaluate_from_fold(builder_ptr, fold, chi_eval) -> star {
                revert(0, 0)
            }
            // IMPORT-YUL FoldUtil.pre.sol
            function fold_expr_evals(plan_ptr, builder_ptr, input_chi_eval, beta, column_count) -> plan_ptr_out, fold {
                revert(0, 0)
            }
            function fold_log_star_evaluate_from_expr_evals(
                plan_ptr, builder_ptr, input_chi_eval, alpha, beta, column_count
            ) -> plan_ptr_out, star {
                let fold
                plan_ptr, fold := fold_expr_evals(plan_ptr, builder_ptr, input_chi_eval, beta, column_count)
                fold := mulmod_bn254(fold, alpha)
                star := fold_log_star_evaluate_from_fold(builder_ptr, fold, input_chi_eval)
                plan_ptr_out := plan_ptr
            }

            // We don't keep the plan out, because it isn't needed for testing.
            let __planOutOffset
            __planOutOffset, __star :=
                fold_log_star_evaluate_from_expr_evals(
                    __plan.offset, __builder, __inputChiEval, __alpha, __beta, __columnCount
                )
        }
        __builderOut = __builder;
    }
}
