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
            // slither-disable-end cyclomatic-complexity
            function fold_log_star_evaluate(builder_ptr, alpha, beta, column_evals, chi_eval) -> star {
                let fold := mulmod_bn254(alpha, compute_fold(beta, column_evals))
                star := builder_consume_final_round_mle(builder_ptr)
                // star + fold * star - chi = 0
                builder_produce_identity_constraint(
                    builder_ptr, submod_bn254(addmod_bn254(star, mulmod_bn254(fold, star)), chi_eval), 2
                )
            }

            __star := fold_log_star_evaluate(__builder, __alpha, __beta, __columnEvals, __chiEval)
        }
        __builderOut = __builder;
    }
}
