// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";

library Verifier {
    function __verify(
        bytes calldata __result,
        bytes calldata __plan,
        bytes calldata __proof,
        uint256[] memory __tableLengths,
        uint256[] memory __commitments
    ) public view {
        assembly {
            // IMPORTED-YUL ../base/Array.pre.sol::get_array_element
            function exclude_coverage_start_get_array_element() {} // solhint-disable-line no-empty-blocks
            function get_array_element(arr_ptr, index) -> value {
                let arr := mload(arr_ptr)
                let length := mload(arr)
                if iszero(lt(index, length)) { err(ERR_INVALID_INDEX) }
                value := mload(add(add(arr, WORD_SIZE), mul(index, WORD_SIZE)))
            }
            function exclude_coverage_stop_get_array_element() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../base/Array.pre.sol::read_uint64_array
            function exclude_coverage_start_read_uint64_array() {} // solhint-disable-line no-empty-blocks
            function read_uint64_array(source_ptr) -> source_ptr_out, array_ptr {
                array_ptr := mload(FREE_PTR)

                let length := shr(UINT64_PADDING_BITS, calldataload(source_ptr))
                mstore(array_ptr, length)
                source_ptr := add(source_ptr, UINT64_SIZE)

                let tmp_ptr := add(array_ptr, WORD_SIZE)
                for {} length { length := sub(length, 1) } {
                    mstore(tmp_ptr, shr(UINT64_PADDING_BITS, calldataload(source_ptr)))
                    source_ptr := add(source_ptr, UINT64_SIZE)
                    tmp_ptr := add(tmp_ptr, WORD_SIZE)
                }

                mstore(FREE_PTR, tmp_ptr)

                source_ptr_out := source_ptr
            }
            function exclude_coverage_stop_read_uint64_array() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../base/Array.pre.sol::read_word_array
            function exclude_coverage_start_read_word_array() {} // solhint-disable-line no-empty-blocks
            function read_word_array(source_ptr) -> source_ptr_out, array_ptr {
                array_ptr := mload(FREE_PTR)

                let length := shr(UINT64_PADDING_BITS, calldataload(source_ptr))
                mstore(array_ptr, length)
                source_ptr := add(source_ptr, UINT64_SIZE)

                let target_ptr := add(array_ptr, WORD_SIZE)
                let copy_size := mul(length, WORD_SIZE)
                calldatacopy(target_ptr, source_ptr, copy_size)

                mstore(FREE_PTR, add(target_ptr, copy_size))

                source_ptr_out := add(source_ptr, copy_size)
            }
            function exclude_coverage_stop_read_word_array() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../base/Array.pre.sol::read_wordx2_array
            function exclude_coverage_start_read_wordx2_array() {} // solhint-disable-line no-empty-blocks
            function read_wordx2_array(source_ptr) -> source_ptr_out, array_ptr {
                // Allocate space for array length
                array_ptr := mload(FREE_PTR)

                let length := shr(UINT64_PADDING_BITS, calldataload(source_ptr))
                mstore(array_ptr, length)
                source_ptr := add(source_ptr, UINT64_SIZE)

                let target_ptr := add(array_ptr, WORD_SIZE)
                let copy_size := mul(length, WORDX2_SIZE)
                calldatacopy(target_ptr, source_ptr, copy_size)

                mstore(FREE_PTR, add(target_ptr, copy_size))

                source_ptr_out := add(source_ptr, copy_size)
            }
            function exclude_coverage_stop_read_wordx2_array() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../base/ECPrecompiles.pre.sol::calldata_ec_add_assign
            function exclude_coverage_start_calldata_ec_add_assign() {} // solhint-disable-line no-empty-blocks
            function calldata_ec_add_assign(args_ptr, c_ptr) {
                calldatacopy(add(args_ptr, WORDX2_SIZE), c_ptr, WORDX2_SIZE)
                ec_add(args_ptr)
            }
            function exclude_coverage_stop_calldata_ec_add_assign() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../base/ECPrecompiles.pre.sol::calldata_ec_mul_add_assign
            function exclude_coverage_start_calldata_ec_mul_add_assign() {} // solhint-disable-line no-empty-blocks
            function calldata_ec_mul_add_assign(args_ptr, c_ptr, scalar) {
                calldatacopy(add(args_ptr, WORDX2_SIZE), c_ptr, WORDX2_SIZE)
                ec_mul_assign(add(args_ptr, WORDX2_SIZE), scalar)
                ec_add(args_ptr)
            }
            function exclude_coverage_stop_calldata_ec_mul_add_assign() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../base/ECPrecompiles.pre.sol::constant_ec_mul_add_assign
            function exclude_coverage_start_constant_ec_mul_add_assign() {} // solhint-disable-line no-empty-blocks
            function constant_ec_mul_add_assign(args_ptr, c_x, c_y, scalar) {
                mstore(add(args_ptr, WORDX2_SIZE), c_x)
                mstore(add(args_ptr, WORDX3_SIZE), c_y)
                ec_mul_assign(add(args_ptr, WORDX2_SIZE), scalar)
                ec_add(args_ptr)
            }
            function exclude_coverage_stop_constant_ec_mul_add_assign() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../base/ECPrecompiles.pre.sol::ec_add
            function exclude_coverage_start_ec_add() {} // solhint-disable-line no-empty-blocks
            function ec_add(args_ptr) {
                if iszero(staticcall(ECADD_GAS, ECADD_ADDRESS, args_ptr, WORDX4_SIZE, args_ptr, WORDX2_SIZE)) {
                    err(ERR_INVALID_EC_ADD_INPUTS)
                }
            }
            function exclude_coverage_stop_ec_add() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../base/ECPrecompiles.pre.sol::ec_add_assign
            function exclude_coverage_start_ec_add_assign() {} // solhint-disable-line no-empty-blocks
            function ec_add_assign(args_ptr, c_ptr) {
                mcopy(add(args_ptr, WORDX2_SIZE), c_ptr, WORDX2_SIZE)
                ec_add(args_ptr)
            }
            function exclude_coverage_stop_ec_add_assign() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../base/ECPrecompiles.pre.sol::ec_mul
            function exclude_coverage_start_ec_mul() {} // solhint-disable-line no-empty-blocks
            function ec_mul(args_ptr) {
                if iszero(staticcall(ECMUL_GAS, ECMUL_ADDRESS, args_ptr, WORDX3_SIZE, args_ptr, WORDX2_SIZE)) {
                    err(ERR_INVALID_EC_MUL_INPUTS)
                }
            }
            function exclude_coverage_stop_ec_mul() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../base/ECPrecompiles.pre.sol::ec_mul_assign
            function exclude_coverage_start_ec_mul_assign() {} // solhint-disable-line no-empty-blocks
            function ec_mul_assign(args_ptr, scalar) {
                mstore(add(args_ptr, WORDX2_SIZE), scalar)
                ec_mul(args_ptr)
            }
            function exclude_coverage_stop_ec_mul_assign() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../base/ECPrecompiles.pre.sol::ec_pairing_x2
            function exclude_coverage_start_ec_pairing_x2() {} // solhint-disable-line no-empty-blocks
            function ec_pairing_x2(args_ptr) -> success {
                if iszero(staticcall(ECPAIRINGX2_GAS, ECPAIRING_ADDRESS, args_ptr, WORDX12_SIZE, args_ptr, WORD_SIZE)) {
                    err(ERR_INVALID_EC_PAIRING_INPUTS)
                }
                success := mload(args_ptr)
            }
            function exclude_coverage_stop_ec_pairing_x2() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../base/Errors.sol::err
            function exclude_coverage_start_err() {} // solhint-disable-line no-empty-blocks
            function err(code) {
                mstore(0, code)
                revert(28, 4)
            }
            function exclude_coverage_stop_err() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../base/LagrangeBasisEvaluation.pre.sol::compute_truncated_lagrange_basis_inner_product
            function exclude_coverage_start_compute_truncated_lagrange_basis_inner_product() {} // solhint-disable-line no-empty-blocks
            function compute_truncated_lagrange_basis_inner_product(length, x_ptr, y_ptr, num_vars) -> result {
                let part := 0 // This is g in the formulas
                result := 1 // This is h in the formulas
                for {} num_vars {} {
                    let x := mload(x_ptr)
                    let y := mload(y_ptr)
                    let xy := mulmod(x, y, MODULUS)
                    // let c := 1 - x
                    // let d := 1 - y
                    let cd := sub(add(MODULUS_PLUS_ONE, xy), addmod(x, y, MODULUS))
                    switch and(length, 1)
                    case 0 { part := mulmod(part, cd, MODULUS) }
                    default { part := add(mulmod(result, cd, MODULUS), mulmod(part, xy, MODULUS)) }
                    result := mulmod(result, add(cd, xy), MODULUS)
                    num_vars := sub(num_vars, 1)
                    length := shr(1, length)
                    x_ptr := add(x_ptr, WORD_SIZE)
                    y_ptr := add(y_ptr, WORD_SIZE)
                }
                if iszero(length) { result := mod(part, MODULUS) } // we return g in "short" cases
            }
            function exclude_coverage_stop_compute_truncated_lagrange_basis_inner_product() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../base/LagrangeBasisEvaluation.pre.sol::compute_truncated_lagrange_basis_sum
            function exclude_coverage_start_compute_truncated_lagrange_basis_sum() {} // solhint-disable-line no-empty-blocks
            function compute_truncated_lagrange_basis_sum(length, x_ptr, num_vars) -> result {
                result := 0

                // Invariant that holds within the for loop:
                // 0 <= result <= modulus + 1
                // This invariant reduces modulus operations.
                for {} num_vars {} {
                    switch and(length, 1)
                    case 0 { result := mulmod(result, sub(MODULUS_PLUS_ONE, mod(mload(x_ptr), MODULUS)), MODULUS) }
                    default {
                        result := sub(MODULUS_PLUS_ONE, mulmod(sub(MODULUS_PLUS_ONE, result), mload(x_ptr), MODULUS))
                    }
                    num_vars := sub(num_vars, 1)
                    length := shr(1, length)
                    x_ptr := add(x_ptr, WORD_SIZE)
                }
                switch length
                case 0 { result := mod(result, MODULUS) }
                default { result := 1 }
            }
            function exclude_coverage_stop_compute_truncated_lagrange_basis_sum() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../base/MathUtil.sol::log2_up
            function exclude_coverage_start_log2_up() {} // solhint-disable-line no-empty-blocks
            function log2_up(value) -> exponent {
                if value { value := sub(value, 1) }
                exponent := 1
                if gt(value, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF) {
                    exponent := add(exponent, 128)
                    value := shr(128, value)
                }
                if gt(value, 0xFFFFFFFFFFFFFFFF) {
                    exponent := add(exponent, 64)
                    value := shr(64, value)
                }
                if gt(value, 0xFFFFFFFF) {
                    exponent := add(exponent, 32)
                    value := shr(32, value)
                }
                if gt(value, 0xFFFF) {
                    exponent := add(exponent, 16)
                    value := shr(16, value)
                }
                if gt(value, 0xFF) {
                    exponent := add(exponent, 8)
                    value := shr(8, value)
                }
                if gt(value, 0xF) {
                    exponent := add(exponent, 4)
                    value := shr(4, value)
                }
                if gt(value, 0x3) {
                    exponent := add(exponent, 2)
                    value := shr(2, value)
                }
                if gt(value, 0x1) {
                    exponent := add(exponent, 1)
                    value := shr(1, value)
                }
            }
            function exclude_coverage_stop_log2_up() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../base/Queue.pre.sol::dequeue
            function exclude_coverage_start_dequeue() {} // solhint-disable-line no-empty-blocks
            function dequeue(queue_ptr) -> value {
                let queue := mload(queue_ptr)
                let length := mload(queue)
                if iszero(length) { err(ERR_EMPTY_QUEUE) }
                queue := add(queue, WORD_SIZE)
                value := mload(queue)
                mstore(queue, sub(length, 1))
                mstore(queue_ptr, queue)
            }
            function exclude_coverage_stop_dequeue() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../base/SwitchUtil.pre.sol::case_const
            function exclude_coverage_start_case_const() {} // solhint-disable-line no-empty-blocks
            function case_const(lhs, rhs) {
                if sub(lhs, rhs) { err(ERR_INCORRECT_CASE_CONST) }
            }
            function exclude_coverage_stop_case_const() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../base/Transcript.sol::append_array
            function exclude_coverage_start_append_array() {} // solhint-disable-line no-empty-blocks
            function append_array(transcript_ptr, array_ptr) {
                let array_len := mload(array_ptr)
                mstore(array_ptr, mload(transcript_ptr))
                mstore(transcript_ptr, keccak256(array_ptr, mul(add(array_len, 1), WORD_SIZE)))
                mstore(array_ptr, array_len)
            }
            function exclude_coverage_stop_append_array() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../base/Transcript.sol::append_calldata
            function exclude_coverage_start_append_calldata() {} // solhint-disable-line no-empty-blocks
            function append_calldata(transcript_ptr, offset, size) {
                let free_ptr := mload(FREE_PTR)
                mstore(free_ptr, mload(transcript_ptr))
                calldatacopy(add(free_ptr, WORD_SIZE), offset, size)
                mstore(transcript_ptr, keccak256(free_ptr, add(size, WORD_SIZE)))
            }
            function exclude_coverage_stop_append_calldata() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../base/Transcript.sol::draw_challenge
            function exclude_coverage_start_draw_challenge() {} // solhint-disable-line no-empty-blocks
            function draw_challenge(transcript_ptr) -> result {
                result := and(mload(transcript_ptr), MODULUS_MASK)
                mstore(transcript_ptr, keccak256(transcript_ptr, WORD_SIZE))
            }
            function exclude_coverage_stop_draw_challenge() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../base/Transcript.sol::draw_challenges
            function exclude_coverage_start_draw_challenges() {} // solhint-disable-line no-empty-blocks
            function draw_challenges(transcript_ptr, count) -> result_ptr {
                // allocate `count` words
                let free_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(free_ptr, mul(add(count, 1), WORD_SIZE)))
                // result is the pointer to the first word
                result_ptr := free_ptr
                // store count in the first word
                mstore(result_ptr, count)
                // increment to next word
                free_ptr := add(free_ptr, WORD_SIZE)
                // first challenge is the current transcript state
                let challenge := mload(transcript_ptr)
                for {} count {} {
                    mstore(transcript_ptr, challenge)

                    // store challenge in next word
                    mstore(free_ptr, and(challenge, MODULUS_MASK))
                    // hash challenge to get next challenge
                    challenge := keccak256(transcript_ptr, WORD_SIZE)
                    // increment to next word
                    free_ptr := add(free_ptr, WORD_SIZE)
                    // decrement count
                    count := sub(count, 1)
                }
                // The last (unused) challenge is the current state of the transcript
                mstore(transcript_ptr, challenge)
            }
            function exclude_coverage_stop_draw_challenges() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_consume_challenge
            function exclude_coverage_start_builder_consume_challenge() {} // solhint-disable-line no-empty-blocks
            function builder_consume_challenge(builder_ptr) -> challenge {
                challenge := dequeue(add(builder_ptr, BUILDER_CHALLENGES_OFFSET))
            }
            function exclude_coverage_stop_builder_consume_challenge() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_consume_chi_evaluation
            function exclude_coverage_start_builder_consume_chi_evaluation() {} // solhint-disable-line no-empty-blocks
            function builder_consume_chi_evaluation(builder_ptr) -> value {
                value := dequeue(add(builder_ptr, BUILDER_CHI_EVALUATIONS_OFFSET))
            }
            function exclude_coverage_stop_builder_consume_chi_evaluation() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_consume_final_round_mle
            function exclude_coverage_start_builder_consume_final_round_mle() {} // solhint-disable-line no-empty-blocks
            function builder_consume_final_round_mle(builder_ptr) -> value {
                value := dequeue(add(builder_ptr, BUILDER_FINAL_ROUND_MLES_OFFSET))
            }
            function exclude_coverage_stop_builder_consume_final_round_mle() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_get_aggregate_evaluation
            function exclude_coverage_start_builder_get_aggregate_evaluation() {} // solhint-disable-line no-empty-blocks
            function builder_get_aggregate_evaluation(builder_ptr) -> value {
                value := mload(add(builder_ptr, BUILDER_AGGREGATE_EVALUATION_OFFSET))
            }
            function exclude_coverage_stop_builder_get_aggregate_evaluation() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_get_chi_evaluations
            function exclude_coverage_start_builder_get_chi_evaluations() {} // solhint-disable-line no-empty-blocks
            function builder_get_chi_evaluations(builder_ptr) -> values_ptr {
                values_ptr := mload(add(builder_ptr, BUILDER_CHI_EVALUATIONS_OFFSET))
            }
            function exclude_coverage_stop_builder_get_chi_evaluations() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_get_column_evaluation
            function exclude_coverage_start_builder_get_column_evaluation() {} // solhint-disable-line no-empty-blocks
            function builder_get_column_evaluation(builder_ptr, column_num) -> value {
                value := get_array_element(add(builder_ptr, BUILDER_COLUMN_EVALUATIONS_OFFSET), column_num)
            }
            function exclude_coverage_stop_builder_get_column_evaluation() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_get_column_evaluations
            function exclude_coverage_start_builder_get_column_evaluations() {} // solhint-disable-line no-empty-blocks
            function builder_get_column_evaluations(builder_ptr) -> values_ptr {
                values_ptr := mload(add(builder_ptr, BUILDER_COLUMN_EVALUATIONS_OFFSET))
            }
            function exclude_coverage_stop_builder_get_column_evaluations() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_get_final_round_commitments
            function exclude_coverage_start_builder_get_final_round_commitments() {} // solhint-disable-line no-empty-blocks
            function builder_get_final_round_commitments(builder_ptr) -> values_ptr {
                values_ptr := mload(add(builder_ptr, BUILDER_FINAL_ROUND_COMMITMENTS_OFFSET))
            }
            function exclude_coverage_stop_builder_get_final_round_commitments() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_get_final_round_mles
            function exclude_coverage_start_builder_get_final_round_mles() {} // solhint-disable-line no-empty-blocks
            function builder_get_final_round_mles(builder_ptr) -> values_ptr {
                values_ptr := mload(add(builder_ptr, BUILDER_FINAL_ROUND_MLES_OFFSET))
            }
            function exclude_coverage_stop_builder_get_final_round_mles() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_get_first_round_commitments
            function exclude_coverage_start_builder_get_first_round_commitments() {} // solhint-disable-line no-empty-blocks
            function builder_get_first_round_commitments(builder_ptr) -> values_ptr {
                values_ptr := mload(add(builder_ptr, BUILDER_FIRST_ROUND_COMMITMENTS_OFFSET))
            }
            function exclude_coverage_stop_builder_get_first_round_commitments() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_get_first_round_mles
            function exclude_coverage_start_builder_get_first_round_mles() {} // solhint-disable-line no-empty-blocks
            function builder_get_first_round_mles(builder_ptr) -> values_ptr {
                values_ptr := mload(add(builder_ptr, BUILDER_FIRST_ROUND_MLES_OFFSET))
            }
            function exclude_coverage_stop_builder_get_first_round_mles() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_get_table_chi_evaluation
            function exclude_coverage_start_builder_get_table_chi_evaluation() {} // solhint-disable-line no-empty-blocks
            function builder_get_table_chi_evaluation(builder_ptr, table_num) -> value {
                value := get_array_element(add(builder_ptr, BUILDER_TABLE_CHI_EVALUATIONS_OFFSET), table_num)
            }
            function exclude_coverage_stop_builder_get_table_chi_evaluation() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_get_table_chi_evaluations
            function exclude_coverage_start_builder_get_table_chi_evaluations() {} // solhint-disable-line no-empty-blocks
            function builder_get_table_chi_evaluations(builder_ptr) -> values_ptr {
                values_ptr := mload(add(builder_ptr, BUILDER_TABLE_CHI_EVALUATIONS_OFFSET))
            }
            function exclude_coverage_stop_builder_get_table_chi_evaluations() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_new
            function exclude_coverage_start_builder_new() {} // solhint-disable-line no-empty-blocks
            function builder_new() -> builder_ptr {
                builder_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(builder_ptr, VERIFICATION_BUILDER_SIZE))
            }
            function exclude_coverage_stop_builder_new() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_produce_identity_constraint
            function exclude_coverage_start_builder_produce_identity_constraint() {} // solhint-disable-line no-empty-blocks
            function builder_produce_identity_constraint(builder_ptr, evaluation, degree) {
                if gt(add(degree, 1), mload(add(builder_ptr, BUILDER_MAX_DEGREE_OFFSET))) {
                    err(ERR_CONSTRAINT_DEGREE_TOO_HIGH)
                }
                // builder.aggregateEvaluation +=
                //     evaluation * dequeue(builder.constraintMultipliers) * builder.rowMultipliersEvaluation;
                mstore(
                    add(builder_ptr, BUILDER_AGGREGATE_EVALUATION_OFFSET),
                    addmod(
                        mload(add(builder_ptr, BUILDER_AGGREGATE_EVALUATION_OFFSET)),
                        mulmod(
                            evaluation,
                            mulmod(
                                dequeue(add(builder_ptr, BUILDER_CONSTRAINT_MULTIPLIERS_OFFSET)),
                                mload(add(builder_ptr, BUILDER_ROW_MULTIPLIERS_EVALUATION_OFFSET)),
                                MODULUS
                            ),
                            MODULUS
                        ),
                        MODULUS
                    )
                )
            }
            function exclude_coverage_stop_builder_produce_identity_constraint() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_produce_zerosum_constraint
            function exclude_coverage_start_builder_produce_zerosum_constraint() {} // solhint-disable-line no-empty-blocks
            function builder_produce_zerosum_constraint(builder_ptr, evaluation, degree) {
                if gt(degree, mload(add(builder_ptr, BUILDER_MAX_DEGREE_OFFSET))) {
                    err(ERR_CONSTRAINT_DEGREE_TOO_HIGH)
                }
                // builder.aggregateEvaluation += evaluation * dequeue(builder.constraintMultipliers)
                mstore(
                    add(builder_ptr, BUILDER_AGGREGATE_EVALUATION_OFFSET),
                    addmod(
                        mload(add(builder_ptr, BUILDER_AGGREGATE_EVALUATION_OFFSET)),
                        mulmod(evaluation, dequeue(add(builder_ptr, BUILDER_CONSTRAINT_MULTIPLIERS_OFFSET)), MODULUS),
                        MODULUS
                    )
                )
            }
            function exclude_coverage_stop_builder_produce_zerosum_constraint() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_set_aggregate_evaluation
            function exclude_coverage_start_builder_set_aggregate_evaluation() {} // solhint-disable-line no-empty-blocks
            function builder_set_aggregate_evaluation(builder_ptr, value) {
                mstore(add(builder_ptr, BUILDER_AGGREGATE_EVALUATION_OFFSET), value)
            }
            function exclude_coverage_stop_builder_set_aggregate_evaluation() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_set_bit_distributions
            function exclude_coverage_start_builder_set_bit_distributions() {} // solhint-disable-line no-empty-blocks
            function builder_set_bit_distributions(builder_ptr, values_ptr) {
                if mload(values_ptr) { err(ERR_UNSUPPORTED_PROOF) }
            }
            function exclude_coverage_stop_builder_set_bit_distributions() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_set_challenges
            function exclude_coverage_start_builder_set_challenges() {} // solhint-disable-line no-empty-blocks
            function builder_set_challenges(builder_ptr, challenges_ptr) {
                mstore(add(builder_ptr, BUILDER_CHALLENGES_OFFSET), challenges_ptr)
            }
            function exclude_coverage_stop_builder_set_challenges() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_set_chi_evaluations
            function exclude_coverage_start_builder_set_chi_evaluations() {} // solhint-disable-line no-empty-blocks
            function builder_set_chi_evaluations(builder_ptr, values_ptr) {
                mstore(add(builder_ptr, BUILDER_CHI_EVALUATIONS_OFFSET), values_ptr)
            }
            function exclude_coverage_stop_builder_set_chi_evaluations() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_set_column_evaluations
            function exclude_coverage_start_builder_set_column_evaluations() {} // solhint-disable-line no-empty-blocks
            function builder_set_column_evaluations(builder_ptr, values_ptr) {
                mstore(add(builder_ptr, BUILDER_COLUMN_EVALUATIONS_OFFSET), values_ptr)
            }
            function exclude_coverage_stop_builder_set_column_evaluations() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_set_constraint_multipliers
            function exclude_coverage_start_builder_set_constraint_multipliers() {} // solhint-disable-line no-empty-blocks
            function builder_set_constraint_multipliers(builder_ptr, values_ptr) {
                mstore(add(builder_ptr, BUILDER_CONSTRAINT_MULTIPLIERS_OFFSET), values_ptr)
            }
            function exclude_coverage_stop_builder_set_constraint_multipliers() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_set_final_round_commitments
            function exclude_coverage_start_builder_set_final_round_commitments() {} // solhint-disable-line no-empty-blocks
            function builder_set_final_round_commitments(builder_ptr, values_ptr) {
                mstore(add(builder_ptr, BUILDER_FINAL_ROUND_COMMITMENTS_OFFSET), values_ptr)
            }
            function exclude_coverage_stop_builder_set_final_round_commitments() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_set_final_round_mles
            function exclude_coverage_start_builder_set_final_round_mles() {} // solhint-disable-line no-empty-blocks
            function builder_set_final_round_mles(builder_ptr, values_ptr) {
                mstore(add(builder_ptr, BUILDER_FINAL_ROUND_MLES_OFFSET), values_ptr)
            }
            function exclude_coverage_stop_builder_set_final_round_mles() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_set_first_round_commitments
            function exclude_coverage_start_builder_set_first_round_commitments() {} // solhint-disable-line no-empty-blocks
            function builder_set_first_round_commitments(builder_ptr, values_ptr) {
                mstore(add(builder_ptr, BUILDER_FIRST_ROUND_COMMITMENTS_OFFSET), values_ptr)
            }
            function exclude_coverage_stop_builder_set_first_round_commitments() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_set_first_round_mles
            function exclude_coverage_start_builder_set_first_round_mles() {} // solhint-disable-line no-empty-blocks
            function builder_set_first_round_mles(builder_ptr, values_ptr) {
                mstore(add(builder_ptr, BUILDER_FIRST_ROUND_MLES_OFFSET), values_ptr)
            }
            function exclude_coverage_stop_builder_set_first_round_mles() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_set_max_degree
            function exclude_coverage_start_builder_set_max_degree() {} // solhint-disable-line no-empty-blocks
            function builder_set_max_degree(builder_ptr, value) {
                mstore(add(builder_ptr, BUILDER_MAX_DEGREE_OFFSET), value)
            }
            function exclude_coverage_stop_builder_set_max_degree() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_set_rho_evaluations
            function exclude_coverage_start_builder_set_rho_evaluations() {} // solhint-disable-line no-empty-blocks
            function builder_set_rho_evaluations(builder_ptr, values_ptr) {
                mstore(add(builder_ptr, BUILDER_RHO_EVALUATIONS_OFFSET), values_ptr)
            }
            function exclude_coverage_stop_builder_set_rho_evaluations() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_set_row_multipliers_evaluation
            function exclude_coverage_start_builder_set_row_multipliers_evaluation() {} // solhint-disable-line no-empty-blocks
            function builder_set_row_multipliers_evaluation(builder_ptr, value) {
                mstore(add(builder_ptr, BUILDER_ROW_MULTIPLIERS_EVALUATION_OFFSET), value)
            }
            function exclude_coverage_stop_builder_set_row_multipliers_evaluation() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_set_table_chi_evaluations
            function exclude_coverage_start_builder_set_table_chi_evaluations() {} // solhint-disable-line no-empty-blocks
            function builder_set_table_chi_evaluations(builder_ptr, values_ptr) {
                mstore(add(builder_ptr, BUILDER_TABLE_CHI_EVALUATIONS_OFFSET), values_ptr)
            }
            function exclude_coverage_stop_builder_set_table_chi_evaluations() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_check_aggregate_evaluation
            function exclude_coverage_start_builder_check_aggregate_evaluation() {} // solhint-disable-line no-empty-blocks
            function builder_check_aggregate_evaluation(builder_ptr) {
                if mload(add(builder_ptr, BUILDER_AGGREGATE_EVALUATION_OFFSET)) {
                    err(ERR_AGGREGATE_EVALUATION_MISMATCH)
                }
            }
            function exclude_coverage_stop_builder_check_aggregate_evaluation() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../hyperkzg/HyperKZGHelpers.pre.sol::bivariate_evaluation
            function exclude_coverage_start_bivariate_evaluation() {} // solhint-disable-line no-empty-blocks
            function bivariate_evaluation(v_ptr, q, d, ell) -> b {
                b := 0
                let v_stack := add(v_ptr, mul(WORDX3_SIZE, ell))
                for {} ell { ell := sub(ell, 1) } {
                    // tmp = v2i
                    v_stack := sub(v_stack, WORD_SIZE)
                    let tmp := calldataload(v_stack)
                    // tmp = v2i * d
                    tmp := mulmod(tmp, d, MODULUS)
                    // tmp += v1i
                    v_stack := sub(v_stack, WORD_SIZE)
                    tmp := addmod(tmp, calldataload(v_stack), MODULUS)
                    // tmp *= d
                    tmp := mulmod(tmp, d, MODULUS)
                    // tmp += v0i
                    v_stack := sub(v_stack, WORD_SIZE)
                    tmp := addmod(tmp, calldataload(v_stack), MODULUS)

                    // b *= q
                    b := mulmod(b, q, MODULUS)
                    // b += tmp
                    b := addmod(b, tmp, MODULUS)
                }
            }
            function exclude_coverage_stop_bivariate_evaluation() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../hyperkzg/HyperKZGHelpers.pre.sol::check_v_consistency
            function exclude_coverage_start_check_v_consistency() {} // solhint-disable-line no-empty-blocks
            function check_v_consistency(v_ptr, r, x, y) {
                let ell := mload(x)
                let v_stack := add(v_ptr, mul(WORDX3_SIZE, ell))
                x := add(x, mul(WORD_SIZE, add(ell, 1)))
                let last_v2 := y
                for {} ell { ell := sub(ell, 1) } {
                    v_stack := sub(v_stack, WORD_SIZE)
                    let v2i := calldataload(v_stack)
                    v_stack := sub(v_stack, WORD_SIZE)
                    let v1i := calldataload(v_stack)
                    v_stack := sub(v_stack, WORD_SIZE)
                    let v0i := calldataload(v_stack)
                    x := sub(x, WORD_SIZE)
                    let xi := mload(x)

                    // r * (2 * y + (xi - 1) * (v1i + v0i)) + xi * (v1i - v0i)
                    if addmod(
                        mulmod(
                            r,
                            addmod(
                                addmod(last_v2, last_v2, MODULUS),
                                mulmod(addmod(xi, MODULUS_MINUS_ONE, MODULUS), addmod(v1i, v0i, MODULUS), MODULUS),
                                MODULUS
                            ),
                            MODULUS
                        ),
                        mulmod(xi, addmod(v1i, sub(MODULUS, mod(v0i, MODULUS)), MODULUS), MODULUS),
                        MODULUS
                    ) { err(ERR_HYPER_KZG_INCONSISTENT_V) }

                    last_v2 := v2i
                }
            }
            function exclude_coverage_stop_check_v_consistency() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../hyperkzg/HyperKZGHelpers.pre.sol::compute_gl_msm
            function exclude_coverage_start_compute_gl_msm() {} // solhint-disable-line no-empty-blocks
            function compute_gl_msm(com_ptr, length, w_ptr, commitment_ptr, r, q, d, b, scratch) {
                univariate_group_evaluation(com_ptr, q, length, scratch)
                // g_l *= q
                ec_mul_assign(scratch, q)
                // g_l += commitment
                ec_add_assign(scratch, commitment_ptr)
                // g_l *= d * (d + 1) + 1
                ec_mul_assign(scratch, addmod(mulmod(d, addmod(d, 1, MODULUS), MODULUS), 1, MODULUS))
                // g_l += -G * b
                constant_ec_mul_add_assign(scratch, G1_NEG_GEN_X, G1_NEG_GEN_Y, b)

                let dr := mulmod(d, r, MODULUS)
                // g_l += w[0] * r
                calldata_ec_mul_add_assign(scratch, w_ptr, r)
                // g_l += w[1] * -d * r
                calldata_ec_mul_add_assign(scratch, add(w_ptr, WORDX2_SIZE), sub(MODULUS, dr))
                // g_l += w[2] * (d * r)^2
                calldata_ec_mul_add_assign(scratch, add(w_ptr, WORDX4_SIZE), mulmod(dr, dr, MODULUS))
            }
            function exclude_coverage_stop_compute_gl_msm() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../hyperkzg/HyperKZGHelpers.pre.sol::run_transcript
            function exclude_coverage_start_run_transcript() {} // solhint-disable-line no-empty-blocks
            function run_transcript(com_ptr, v_ptr, w_ptr, transcript_ptr, ell) -> r, q, d {
                append_calldata(transcript_ptr, com_ptr, mul(WORDX2_SIZE, sub(ell, 1)))
                r := draw_challenge(transcript_ptr)

                append_calldata(transcript_ptr, v_ptr, mul(WORDX3_SIZE, ell))
                q := draw_challenge(transcript_ptr)

                append_calldata(transcript_ptr, w_ptr, WORDX6_SIZE)
                d := draw_challenge(transcript_ptr)
            }
            function exclude_coverage_stop_run_transcript() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../hyperkzg/HyperKZGHelpers.pre.sol::univariate_group_evaluation
            function exclude_coverage_start_univariate_group_evaluation() {} // solhint-disable-line no-empty-blocks
            function univariate_group_evaluation(g_ptr, e, length, scratch) {
                switch length
                case 0 {
                    mstore(scratch, 0)
                    mstore(add(scratch, WORD_SIZE), 0)
                }
                default {
                    length := sub(length, 1)
                    g_ptr := add(g_ptr, mul(length, WORDX2_SIZE))
                    // result = g.pop()
                    calldatacopy(scratch, g_ptr, WORDX2_SIZE)
                    for {} length { length := sub(length, 1) } {
                        // g_l *= e
                        ec_mul_assign(scratch, e)
                        // g_l += com.pop()
                        g_ptr := sub(g_ptr, WORDX2_SIZE)
                        calldata_ec_add_assign(scratch, g_ptr)
                    }
                }
            }
            function exclude_coverage_stop_univariate_group_evaluation() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../hyperkzg/HyperKZGVerifier.pre.sol::verify_hyperkzg
            function exclude_coverage_start_verify_hyperkzg() {} // solhint-disable-line no-empty-blocks
            function verify_hyperkzg(proof_ptr, transcript_ptr, commitment_ptr, x, y) {
                function v_ptr(ptr, l) -> result {
                    result := add(ptr, add(UINT64_SIZE, sub(mul(WORDX2_SIZE, l), WORDX2_SIZE)))
                }
                function w_ptr(ptr, l) -> result {
                    result := add(ptr, add(UINT64_SIZE, sub(mul(WORDX5_SIZE, l), WORDX2_SIZE)))
                }

                let ell := mload(x)

                // if ell == 0, then error
                if iszero(ell) { err(ERR_HYPER_KZG_EMPTY_POINT) }
                {
                    let com_len := shr(UINT64_PADDING_BITS, calldataload(proof_ptr))
                    if sub(com_len, sub(ell, 1)) { err(ERR_HYPER_KZG_PROOF_SIZE_MISMATCH) }
                    proof_ptr := add(proof_ptr, UINT64_SIZE)
                    let v_len := shr(UINT64_PADDING_BITS, calldataload(add(proof_ptr, mul(WORDX2_SIZE, sub(ell, 1)))))
                    if sub(v_len, ell) { err(ERR_HYPER_KZG_PROOF_SIZE_MISMATCH) }
                }

                // Step 1: Run the transcript
                // WARNING: The public inputs (x, y, the commitments, digest of the KZG SRS, degree bound, etc) are
                // NOT included in the transcript and need to be added, either explicitly or implicitly,
                // before calling this function
                let r, q, d :=
                    run_transcript(proof_ptr, v_ptr(proof_ptr, ell), w_ptr(proof_ptr, ell), transcript_ptr, ell)

                // Step 2: Compute bivariate evaluation
                let b := bivariate_evaluation(v_ptr(proof_ptr, ell), q, d, ell)

                // Step 3: Check v consistency
                check_v_consistency(v_ptr(proof_ptr, ell), r, x, y)

                // Allocate scratch space for L, R, and the pairing check
                let scratch := mload(FREE_PTR)

                // Step 4: Compute L
                compute_gl_msm(proof_ptr, sub(ell, 1), w_ptr(proof_ptr, ell), commitment_ptr, r, q, d, b, scratch)

                // Step 5: Compute R
                univariate_group_evaluation(w_ptr(proof_ptr, ell), d, 3, add(scratch, WORDX6_SIZE))

                // Step 6: Verify the pairing equation
                mstore(add(scratch, WORDX2_SIZE), G2_NEG_GEN_X_IMAG)
                mstore(add(scratch, WORDX3_SIZE), G2_NEG_GEN_X_REAL)
                mstore(add(scratch, WORDX4_SIZE), G2_NEG_GEN_Y_IMAG)
                mstore(add(scratch, WORDX5_SIZE), G2_NEG_GEN_Y_REAL)
                mstore(add(scratch, WORDX8_SIZE), VK_TAU_HX_IMAG)
                mstore(add(scratch, WORDX9_SIZE), VK_TAU_HX_REAL)
                mstore(add(scratch, WORDX10_SIZE), VK_TAU_HY_IMAG)
                mstore(add(scratch, WORDX11_SIZE), VK_TAU_HY_REAL)
                if iszero(ec_pairing_x2(scratch)) { err(ERR_HYPER_KZG_PAIRING_CHECK_FAILED) }
            }
            function exclude_coverage_stop_verify_hyperkzg() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_exprs/ColumnExpr.pre.sol::column_expr_evaluate
            function exclude_coverage_start_column_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            function column_expr_evaluate(expr_ptr, builder_ptr) -> expr_ptr_out, eval {
                let column_num := shr(UINT64_PADDING_BITS, calldataload(expr_ptr))
                expr_ptr := add(expr_ptr, UINT64_SIZE)

                eval := builder_get_column_evaluation(builder_ptr, column_num)

                expr_ptr_out := expr_ptr
            }
            function exclude_coverage_stop_column_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_exprs/EqualsExpr.pre.sol::equals_expr_evaluate
            function exclude_coverage_start_equals_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            function equals_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                let lhs_eval
                expr_ptr, lhs_eval := proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval)

                let rhs_eval
                expr_ptr, rhs_eval := proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval)

                let diff_eval := addmod(lhs_eval, mulmod(MODULUS_MINUS_ONE, rhs_eval, MODULUS), MODULUS)
                let diff_star_eval := builder_consume_final_round_mle(builder_ptr)
                result_eval := mod(builder_consume_final_round_mle(builder_ptr), MODULUS)

                builder_produce_identity_constraint(builder_ptr, mulmod(result_eval, diff_eval, MODULUS), 2)
                builder_produce_identity_constraint(
                    builder_ptr,
                    addmod(
                        chi_eval,
                        mulmod(
                            MODULUS_MINUS_ONE,
                            addmod(mulmod(diff_eval, diff_star_eval, MODULUS), result_eval, MODULUS),
                            MODULUS
                        ),
                        MODULUS
                    ),
                    2
                )

                expr_ptr_out := expr_ptr
            }
            function exclude_coverage_stop_equals_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_exprs/LiteralExpr.pre.sol::literal_expr_evaluate
            function exclude_coverage_start_literal_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            function literal_expr_evaluate(expr_ptr, chi_eval) -> expr_ptr_out, eval {
                let literal_variant
                expr_ptr, literal_variant := read_data_type(expr_ptr)
                expr_ptr, eval := read_entry(expr_ptr, literal_variant)
                eval := mulmod(eval, chi_eval, MODULUS)
                expr_ptr_out := expr_ptr
            }
            function exclude_coverage_stop_literal_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_exprs/AddExpr.pre.sol::add_expr_evaluate
            function exclude_coverage_start_add_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            function add_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                let lhs_eval
                expr_ptr, lhs_eval := proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval)

                let rhs_eval
                expr_ptr, rhs_eval := proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval)

                result_eval := addmod(lhs_eval, rhs_eval, MODULUS)
                expr_ptr_out := expr_ptr
            }
            function exclude_coverage_stop_add_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_exprs/SubtractExpr.pre.sol::subtract_expr_evaluate
            function exclude_coverage_start_subtract_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            function subtract_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                let lhs_eval
                expr_ptr, lhs_eval := proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval)

                let rhs_eval
                expr_ptr, rhs_eval := proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval)

                result_eval := addmod(lhs_eval, mulmod(MODULUS_MINUS_ONE, rhs_eval, MODULUS), MODULUS)
                expr_ptr_out := expr_ptr
            }
            function exclude_coverage_stop_subtract_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_exprs/MultiplyExpr.pre.sol::multiply_expr_evaluate
            function exclude_coverage_start_multiply_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            function multiply_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                let lhs_eval
                expr_ptr, lhs_eval := proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval)

                let rhs_eval
                expr_ptr, rhs_eval := proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval)

                result_eval := mod(builder_consume_final_round_mle(builder_ptr), MODULUS)
                builder_produce_identity_constraint(
                    builder_ptr,
                    addmod(
                        result_eval, mulmod(MODULUS_MINUS_ONE, mulmod(lhs_eval, rhs_eval, MODULUS), MODULUS), MODULUS
                    ),
                    2
                )

                expr_ptr_out := expr_ptr
            }
            function exclude_coverage_stop_multiply_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_exprs/AndExpr.pre.sol::and_expr_evaluate
            function exclude_coverage_start_and_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            function and_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                let lhs_eval
                expr_ptr, lhs_eval := proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval)

                let rhs_eval
                expr_ptr, rhs_eval := proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval)

                result_eval := mod(builder_consume_final_round_mle(builder_ptr), MODULUS)
                builder_produce_identity_constraint(
                    builder_ptr,
                    addmod(
                        result_eval, mulmod(MODULUS_MINUS_ONE, mulmod(lhs_eval, rhs_eval, MODULUS), MODULUS), MODULUS
                    ),
                    2
                )

                expr_ptr_out := expr_ptr
            }
            function exclude_coverage_stop_and_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_exprs/OrExpr.pre.sol::or_expr_evaluate
            function exclude_coverage_start_or_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            function or_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                let lhs_eval
                expr_ptr, lhs_eval := proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval)

                let rhs_eval
                expr_ptr, rhs_eval := proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval)

                let lhs_times_rhs_eval := builder_consume_final_round_mle(builder_ptr)
                result_eval :=
                    addmod(
                        addmod(lhs_eval, rhs_eval, MODULUS), mulmod(MODULUS_MINUS_ONE, lhs_times_rhs_eval, MODULUS), MODULUS
                    )
                builder_produce_identity_constraint(
                    builder_ptr,
                    addmod(
                        lhs_times_rhs_eval,
                        mulmod(MODULUS_MINUS_ONE, mulmod(lhs_eval, rhs_eval, MODULUS), MODULUS),
                        MODULUS
                    ),
                    2
                )

                expr_ptr_out := expr_ptr
            }
            function exclude_coverage_stop_or_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_exprs/NotExpr.pre.sol::not_expr_evaluate
            function exclude_coverage_start_not_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            function not_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                let input_eval
                expr_ptr, input_eval := proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval)

                result_eval := addmod(chi_eval, mulmod(MODULUS_MINUS_ONE, input_eval, MODULUS), MODULUS)
                expr_ptr_out := expr_ptr
            }
            function exclude_coverage_stop_not_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_exprs/CastExpr.pre.sol::cast_expr_evaluate
            function exclude_coverage_start_cast_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            function cast_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                let data_type
                expr_ptr, data_type := read_data_type(expr_ptr)
                expr_ptr_out, result_eval := proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval)
            }
            function exclude_coverage_stop_cast_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_exprs/ProofExpr.pre.sol::proof_expr_evaluate
            function exclude_coverage_start_proof_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            function proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                let proof_expr_variant := shr(UINT32_PADDING_BITS, calldataload(expr_ptr))
                expr_ptr := add(expr_ptr, UINT32_SIZE)

                switch proof_expr_variant
                case 0 {
                    case_const(0, COLUMN_EXPR_VARIANT)
                    expr_ptr_out, eval := column_expr_evaluate(expr_ptr, builder_ptr)
                }
                case 1 {
                    case_const(1, LITERAL_EXPR_VARIANT)
                    expr_ptr_out, eval := literal_expr_evaluate(expr_ptr, chi_eval)
                }
                case 2 {
                    case_const(2, EQUALS_EXPR_VARIANT)
                    expr_ptr_out, eval := equals_expr_evaluate(expr_ptr, builder_ptr, chi_eval)
                }
                case 3 {
                    case_const(3, ADD_EXPR_VARIANT)
                    expr_ptr_out, eval := add_expr_evaluate(expr_ptr, builder_ptr, chi_eval)
                }
                case 4 {
                    case_const(4, SUBTRACT_EXPR_VARIANT)
                    expr_ptr_out, eval := subtract_expr_evaluate(expr_ptr, builder_ptr, chi_eval)
                }
                case 5 {
                    case_const(5, MULTIPLY_EXPR_VARIANT)
                    expr_ptr_out, eval := multiply_expr_evaluate(expr_ptr, builder_ptr, chi_eval)
                }
                case 6 {
                    case_const(6, AND_EXPR_VARIANT)
                    expr_ptr_out, eval := and_expr_evaluate(expr_ptr, builder_ptr, chi_eval)
                }
                case 7 {
                    case_const(7, OR_EXPR_VARIANT)
                    expr_ptr_out, eval := or_expr_evaluate(expr_ptr, builder_ptr, chi_eval)
                }
                case 8 {
                    case_const(8, NOT_EXPR_VARIANT)
                    expr_ptr_out, eval := not_expr_evaluate(expr_ptr, builder_ptr, chi_eval)
                }
                case 9 {
                    case_const(9, CAST_EXPR_VARIANT)
                    expr_ptr_out, eval := cast_expr_evaluate(expr_ptr, builder_ptr, chi_eval)
                }
                default { err(ERR_UNSUPPORTED_PROOF_EXPR_VARIANT) }
            }
            function exclude_coverage_stop_proof_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_plans/FilterExec.pre.sol::compute_folds
            function exclude_coverage_start_compute_folds() {} // solhint-disable-line no-empty-blocks
            function compute_folds(plan_ptr, builder_ptr, input_chi_eval) ->
                plan_ptr_out,
                c_fold,
                d_fold,
                evaluations_ptr
            {
                let beta := builder_consume_challenge(builder_ptr)

                let column_count := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                plan_ptr := add(plan_ptr, UINT64_SIZE)

                evaluations_ptr := mload(FREE_PTR)
                mstore(evaluations_ptr, column_count)
                evaluations_ptr := add(evaluations_ptr, WORD_SIZE)

                c_fold := 0
                for { let i := column_count } i { i := sub(i, 1) } {
                    let c
                    plan_ptr, c := proof_expr_evaluate(plan_ptr, builder_ptr, input_chi_eval)
                    c_fold := addmod(mulmod(c_fold, beta, MODULUS), c, MODULUS)
                }

                d_fold := 0
                for { let i := column_count } i { i := sub(i, 1) } {
                    let d := builder_consume_final_round_mle(builder_ptr)
                    d_fold := addmod(mulmod(d_fold, beta, MODULUS), d, MODULUS)

                    mstore(evaluations_ptr, d)
                    evaluations_ptr := add(evaluations_ptr, WORD_SIZE)
                }
                evaluations_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(evaluations_ptr, add(WORD_SIZE, mul(column_count, WORD_SIZE))))
                plan_ptr_out := plan_ptr
            }
            function exclude_coverage_stop_compute_folds() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_plans/FilterExec.pre.sol::filter_exec_evaluate
            function exclude_coverage_start_filter_exec_evaluate() {} // solhint-disable-line no-empty-blocks
            function filter_exec_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr {
                let alpha := builder_consume_challenge(builder_ptr)

                let input_chi_eval :=
                    builder_get_table_chi_evaluation(builder_ptr, shr(UINT64_PADDING_BITS, calldataload(plan_ptr)))
                plan_ptr := add(plan_ptr, UINT64_SIZE)

                let selection_eval
                plan_ptr, selection_eval := proof_expr_evaluate(plan_ptr, builder_ptr, input_chi_eval)

                let c_fold, d_fold
                plan_ptr, c_fold, d_fold, evaluations_ptr := compute_folds(plan_ptr, builder_ptr, input_chi_eval)
                let c_star := builder_consume_final_round_mle(builder_ptr)
                let d_star := builder_consume_final_round_mle(builder_ptr)
                let output_chi_eval := builder_consume_chi_evaluation(builder_ptr)

                builder_produce_zerosum_constraint(
                    builder_ptr,
                    addmod(mulmod(c_star, selection_eval, MODULUS), mulmod(MODULUS_MINUS_ONE, d_star, MODULUS), MODULUS),
                    2
                )
                builder_produce_identity_constraint(
                    builder_ptr,
                    addmod(
                        mulmod(add(1, mulmod(alpha, c_fold, MODULUS)), c_star, MODULUS),
                        mulmod(MODULUS_MINUS_ONE, input_chi_eval, MODULUS),
                        MODULUS
                    ),
                    2
                )
                builder_produce_identity_constraint(
                    builder_ptr,
                    addmod(
                        mulmod(add(1, mulmod(alpha, d_fold, MODULUS)), d_star, MODULUS),
                        mulmod(MODULUS_MINUS_ONE, output_chi_eval, MODULUS),
                        MODULUS
                    ),
                    2
                )
                builder_produce_identity_constraint(
                    builder_ptr,
                    mulmod(mulmod(alpha, d_fold, MODULUS), addmod(output_chi_eval, MODULUS_MINUS_ONE, MODULUS), MODULUS),
                    2
                )
                plan_ptr_out := plan_ptr
            }
            function exclude_coverage_stop_filter_exec_evaluate() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../sumcheck/Sumcheck.pre.sol::process_round
            function exclude_coverage_start_process_round() {} // solhint-disable-line no-empty-blocks
            function process_round(proof_ptr, degree, challenge) -> proof_ptr_out, round_evaluation, actual_sum {
                let coefficient := mod(calldataload(proof_ptr), MODULUS)
                proof_ptr := add(proof_ptr, WORD_SIZE)
                round_evaluation := coefficient
                actual_sum := coefficient
                for {} degree { degree := sub(degree, 1) } {
                    coefficient := calldataload(proof_ptr)
                    proof_ptr := add(proof_ptr, WORD_SIZE)
                    round_evaluation := mulmod(round_evaluation, challenge, MODULUS)
                    round_evaluation := addmod(round_evaluation, coefficient, MODULUS)
                    actual_sum := addmod(actual_sum, coefficient, MODULUS)
                }
                actual_sum := addmod(actual_sum, coefficient, MODULUS)
                proof_ptr_out := proof_ptr
            }
            function exclude_coverage_stop_process_round() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../sumcheck/Sumcheck.pre.sol::verify_sumcheck_proof
            function exclude_coverage_start_verify_sumcheck_proof() {} // solhint-disable-line no-empty-blocks
            function verify_sumcheck_proof(transcript_ptr, proof_ptr, num_vars) ->
                proof_ptr_out,
                evaluation_point_ptr,
                expected_evaluation,
                degree
            {
                append_calldata(transcript_ptr, proof_ptr, UINT64_SIZE)
                let sumcheck_length := shr(UINT64_PADDING_BITS, calldataload(proof_ptr))
                proof_ptr := add(proof_ptr, UINT64_SIZE)
                if or(or(iszero(num_vars), iszero(sumcheck_length)), mod(sumcheck_length, num_vars)) {
                    err(ERR_INVALID_SUMCHECK_PROOF_SIZE)
                }
                degree := sub(div(sumcheck_length, num_vars), 1)

                expected_evaluation := 0
                evaluation_point_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(evaluation_point_ptr, mul(WORD_SIZE, add(num_vars, 1))))
                let evaluation_ptr := evaluation_point_ptr
                mstore(evaluation_ptr, num_vars)
                evaluation_ptr := add(evaluation_ptr, WORD_SIZE)
                for {} num_vars { num_vars := sub(num_vars, 1) } {
                    append_calldata(transcript_ptr, proof_ptr, mul(WORD_SIZE, add(degree, 1)))
                    let challenge := and(mload(transcript_ptr), MODULUS_MASK)
                    mstore(evaluation_ptr, challenge)
                    evaluation_ptr := add(evaluation_ptr, WORD_SIZE)
                    let round_evaluation, actual_sum
                    proof_ptr, round_evaluation, actual_sum := process_round(proof_ptr, degree, challenge)
                    if sub(expected_evaluation, actual_sum) { err(ERR_ROUND_EVALUATION_MISMATCH) }
                    expected_evaluation := round_evaluation
                }
                proof_ptr_out := proof_ptr
            }
            function exclude_coverage_stop_verify_sumcheck_proof() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_plans/ProofPlan.pre.sol::proof_plan_evaluate
            function exclude_coverage_start_proof_plan_evaluate() {} // solhint-disable-line no-empty-blocks
            function proof_plan_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr {
                let proof_plan_variant := shr(UINT32_PADDING_BITS, calldataload(plan_ptr))
                plan_ptr := add(plan_ptr, UINT32_SIZE)

                switch proof_plan_variant
                case 0 {
                    case_const(0, FILTER_EXEC_VARIANT)
                    plan_ptr_out, evaluations_ptr := filter_exec_evaluate(plan_ptr, builder_ptr)
                }
                default { err(ERR_UNSUPPORTED_PROOF_PLAN_VARIANT) }
            }
            function exclude_coverage_stop_proof_plan_evaluate() {} // solhint-disable-line no-empty-blocks

            function read_first_round_message(proof_ptr_init, transcript_ptr, builder_ptr) ->
                proof_ptr,
                range_length,
                num_challenges
            {
                proof_ptr := proof_ptr_init

                range_length := shr(UINT64_PADDING_BITS, calldataload(proof_ptr))
                proof_ptr := add(proof_ptr, UINT64_SIZE)

                num_challenges := shr(UINT64_PADDING_BITS, calldataload(proof_ptr))
                proof_ptr := add(proof_ptr, UINT64_SIZE)

                let array_ptr

                proof_ptr, array_ptr := read_uint64_array(proof_ptr)
                builder_set_chi_evaluations(builder_ptr, array_ptr)

                proof_ptr, array_ptr := read_uint64_array(proof_ptr)
                builder_set_rho_evaluations(builder_ptr, array_ptr)

                proof_ptr, array_ptr := read_wordx2_array(proof_ptr)
                builder_set_first_round_commitments(builder_ptr, array_ptr)

                append_calldata(transcript_ptr, proof_ptr_init, sub(proof_ptr, proof_ptr_init))
            }
            function read_final_round_message(proof_ptr_init, transcript_ptr, builder_ptr) -> proof_ptr, num_constraints
            {
                proof_ptr := proof_ptr_init

                num_constraints := shr(UINT64_PADDING_BITS, calldataload(proof_ptr))
                proof_ptr := add(proof_ptr, UINT64_SIZE)

                let array_ptr

                proof_ptr, array_ptr := read_wordx2_array(proof_ptr)
                builder_set_final_round_commitments(builder_ptr, array_ptr)

                proof_ptr, array_ptr := read_wordx2_array(proof_ptr)
                builder_set_bit_distributions(builder_ptr, array_ptr)

                append_calldata(transcript_ptr, proof_ptr_init, sub(proof_ptr, proof_ptr_init))
            }
            function read_and_verify_sumcheck_proof(proof_ptr_init, transcript_ptr, builder_ptr, num_vars) ->
                proof_ptr,
                evaluation_point_ptr
            {
                let expected_evaluation, sumcheck_degree
                proof_ptr, evaluation_point_ptr, expected_evaluation, sumcheck_degree :=
                    verify_sumcheck_proof(transcript_ptr, proof_ptr_init, num_vars)
                builder_set_aggregate_evaluation(builder_ptr, mulmod(MODULUS_MINUS_ONE, expected_evaluation, MODULUS))
                builder_set_max_degree(builder_ptr, sumcheck_degree)
            }
            // IMPORTED-YUL ../base/LagrangeBasisEvaluation.pre.sol::compute_evaluations
            function exclude_coverage_start_compute_evaluations() {} // solhint-disable-line no-empty-blocks
            function compute_evaluations(evaluation_point_ptr, array_ptr) {
                let num_vars := mload(evaluation_point_ptr)
                let x := add(evaluation_point_ptr, WORD_SIZE)
                let array_len := mload(array_ptr)
                array_ptr := add(array_ptr, WORD_SIZE)
                for {} array_len { array_len := sub(array_len, 1) } {
                    mstore(array_ptr, compute_truncated_lagrange_basis_sum(mload(array_ptr), x, num_vars))
                    array_ptr := add(array_ptr, WORD_SIZE)
                }
            }
            function exclude_coverage_stop_compute_evaluations() {} // solhint-disable-line no-empty-blocks
            function read_pcs_evaluations(proof_ptr_init, transcript_ptr, builder_ptr) -> proof_ptr {
                proof_ptr := proof_ptr_init

                let array_ptr

                proof_ptr, array_ptr := read_word_array(proof_ptr)
                builder_set_first_round_mles(builder_ptr, array_ptr)

                proof_ptr, array_ptr := read_word_array(proof_ptr)
                builder_set_column_evaluations(builder_ptr, array_ptr)

                proof_ptr, array_ptr := read_word_array(proof_ptr)
                builder_set_final_round_mles(builder_ptr, array_ptr)

                append_calldata(transcript_ptr, proof_ptr_init, sub(proof_ptr, proof_ptr_init))
            }
            // IMPORTED-YUL PlanUtil.pre.sol::skip_plan_names
            function exclude_coverage_start_skip_plan_names() {} // solhint-disable-line no-empty-blocks
            function skip_plan_names(plan_ptr) -> plan_ptr_out {
                // skip over the table names
                let num_tables := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                plan_ptr := add(plan_ptr, UINT64_SIZE)
                for {} num_tables { num_tables := sub(num_tables, 1) } {
                    let name_len := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                    plan_ptr := add(plan_ptr, add(UINT64_SIZE, name_len))
                }
                // skip over the column names
                let num_columns := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                plan_ptr := add(plan_ptr, UINT64_SIZE)
                for {} num_columns { num_columns := sub(num_columns, 1) } {
                    plan_ptr := add(plan_ptr, UINT64_SIZE)
                    let name_len := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                    plan_ptr := add(plan_ptr, add(UINT64_SIZE, name_len))
                    let data_type
                    plan_ptr, data_type := read_data_type(plan_ptr)
                }
                // skip over the output column names
                let num_outputs := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                plan_ptr := add(plan_ptr, UINT64_SIZE)
                for {} num_outputs { num_outputs := sub(num_outputs, 1) } {
                    let name_len := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                    plan_ptr := add(plan_ptr, add(UINT64_SIZE, name_len))
                }

                plan_ptr_out := plan_ptr
            }
            function exclude_coverage_stop_skip_plan_names() {} // solhint-disable-line no-empty-blocks

            // IMPORTED-YUL ../hyperkzg/HyperKZGBatch.pre.sol::batch_pcs
            function exclude_coverage_start_batch_pcs() {} // solhint-disable-line no-empty-blocks
            function batch_pcs(args_ptr, transcript_ptr, commitments_ptr, evaluations_ptr, batch_eval) -> batch_eval_out
            {
                let num_commitments := mload(commitments_ptr)
                commitments_ptr := add(commitments_ptr, WORD_SIZE)
                let num_evaluations := mload(evaluations_ptr)
                evaluations_ptr := add(evaluations_ptr, WORD_SIZE)
                if sub(num_commitments, num_evaluations) { err(ERR_PCS_BATCH_LENGTH_MISMATCH) }
                for {} num_commitments { num_commitments := sub(num_commitments, 1) } {
                    let challenge := draw_challenge(transcript_ptr)
                    constant_ec_mul_add_assign(
                        args_ptr, mload(commitments_ptr), mload(add(commitments_ptr, WORD_SIZE)), challenge
                    )
                    commitments_ptr := add(commitments_ptr, WORDX2_SIZE)
                    batch_eval := addmod(batch_eval, mulmod(mload(evaluations_ptr), challenge, MODULUS), MODULUS)
                    evaluations_ptr := add(evaluations_ptr, WORD_SIZE)
                }
                batch_eval_out := mod(batch_eval, MODULUS)
            }
            function exclude_coverage_stop_batch_pcs() {} // solhint-disable-line no-empty-blocks

            // TODO: possibly move this to another file and add unit tests
            function verify_pcs_evaluations(
                proof_ptr, commitments_ptr, transcript_ptr, builder_ptr, evaluation_point_ptr
            ) {
                let batch_commitment_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(batch_commitment_ptr, WORDX5_SIZE))
                mstore(batch_commitment_ptr, 0)
                mstore(add(batch_commitment_ptr, WORD_SIZE), 0)
                let batch_eval := 0
                batch_eval :=
                    batch_pcs(
                        batch_commitment_ptr,
                        transcript_ptr,
                        builder_get_first_round_commitments(builder_ptr),
                        builder_get_first_round_mles(builder_ptr),
                        batch_eval
                    )
                batch_eval :=
                    batch_pcs(
                        batch_commitment_ptr,
                        transcript_ptr,
                        commitments_ptr,
                        builder_get_column_evaluations(builder_ptr),
                        batch_eval
                    )
                batch_eval :=
                    batch_pcs(
                        batch_commitment_ptr,
                        transcript_ptr,
                        builder_get_final_round_commitments(builder_ptr),
                        builder_get_final_round_mles(builder_ptr),
                        batch_eval
                    )

                verify_hyperkzg(proof_ptr, transcript_ptr, batch_commitment_ptr, evaluation_point_ptr, batch_eval)
            }

            // IMPORTED-YUL ../base/LagrangeBasisEvaluation.pre.sol::compute_evaluation_vec
            function exclude_coverage_start_compute_evaluation_vec() {} // solhint-disable-line no-empty-blocks
            function compute_evaluation_vec(length, evaluation_point_ptr) -> evaluations_ptr {
                evaluations_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(evaluations_ptr, mul(length, WORD_SIZE)))
                mstore(evaluations_ptr, 1)
                let num_vars := mload(evaluation_point_ptr)
                if gt(length, shl(num_vars, 1)) { err(ERR_EVALUATION_LENGTH_TOO_LARGE) }
                for { let len := 1 } num_vars { num_vars := sub(num_vars, 1) } {
                    let x := mod(mload(add(evaluation_point_ptr, mul(num_vars, WORD_SIZE))), MODULUS)
                    let one_minus_x := sub(MODULUS_PLUS_ONE, x)
                    len := mul(len, 2)
                    if gt(len, length) { len := length }
                    for { let l := len } l {} {
                        l := sub(l, 1)
                        let to_ptr := add(evaluations_ptr, mul(l, WORD_SIZE))
                        let from_ptr := add(evaluations_ptr, mul(shr(1, l), WORD_SIZE))
                        switch mod(l, 2)
                        case 0 { mstore(to_ptr, mulmod(mload(from_ptr), one_minus_x, MODULUS)) }
                        case 1 { mstore(to_ptr, mulmod(mload(from_ptr), x, MODULUS)) }
                    }
                }
            }
            function exclude_coverage_stop_compute_evaluation_vec() {} // solhint-disable-line no-empty-blocks

            // IMPORTED-YUL ../base/DataType.pre.sol::read_entry
            function exclude_coverage_start_read_entry() {} // solhint-disable-line no-empty-blocks
            function read_entry(result_ptr, data_type_variant) -> result_ptr_out, entry {
                result_ptr_out := result_ptr
                switch data_type_variant
                case 0 {
                    case_const(0, DATA_TYPE_BOOLEAN_VARIANT)
                    entry := shr(BOOLEAN_PADDING_BITS, calldataload(result_ptr))
                    if shr(1, entry) { err(ERR_INVALID_BOOLEAN) }
                    result_ptr_out := add(result_ptr, BOOLEAN_SIZE)
                }
                case 2 {
                    case_const(2, DATA_TYPE_TINYINT_VARIANT)
                    entry :=
                        add(MODULUS, signextend(INT8_SIZE_MINUS_ONE, shr(INT8_PADDING_BITS, calldataload(result_ptr))))
                    result_ptr_out := add(result_ptr, INT8_SIZE)
                }
                case 3 {
                    case_const(3, DATA_TYPE_SMALLINT_VARIANT)
                    entry :=
                        add(MODULUS, signextend(INT16_SIZE_MINUS_ONE, shr(INT16_PADDING_BITS, calldataload(result_ptr))))
                    result_ptr_out := add(result_ptr, INT16_SIZE)
                }
                case 4 {
                    case_const(4, DATA_TYPE_INT_VARIANT)
                    entry :=
                        add(MODULUS, signextend(INT32_SIZE_MINUS_ONE, shr(INT32_PADDING_BITS, calldataload(result_ptr))))
                    result_ptr_out := add(result_ptr, INT32_SIZE)
                }
                case 5 {
                    case_const(5, DATA_TYPE_BIGINT_VARIANT)
                    entry :=
                        add(MODULUS, signextend(INT64_SIZE_MINUS_ONE, shr(INT64_PADDING_BITS, calldataload(result_ptr))))
                    result_ptr_out := add(result_ptr, INT64_SIZE)
                }
                case 8 {
                    case_const(8, DATA_TYPE_DECIMAL75_VARIANT)
                    entry := calldataload(result_ptr)
                    result_ptr_out := add(result_ptr, WORD_SIZE)
                }
                case 9 {
                    case_const(9, DATA_TYPE_TIMESTAMP_VARIANT)
                    entry :=
                        add(MODULUS, signextend(INT64_SIZE_MINUS_ONE, shr(INT64_PADDING_BITS, calldataload(result_ptr))))
                    result_ptr_out := add(result_ptr, INT64_SIZE)
                }
                default { err(ERR_UNSUPPORTED_DATA_TYPE_VARIANT) }
                entry := mod(entry, MODULUS)
            }
            function exclude_coverage_stop_read_entry() {} // solhint-disable-line no-empty-blocks

            // IMPORTED-YUL ../base/DataType.pre.sol::read_data_type
            function exclude_coverage_start_read_data_type() {} // solhint-disable-line no-empty-blocks
            function read_data_type(ptr) -> ptr_out, data_type {
                data_type := shr(UINT32_PADDING_BITS, calldataload(ptr))
                ptr_out := add(ptr, UINT32_SIZE)
                switch data_type
                case 0 { case_const(0, DATA_TYPE_BOOLEAN_VARIANT) }
                case 2 { case_const(2, DATA_TYPE_TINYINT_VARIANT) }
                case 3 { case_const(3, DATA_TYPE_SMALLINT_VARIANT) }
                case 4 { case_const(4, DATA_TYPE_INT_VARIANT) }
                case 5 { case_const(5, DATA_TYPE_BIGINT_VARIANT) }
                case 8 {
                    case_const(8, DATA_TYPE_DECIMAL75_VARIANT)
                    ptr_out := add(ptr_out, UINT8_SIZE) // Skip precision
                    ptr_out := add(ptr_out, INT8_SIZE) // Skip scale
                }
                case 9 {
                    case_const(9, DATA_TYPE_TIMESTAMP_VARIANT)
                    ptr_out := add(ptr_out, UINT32_SIZE) // Skip timeunit
                    ptr_out := add(ptr_out, INT32_SIZE) // Skip timezone
                }
                default { err(ERR_UNSUPPORTED_DATA_TYPE_VARIANT) }
            }
            function exclude_coverage_stop_read_data_type() {} // solhint-disable-line no-empty-blocks

            // IMPORTED-YUL ResultVerifier.pre.sol::verify_result_evaluations
            function exclude_coverage_start_verify_result_evaluations() {} // solhint-disable-line no-empty-blocks
            function verify_result_evaluations(result_ptr, evaluation_point_ptr, evaluations_ptr) {
                let num_columns := shr(UINT64_PADDING_BITS, calldataload(result_ptr))
                result_ptr := add(result_ptr, UINT64_SIZE)
                if sub(num_columns, mload(evaluations_ptr)) { err(ERR_RESULT_COLUMN_COUNT_MISMATCH) }
                evaluations_ptr := add(evaluations_ptr, WORD_SIZE)

                let first := 1
                let table_len
                let eval_vec
                for {} num_columns { num_columns := sub(num_columns, 1) } {
                    let name_length := shr(UINT64_PADDING_BITS, calldataload(result_ptr))
                    result_ptr := add(result_ptr, add(UINT64_SIZE, name_length))
                    if byte(0, calldataload(result_ptr)) { err(ERR_INVALID_RESULT_COLUMN_NAME) }
                    result_ptr := add(result_ptr, 1)

                    let value := mload(evaluations_ptr)
                    evaluations_ptr := add(evaluations_ptr, WORD_SIZE)

                    let data_type_variant
                    result_ptr, data_type_variant := read_data_type(result_ptr)
                    let column_length := shr(UINT64_PADDING_BITS, calldataload(result_ptr))
                    result_ptr := add(result_ptr, UINT64_SIZE)

                    if first {
                        first := 0
                        table_len := column_length
                        eval_vec := compute_evaluation_vec(table_len, evaluation_point_ptr)
                    }
                    if sub(table_len, column_length) { err(ERR_INCONSISTENT_RESULT_COLUMN_LENGTHS) }

                    value := mulmod(MODULUS_MINUS_ONE, value, MODULUS)
                    for { let i := 0 } sub(table_len, i) { i := add(i, 1) } {
                        let entry
                        result_ptr, entry := read_entry(result_ptr, data_type_variant)
                        value := addmod(value, mulmod(entry, mload(add(eval_vec, mul(i, WORD_SIZE))), MODULUS), MODULUS)
                    }
                    if value { err(ERR_INCORRECT_RESULT) }
                }
            }
            function exclude_coverage_stop_verify_result_evaluations() {} // solhint-disable-line no-empty-blocks

            function make_transcript(result_ptr, plan_ptr, table_lengths_ptr, commitments_ptr) -> transcript_ptr {
                transcript_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(transcript_ptr, WORD_SIZE))
                mstore(transcript_ptr, INITIAL_TRANSCRIPT_STATE)

                append_calldata(transcript_ptr, plan_ptr, calldataload(sub(plan_ptr, WORD_SIZE)))
                append_calldata(transcript_ptr, result_ptr, calldataload(sub(result_ptr, WORD_SIZE)))
                append_array(transcript_ptr, table_lengths_ptr)

                let commitment_len := mload(commitments_ptr)
                mstore(commitments_ptr, mul(commitment_len, 2))
                append_array(transcript_ptr, commitments_ptr)
                mstore(commitments_ptr, commitment_len)

                mstore(mload(FREE_PTR), mload(transcript_ptr))
                mstore(add(mload(FREE_PTR), WORD_SIZE), 0)
                mstore(transcript_ptr, keccak256(mload(FREE_PTR), add(UINT64_SIZE, WORD_SIZE)))
            }

            function verify_proof(result_ptr, plan_ptr, proof_ptr, table_lengths_ptr, commitments_ptr) ->
                evaluation_point_ptr,
                evaluations_ptr
            {
                let transcript_ptr := make_transcript(result_ptr, plan_ptr, table_lengths_ptr, commitments_ptr)
                let builder_ptr := builder_new()
                builder_set_table_chi_evaluations(builder_ptr, table_lengths_ptr)

                let range_length
                {
                    let num_challenges
                    proof_ptr, range_length, num_challenges :=
                        read_first_round_message(proof_ptr, transcript_ptr, builder_ptr)

                    builder_set_challenges(builder_ptr, draw_challenges(transcript_ptr, num_challenges))
                }
                {
                    let num_constraints
                    proof_ptr, num_constraints := read_final_round_message(proof_ptr, transcript_ptr, builder_ptr)

                    builder_set_constraint_multipliers(builder_ptr, draw_challenges(transcript_ptr, num_constraints))
                }
                let num_vars := log2_up(range_length)
                let row_multipliers_challenges := draw_challenges(transcript_ptr, num_vars)

                proof_ptr, evaluation_point_ptr :=
                    read_and_verify_sumcheck_proof(proof_ptr, transcript_ptr, builder_ptr, num_vars)

                proof_ptr := read_pcs_evaluations(proof_ptr, transcript_ptr, builder_ptr)

                verify_pcs_evaluations(proof_ptr, commitments_ptr, transcript_ptr, builder_ptr, evaluation_point_ptr)

                compute_evaluations(evaluation_point_ptr, builder_get_table_chi_evaluations(builder_ptr))
                compute_evaluations(evaluation_point_ptr, builder_get_chi_evaluations(builder_ptr))

                builder_set_row_multipliers_evaluation(
                    builder_ptr,
                    compute_truncated_lagrange_basis_inner_product(
                        range_length,
                        add(row_multipliers_challenges, WORD_SIZE),
                        add(evaluation_point_ptr, WORD_SIZE),
                        num_vars
                    )
                )

                plan_ptr := skip_plan_names(plan_ptr)
                plan_ptr, evaluations_ptr := proof_plan_evaluate(plan_ptr, builder_ptr)
                builder_check_aggregate_evaluation(builder_ptr)
            }

            function verify_query(result_ptr, plan_ptr, proof_ptr, table_lengths_ptr, commitments_ptr) {
                let evaluation_point_ptr, evaluations_ptr :=
                    verify_proof(result_ptr, plan_ptr, proof_ptr, table_lengths_ptr, commitments_ptr)
                verify_result_evaluations(result_ptr, evaluation_point_ptr, evaluations_ptr)
            }

            // Revert if the commitments array has an odd length
            let commitments_len := mload(__commitments)
            if mod(commitments_len, 2) { err(ERR_COMMITMENT_ARRAY_ODD_LENGTH) }
            mstore(__commitments, div(commitments_len, 2))
            verify_query(__result.offset, __plan.offset, __proof.offset, __tableLengths, __commitments)
        }
    }
}
