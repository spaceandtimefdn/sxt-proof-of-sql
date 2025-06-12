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

                let plan_count := shr(UINT64_PADDING_BITS, mload(plan_ptr))
                plan_ptr := add(plan_ptr, UINT64_SIZE)

                // TODO: Add error type
                if lt(plan_count, 2) { revert(0, 0) }

                let star_sum
                let num_evaluations
                plan_ptr, star_sum, num_evaluations :=
                    evaluate_plan_and_apply_fold_constraint_for_union(plan_ptr, builder_ptr, gamma, beta)

                for {} plan_count { plan_count := sub(plan_count, 1) } {
                    let c_star
                    let num_evaluations_to_compare
                    plan_ptr, c_star, num_evaluations_to_compare :=
                        evaluate_plan_and_apply_fold_constraint_for_union(plan_ptr, builder_ptr, gamma, beta)
                    // TODO: Add error type
                    if sub(num_evaluations, num_evaluations_to_compare) { revert(0, 0) }
                    star_sum := addmod_bn254(star_sum, c_star)
                }

                let d_bar_fold
                let output_evaluations
                d_bar_fold, output_evaluations := fold_final_round_mles(builder_ptr, num_evaluations, beta)
                let d_star := builder_consume_final_round_mle(builder_ptr)

                let chi_m_eval := builder_consume_chi_evaluation(builder_ptr)

                // d_star + d_bar_fold * d_star - chi_m = 0
                builder_produce_identity_constraint(
                    builder_ptr, addmod_bn254(d_star, submod_bn254(mulmod_bn254(d_bar_fold, d_star), chi_m_eval)), 2
                )

                // sum (sum c_star) - d_star = 0
                builder_produce_zerosum_constraint(builder_ptr, submod_bn254(star_sum, d_star), 1)
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
