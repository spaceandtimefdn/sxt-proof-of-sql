// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";
import {VerificationBuilder} from "../builder/VerificationBuilder.pre.sol";

/// @title MembershipCheck
/// @dev Library for handling membership checks
library MembershipCheck {
    /// @notice verifies that the first n rows of a column contain the first m rows of another column
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// membership_check_evaluate(builder_ptr, alpha, beta, chi_n_eval, chi_m_eval, column_evals, candidate_evals)
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the verification builder
    /// * `alpha` - a challenge
    /// * `beta` - a challenge
    /// * `chi_n_eval` - the evaluation of a column of 1s with length n, where n is the number of original column rows that should contain the relevant candidate rows
    /// * `chi_m_eval` - the evaluation of a column of 1s with length m, where m is the number of candidate rows that should be contained by the relevant column rows
    /// * `column_evals` - the evaluations of the containing set
    /// * `candidate_evals` - the evaluation of the contained set
    /// ##### Return Values
    /// * `multiplicity_eval` - the multiplicity evaluation used to verify membership
    /// @notice verifies that the first n rows of a column contain the first m rows of another column
    /// @param __builder The verification builder
    /// @param __alpha a challenge
    /// @param __beta a challenge
    /// @param __chiNEval The evaluation of a column of 1s with length n, where n is the number of original column rows that should contain the relevant candidate rows
    /// @param __chiMEval The evaluation of a column of 1s with length m, where m is the number of candidate rows that should be contained by the relevant column rows
    /// @param __columnEvals The evaluations of the containing set
    /// @param __candidateEvals The evaluation of the contained set
    /// @return __builderOut The verification builder result
    /// @return __multiplicityEval The multiplicity evaluation used to verify membership
    function __membershipCheckEvaluate( // solhint-disable-line gas-calldata-parameters
        VerificationBuilder.Builder memory __builder,
        uint256 __alpha,
        uint256 __beta,
        uint256 __chiNEval,
        uint256 __chiMEval,
        uint256[] memory __columnEvals,
        uint256[] memory __candidateEvals
    ) internal pure returns (VerificationBuilder.Builder memory __builderOut, uint256 __multiplicityEval) {
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Queue.pre.sol
            function dequeue(queue_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Array.pre.sol
            function get_array_element(arr_ptr, index) -> value {
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
            function builder_consume_rho_evaluation(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/MathUtil.pre.sol
            function mulmod_bn254(lhs, rhs) -> product {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/MathUtil.pre.sol
            function addmod_bn254(lhs, rhs) -> difference {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/MathUtil.pre.sol
            function submod_bn254(lhs, rhs) -> product {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/MathUtil.pre.sol
            function compute_fold(beta, evals) -> fold {
                revert(0, 0)
            }

            function membership_check_evaluate(
                builder_ptr, alpha, beta, chi_n_eval, chi_m_eval, column_evals, candidate_evals
            ) -> multiplicity_eval {
                let num_columns := mload(column_evals)
                let num_candidate_columns := mload(candidate_evals)
                if sub(num_columns, num_candidate_columns) { err(ERR_INTERNAL) }
                let c_fold := mulmod_bn254(compute_fold(beta, column_evals), alpha)
                let d_fold := mulmod_bn254(compute_fold(beta, candidate_evals), alpha)
                multiplicity_eval := builder_consume_final_round_mle(builder_ptr)
                let c_star_eval := builder_consume_final_round_mle(builder_ptr)
                let d_star_eval := builder_consume_final_round_mle(builder_ptr)

                // sum c_star * multiplicity_eval - d_star = 0
                builder_produce_zerosum_constraint(
                    builder_ptr, submod_bn254(mulmod_bn254(c_star_eval, multiplicity_eval), d_star_eval), 2
                )

                // c_star + c_fold * c_star - chi_n = 0
                builder_produce_identity_constraint(
                    builder_ptr,
                    submod_bn254(addmod_bn254(c_star_eval, mulmod_bn254(c_fold, c_star_eval)), chi_n_eval),
                    2
                )

                // d_star + d_fold * d_star - chi_m = 0
                builder_produce_identity_constraint(
                    builder_ptr,
                    submod_bn254(addmod_bn254(d_star_eval, mulmod_bn254(d_fold, d_star_eval)), chi_m_eval),
                    2
                )
            }

            __multiplicityEval :=
                membership_check_evaluate(
                    __builder, __alpha, __beta, __chiNEval, __chiMEval, __columnEvals, __candidateEvals
                )
        }
        __builderOut = __builder;
    }
}
