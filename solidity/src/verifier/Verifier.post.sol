// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";

library Verifier {
    function __verify(
        bytes calldata __result,
        bytes calldata __plan,
        uint256[] memory __placeholderParameters,
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
            // IMPORTED-YUL ../base/Array.pre.sol::read_uint64_array_as_uint512_array
            function exclude_coverage_start_read_uint64_array_as_uint512_array() {} // solhint-disable-line no-empty-blocks
            function read_uint64_array_as_uint512_array(source_ptr) -> source_ptr_out, array_ptr {
                array_ptr := mload(FREE_PTR)

                let length := shr(UINT64_PADDING_BITS, calldataload(source_ptr))
                mstore(array_ptr, length)
                source_ptr := add(source_ptr, UINT64_SIZE)
                let target_ptr := add(array_ptr, WORD_SIZE)

                for {} length { length := sub(length, 1) } {
                    mstore(target_ptr, shr(UINT64_PADDING_BITS, calldataload(source_ptr)))
                    mstore(add(target_ptr, WORD_SIZE), 0)
                    source_ptr := add(source_ptr, UINT64_SIZE)
                    target_ptr := add(target_ptr, WORDX2_SIZE)
                }

                mstore(FREE_PTR, target_ptr)

                source_ptr_out := source_ptr
            }
            function exclude_coverage_stop_read_uint64_array_as_uint512_array() {} // solhint-disable-line no-empty-blocks
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
            // IMPORTED-YUL ../base/MathUtil.pre.sol::addmod_bn254
            function exclude_coverage_start_addmod_bn254() {} // solhint-disable-line no-empty-blocks
            function addmod_bn254(lhs, rhs) -> sum {
                sum := addmod(lhs, rhs, MODULUS)
            }
            function exclude_coverage_stop_addmod_bn254() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../base/MathUtil.pre.sol::submod_bn254
            function exclude_coverage_start_submod_bn254() {} // solhint-disable-line no-empty-blocks
            function submod_bn254(lhs, rhs) -> difference {
                difference := addmod(lhs, mulmod(rhs, MODULUS_MINUS_ONE, MODULUS), MODULUS)
            }
            function exclude_coverage_stop_submod_bn254() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../base/MathUtil.pre.sol::mulmod_bn254
            function exclude_coverage_start_mulmod_bn254() {} // solhint-disable-line no-empty-blocks
            function mulmod_bn254(lhs, rhs) -> product {
                product := mulmod(lhs, rhs, MODULUS)
            }
            function exclude_coverage_stop_mulmod_bn254() {} // solhint-disable-line no-empty-blocks
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
            // IMPORTED-YUL ../base/MathUtil.pre.sol::log2_up
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
            // IMPORTED-YUL ../base/Queue.pre.sol::dequeue_uint512
            function exclude_coverage_start_dequeue_uint512() {} // solhint-disable-line no-empty-blocks
            function dequeue_uint512(queue_ptr) -> upper, lower {
                let queue := mload(queue_ptr)
                let length := mload(queue)
                if iszero(length) { err(ERR_EMPTY_QUEUE) }
                queue := add(queue, WORD_SIZE)
                upper := mload(queue)
                queue := add(queue, WORD_SIZE)
                lower := mload(queue)
                mstore(queue, sub(length, 1))
                mstore(queue_ptr, queue)
            }
            function exclude_coverage_stop_dequeue_uint512() {} // solhint-disable-line no-empty-blocks
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
                let length
                length, value := dequeue_uint512(add(builder_ptr, BUILDER_CHI_EVALUATIONS_OFFSET))
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
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_get_singleton_chi_evaluation
            function exclude_coverage_start_builder_get_singleton_chi_evaluation() {} // solhint-disable-line no-empty-blocks
            function builder_get_singleton_chi_evaluation(builder_ptr) -> value {
                value := mload(add(builder_ptr, BUILDER_SINGLETON_CHI_EVALUATION_OFFSET))
            }
            function exclude_coverage_stop_builder_get_singleton_chi_evaluation() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_get_rho_evaluations
            function exclude_coverage_start_builder_get_rho_evaluations() {} // solhint-disable-line no-empty-blocks
            function builder_get_rho_evaluations(builder_ptr) -> values_ptr {
                values_ptr := mload(add(builder_ptr, BUILDER_RHO_EVALUATIONS_OFFSET))
            }
            function exclude_coverage_stop_builder_get_rho_evaluations() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_consume_rho_evaluation
            function exclude_coverage_start_builder_consume_rho_evaluation() {} // solhint-disable-line no-empty-blocks
            function builder_consume_rho_evaluation(builder_ptr) -> value {
                value := dequeue(add(builder_ptr, BUILDER_RHO_EVALUATIONS_OFFSET))
            }
            function exclude_coverage_stop_builder_consume_rho_evaluation() {} // solhint-disable-line no-empty-blocks
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
            // IMPORTED-YUL ../base/Array.pre.sol::get_uint512_array_element
            function exclude_coverage_start_get_uint512_array_element() {} // solhint-disable-line no-empty-blocks
            function get_uint512_array_element(arr_ptr, index) -> upper, lower {
                let arr := mload(arr_ptr)
                let length := mload(arr)
                if iszero(lt(index, length)) { err(ERR_INVALID_INDEX) }
                let element_ptr := add(add(arr, WORD_SIZE), mul(index, WORDX2_SIZE))
                upper := mload(element_ptr)
                lower := mload(add(element_ptr, WORD_SIZE))
            }
            function exclude_coverage_stop_get_uint512_array_element() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_get_table_chi_evaluation
            function exclude_coverage_start_builder_get_table_chi_evaluation() {} // solhint-disable-line no-empty-blocks
            function builder_get_table_chi_evaluation(builder_ptr, table_num) -> value {
                let length
                length, value :=
                    get_uint512_array_element(add(builder_ptr, BUILDER_TABLE_CHI_EVALUATIONS_OFFSET), table_num)
            }
            function exclude_coverage_stop_builder_get_table_chi_evaluation() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_get_table_chi_evaluations
            function exclude_coverage_start_builder_get_table_chi_evaluations() {} // solhint-disable-line no-empty-blocks
            function builder_get_table_chi_evaluations(builder_ptr) -> values_ptr {
                values_ptr := mload(add(builder_ptr, BUILDER_TABLE_CHI_EVALUATIONS_OFFSET))
            }
            function exclude_coverage_stop_builder_get_table_chi_evaluations() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_get_placeholder_parameter
            function exclude_coverage_start_builder_get_placeholder_parameter() {} // solhint-disable-line no-empty-blocks
            function builder_get_placeholder_parameter(builder_ptr, index) -> value {
                value := get_array_element(add(builder_ptr, BUILDER_PLACEHOLDER_PARAMETERS_OFFSET), index)
            }
            function exclude_coverage_stop_builder_get_placeholder_parameter() {} // solhint-disable-line no-empty-blocks
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
                mstore(add(builder_ptr, BUILDER_FINAL_ROUND_BIT_DISTRIBUTIONS_OFFSET), values_ptr)
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
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_set_singleton_chi_evaluation
            function exclude_coverage_start_builder_set_singleton_chi_evaluation() {} // solhint-disable-line no-empty-blocks
            function builder_set_singleton_chi_evaluation(builder_ptr, value) {
                mstore(add(builder_ptr, BUILDER_SINGLETON_CHI_EVALUATION_OFFSET), value)
            }
            function exclude_coverage_stop_builder_set_singleton_chi_evaluation() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_set_table_chi_evaluations
            function exclude_coverage_start_builder_set_table_chi_evaluations() {} // solhint-disable-line no-empty-blocks
            function builder_set_table_chi_evaluations(builder_ptr, values_ptr) {
                mstore(add(builder_ptr, BUILDER_TABLE_CHI_EVALUATIONS_OFFSET), values_ptr)
            }
            function exclude_coverage_stop_builder_set_table_chi_evaluations() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_set_placeholder_parameters
            function exclude_coverage_start_builder_set_placeholder_parameters() {} // solhint-disable-line no-empty-blocks
            function builder_set_placeholder_parameters(builder_ptr, values_ptr) {
                mstore(add(builder_ptr, BUILDER_PLACEHOLDER_PARAMETERS_OFFSET), values_ptr)
            }
            function exclude_coverage_stop_builder_set_placeholder_parameters() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../builder/VerificationBuilder.pre.sol::builder_consume_bit_distribution
            function exclude_coverage_start_builder_consume_bit_distribution() {} // solhint-disable-line no-empty-blocks
            function builder_consume_bit_distribution(builder_ptr) -> vary_mask, leading_bit_mask {
                let values_ptr := add(builder_ptr, BUILDER_FINAL_ROUND_BIT_DISTRIBUTIONS_OFFSET)
                vary_mask, leading_bit_mask := dequeue_uint512(values_ptr)
            }
            function exclude_coverage_stop_builder_consume_bit_distribution() {} // solhint-disable-line no-empty-blocks
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
                    tmp := mulmod_bn254(tmp, d)
                    // tmp += v1i
                    v_stack := sub(v_stack, WORD_SIZE)
                    tmp := addmod_bn254(tmp, calldataload(v_stack))
                    // tmp *= d
                    tmp := mulmod_bn254(tmp, d)
                    // tmp += v0i
                    v_stack := sub(v_stack, WORD_SIZE)
                    tmp := addmod_bn254(tmp, calldataload(v_stack))

                    // b *= q
                    b := mulmod_bn254(b, q)
                    // b += tmp
                    b := addmod_bn254(b, tmp)
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
                    if addmod_bn254(
                        mulmod_bn254(
                            r,
                            addmod_bn254(
                                addmod_bn254(last_v2, last_v2),
                                mulmod_bn254(submod_bn254(xi, 1), addmod_bn254(v1i, v0i))
                            )
                        ),
                        mulmod_bn254(xi, submod_bn254(v1i, v0i))
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
                ec_mul_assign(scratch, addmod_bn254(mulmod_bn254(d, addmod_bn254(d, 1)), 1))
                // g_l += -G * b
                constant_ec_mul_add_assign(scratch, G1_NEG_GEN_X, G1_NEG_GEN_Y, b)

                let dr := mulmod_bn254(d, r)
                // g_l += w[0] * r
                calldata_ec_mul_add_assign(scratch, w_ptr, r)
                // g_l += w[1] * -d * r
                calldata_ec_mul_add_assign(scratch, add(w_ptr, WORDX2_SIZE), sub(MODULUS, dr))
                // g_l += w[2] * (d * r)^2
                calldata_ec_mul_add_assign(scratch, add(w_ptr, WORDX4_SIZE), mulmod_bn254(dr, dr))
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
            // IMPORTED-YUL ../proof_gadgets/Shift.pre.sol::compute_shift_identity_constraint
            function exclude_coverage_start_compute_shift_identity_constraint() {} // solhint-disable-line no-empty-blocks
            function compute_shift_identity_constraint(star, chi_plus_one, fold) -> constraint {
                constraint := addmod_bn254(submod_bn254(star, chi_plus_one), mulmod_bn254(fold, star))
            }
            function exclude_coverage_stop_compute_shift_identity_constraint() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_gadgets/Shift.pre.sol::compute_shift_fold
            function exclude_coverage_start_compute_shift_fold() {} // solhint-disable-line no-empty-blocks
            function compute_shift_fold(alpha, beta, eval, rho) -> fold {
                fold := mulmod_bn254(alpha, addmod_bn254(mulmod_bn254(beta, rho), eval))
            }
            function exclude_coverage_stop_compute_shift_fold() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_gadgets/Shift.pre.sol::shift_evaluate
            function exclude_coverage_start_shift_evaluate() {} // solhint-disable-line no-empty-blocks
            function shift_evaluate(builder_ptr, alpha, beta, expr_eval, shifted_expr_eval, chi_eval, chi_plus_one_eval)
            {
                let rho_eval := builder_consume_rho_evaluation(builder_ptr)
                let rho_plus_one_eval := builder_consume_rho_evaluation(builder_ptr)
                let c_star_eval := builder_consume_final_round_mle(builder_ptr)
                let d_star_eval := builder_consume_final_round_mle(builder_ptr)
                // sum c_star - d_star = 0
                builder_produce_zerosum_constraint(builder_ptr, submod_bn254(c_star_eval, d_star_eval), 1)
                // c_star + c_fold * c_star - chi_n_plus_1 = 0
                {
                    let c_fold := compute_shift_fold(alpha, beta, expr_eval, addmod_bn254(rho_eval, chi_eval))
                    builder_produce_identity_constraint(
                        builder_ptr, compute_shift_identity_constraint(c_star_eval, chi_plus_one_eval, c_fold), 2
                    )
                }
                // d_star + d_fold * d_star - chi_n_plus_1 = 0
                {
                    let d_fold := compute_shift_fold(alpha, beta, shifted_expr_eval, rho_plus_one_eval)
                    builder_produce_identity_constraint(
                        builder_ptr, compute_shift_identity_constraint(d_star_eval, chi_plus_one_eval, d_fold), 2
                    )
                }
            }
            function exclude_coverage_stop_shift_evaluate() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_gadgets/Monotonic.pre.sol::monotonic_verify
            function exclude_coverage_start_monotonic_verify() {} // solhint-disable-line no-empty-blocks
            function monotonic_verify(builder_ptr, alpha, beta, column_eval, chi_eval, strict, asc) {
                // 1. Verify that `shifted_column` is a shift of `column`
                let shifted_column_eval := builder_consume_final_round_mle(builder_ptr)
                let shifted_chi_eval := builder_consume_chi_evaluation(builder_ptr)

                shift_evaluate(builder_ptr, alpha, beta, column_eval, shifted_column_eval, chi_eval, shifted_chi_eval)

                // 2. Compute indicator evaluation based on strictness and direction
                let ind_eval
                switch eq(strict, asc)
                case 1 {
                    // (strict && asc) || (!strict && !asc): ind = shifted_column - column
                    ind_eval := submod_bn254(shifted_column_eval, column_eval)
                }
                default {
                    // (!strict && asc) || (strict && !asc): ind = column - shifted_column
                    ind_eval := submod_bn254(column_eval, shifted_column_eval)
                }

                // 3. Verify the sign of `ind`
                let sign_eval := sign_expr_evaluate(ind_eval, builder_ptr, shifted_chi_eval)
                let singleton_chi_eval := builder_get_singleton_chi_evaluation(builder_ptr)

                // 4. Check if sign_eval is in allowed evaluations
                let is_valid := 0
                switch strict
                case 1 {
                    // Strict monotonicity: sign(ind) == 1 for all but first and last element
                    // Allowed evaluations: chi_eval, shifted_chi_eval - singleton_chi_eval, chi_eval - singleton_chi_eval
                    is_valid :=
                        or(
                            or(eq(sign_eval, chi_eval), eq(sign_eval, submod_bn254(shifted_chi_eval, singleton_chi_eval))),
                            eq(sign_eval, submod_bn254(chi_eval, singleton_chi_eval))
                        )
                }
                default {
                    // Non-strict monotonicity: sign(ind) == 0 for all but first and last element
                    // Allowed evaluations: singleton_chi_eval, shifted_chi_eval - chi_eval,
                    // singleton_chi_eval + shifted_chi_eval - chi_eval, 0
                    is_valid :=
                        or(
                            or(eq(sign_eval, singleton_chi_eval), eq(sign_eval, submod_bn254(shifted_chi_eval, chi_eval))),
                            or(
                                eq(sign_eval, submod_bn254(addmod_bn254(singleton_chi_eval, shifted_chi_eval), chi_eval)),
                                iszero(sign_eval)
                            )
                        )
                }

                if iszero(is_valid) { err(ERR_MONOTONY_CHECK_FAILED) }
            }
            function exclude_coverage_stop_monotonic_verify() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_gadgets/SignExpr.pre.sol::sign_expr_evaluate
            function exclude_coverage_start_sign_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            function sign_expr_evaluate(expr_eval, builder_ptr, chi_eval) -> result_eval {
                let vary_mask
                let leading_bit_mask
                vary_mask, leading_bit_mask := builder_consume_bit_distribution(builder_ptr)

                // Other than the lead bit, no bit should vary past some max bit position, depending on the field
                if and(vary_mask, MODULUS_INVALID_VARY_MASK) { err(ERR_INVALID_VARYING_BITS) }

                // The lead bit of the leading_bit_mask dictates the sign, if it's constant sign.
                // So this will be the value if sign is constant. Otherwise, it will be overwritten
                let sign_eval := mulmod_bn254(shr(255, leading_bit_mask), chi_eval)

                // For future computations, leading_bit_mask should have a 1 in the lead bit
                leading_bit_mask := or(leading_bit_mask, shl(255, 1))

                // leading_bit_inverse_mask identifies columns that match the inverse of the lead bit column
                // So !vary_mask ^ leading_bit_mask, with a lead bit of zero.
                let leading_bit_inverse_mask := shr(1, shl(1, xor(not(vary_mask), leading_bit_mask)))

                // sum_eval should ultimately add up to the original column of data
                // It will effectively be a recomposition of the bit decomposition
                let sum_eval := 0

                for { let i := 0 } vary_mask {
                    i := add(i, 1)
                    vary_mask := shr(1, vary_mask)
                } {
                    if and(vary_mask, 1) {
                        // For any varying bits...
                        let bit_eval := builder_consume_final_round_mle(builder_ptr)

                        // Verify that every eval is a bit
                        // bit_eval - bit_eval * bit_eval = 0
                        builder_produce_identity_constraint(
                            builder_ptr, submod_bn254(bit_eval, mulmod_bn254(bit_eval, bit_eval)), 2
                        )

                        switch i
                        // If the lead bit varies, that we get the sign from the mles.
                        case 255 { sign_eval := bit_eval }
                        // For varying non lead bits,
                        // we add bit_eval * 2ⁱ to the sum in order to recompose the original value of the column
                        default { sum_eval := addmod_bn254(sum_eval, mulmod_bn254(bit_eval, shl(i, 1))) }
                    }
                }

                result_eval := submod_bn254(chi_eval, sign_eval)

                // For constant and lead bits...
                // sum += sign_eval * leading_bit_mask + (sign_eval - chi_eval) * leading_bit_inverse_mask - chi_eval * (1 << 255)
                sum_eval :=
                    submod_bn254(
                        addmod_bn254(
                            addmod_bn254(sum_eval, mulmod_bn254(sign_eval, leading_bit_mask)),
                            mulmod_bn254(result_eval, leading_bit_inverse_mask)
                        ),
                        mulmod_bn254(chi_eval, shl(255, 1))
                    )

                // Verify the bit recomposition matches the original column evaluation
                if sub(sum_eval, expr_eval) { err(ERR_BIT_DECOMPOSITION_INVALID) }
            }
            function exclude_coverage_stop_sign_expr_evaluate() {} // solhint-disable-line no-empty-blocks
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

                let diff_eval := submod_bn254(lhs_eval, rhs_eval)
                let diff_star_eval := builder_consume_final_round_mle(builder_ptr)
                result_eval := mod(builder_consume_final_round_mle(builder_ptr), MODULUS)

                builder_produce_identity_constraint(builder_ptr, mulmod_bn254(result_eval, diff_eval), 2)
                builder_produce_identity_constraint(
                    builder_ptr,
                    submod_bn254(chi_eval, addmod_bn254(mulmod_bn254(diff_eval, diff_star_eval), result_eval)),
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
                eval := mulmod_bn254(eval, chi_eval)
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

                result_eval := addmod_bn254(lhs_eval, rhs_eval)
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

                result_eval := submod_bn254(lhs_eval, rhs_eval)
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
                    builder_ptr, submod_bn254(result_eval, mulmod_bn254(lhs_eval, rhs_eval)), 2
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
                    builder_ptr, submod_bn254(result_eval, mulmod_bn254(lhs_eval, rhs_eval)), 2
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
                result_eval := submod_bn254(addmod_bn254(lhs_eval, rhs_eval), lhs_times_rhs_eval)
                builder_produce_identity_constraint(
                    builder_ptr, submod_bn254(lhs_times_rhs_eval, mulmod_bn254(lhs_eval, rhs_eval)), 2
                )

                expr_ptr_out := expr_ptr
            }
            function exclude_coverage_stop_or_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_exprs/NotExpr.pre.sol::not_expr_evaluate
            function exclude_coverage_start_not_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            function not_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                let input_eval
                expr_ptr, input_eval := proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval)

                result_eval := submod_bn254(chi_eval, input_eval)
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
            // IMPORTED-YUL ../proof_exprs/InequalityExpr.pre.sol::inequality_expr_evaluate
            function exclude_coverage_start_inequality_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            function inequality_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                let lhs_eval
                expr_ptr, lhs_eval := proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval)

                let rhs_eval
                expr_ptr, rhs_eval := proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval)

                let is_lt := shr(BOOLEAN_PADDING_BITS, calldataload(expr_ptr))
                expr_ptr := add(expr_ptr, BOOLEAN_SIZE)

                let diff_eval
                switch is_lt
                case 0 { diff_eval := submod_bn254(rhs_eval, lhs_eval) }
                default { diff_eval := submod_bn254(lhs_eval, rhs_eval) }
                result_eval := sign_expr_evaluate(diff_eval, builder_ptr, chi_eval)
                expr_ptr_out := expr_ptr
            }
            function exclude_coverage_stop_inequality_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_plans/GroupByExec.pre.sol::compute_g_in_star_eval
            function exclude_coverage_start_compute_g_in_star_eval() {} // solhint-disable-line no-empty-blocks
            function compute_g_in_star_eval(plan_ptr, builder_ptr, alpha, beta, input_chi_eval) ->
                plan_ptr_out,
                g_in_star_eval_times_selection_eval,
                num_group_by_columns
            {
                num_group_by_columns := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                // We can not prove uniqueness for multiple columns yet
                if sub(num_group_by_columns, 1) { err(ERR_UNPROVABLE_GROUP_BY) }
                plan_ptr := add(plan_ptr, UINT64_SIZE)

                // Process group by columns
                let g_in_fold
                plan_ptr, g_in_fold := fold_column_expr_evals(plan_ptr, builder_ptr, beta, num_group_by_columns)
                g_in_fold := mulmod_bn254(g_in_fold, alpha)

                // Get the g_in_star and g_out_star evaluations
                let g_in_star_eval := builder_consume_final_round_mle(builder_ptr)
                let selection_eval
                plan_ptr, selection_eval := proof_expr_evaluate(plan_ptr, builder_ptr, input_chi_eval)

                // First constraint: g_in_star + g_in_star * g_in_fold - input_chi_eval = 0
                builder_produce_identity_constraint(
                    builder_ptr,
                    submod_bn254(addmod_bn254(g_in_star_eval, mulmod_bn254(g_in_star_eval, g_in_fold)), input_chi_eval),
                    2
                )
                g_in_star_eval_times_selection_eval := mulmod_bn254(g_in_star_eval, selection_eval)
                plan_ptr_out := plan_ptr
            }
            function exclude_coverage_stop_compute_g_in_star_eval() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_plans/GroupByExec.pre.sol::compute_sum_in_fold_eval
            function exclude_coverage_start_compute_sum_in_fold_eval() {} // solhint-disable-line no-empty-blocks
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
            function exclude_coverage_stop_compute_sum_in_fold_eval() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_plans/GroupByExec.pre.sol::compute_g_out_star_eval
            function exclude_coverage_start_compute_g_out_star_eval() {} // solhint-disable-line no-empty-blocks
            function compute_g_out_star_eval(
                builder_ptr, alpha, beta, output_chi_eval, num_group_by_columns, evaluations_ptr
            ) -> g_out_star_eval {
                let g_out_fold := 0
                for {} num_group_by_columns { num_group_by_columns := sub(num_group_by_columns, 1) } {
                    let mle := builder_consume_final_round_mle(builder_ptr)
                    g_out_fold := addmod_bn254(mulmod_bn254(g_out_fold, beta), mle)
                    mstore(evaluations_ptr, mle)
                    evaluations_ptr := add(evaluations_ptr, WORD_SIZE)
                }
                // Uniqueness constraint, currently only for single group by column using monotonicity
                monotonic_verify(builder_ptr, alpha, beta, g_out_fold, output_chi_eval, 1, 1)
                g_out_fold := mulmod_bn254(g_out_fold, alpha)
                g_out_star_eval := builder_consume_final_round_mle(builder_ptr)
                // Second constraint: g_out_star + g_out_star * g_out_fold - output_chi_eval = 0
                builder_produce_identity_constraint(
                    builder_ptr,
                    submod_bn254(
                        addmod_bn254(g_out_star_eval, mulmod_bn254(g_out_star_eval, g_out_fold)), output_chi_eval
                    ),
                    2
                )
            }
            function exclude_coverage_stop_compute_g_out_star_eval() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_plans/GroupByExec.pre.sol::compute_sum_out_fold_eval
            function exclude_coverage_start_compute_sum_out_fold_eval() {} // solhint-disable-line no-empty-blocks
            function compute_sum_out_fold_eval(
                builder_ptr, alpha, beta, output_chi_eval, num_sum_columns, evaluations_ptr
            ) -> sum_out_fold_eval {
                sum_out_fold_eval := 0
                for {} num_sum_columns { num_sum_columns := sub(num_sum_columns, 1) } {
                    let mle := builder_consume_final_round_mle(builder_ptr)
                    sum_out_fold_eval := addmod_bn254(mulmod_bn254(sum_out_fold_eval, beta), mle)
                    mstore(evaluations_ptr, mle)
                    evaluations_ptr := add(evaluations_ptr, WORD_SIZE)
                }
                // Consume count column evaluation
                let count_out_eval := builder_consume_final_round_mle(builder_ptr)
                mstore(evaluations_ptr, count_out_eval)
                sum_out_fold_eval := addmod_bn254(mulmod_bn254(sum_out_fold_eval, beta), count_out_eval)
            }
            function exclude_coverage_stop_compute_sum_out_fold_eval() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_plans/GroupByExec.pre.sol::read_input_evals
            function exclude_coverage_start_read_input_evals() {} // solhint-disable-line no-empty-blocks
            function read_input_evals(plan_ptr, builder_ptr, alpha, beta) ->
                plan_ptr_out,
                partial_dlog_zero_sum_constraint_eval,
                num_group_by_columns,
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
                plan_ptr, g_in_star_eval_times_selection_eval, num_group_by_columns :=
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
            function exclude_coverage_stop_read_input_evals() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_plans/GroupByExec.pre.sol::read_output_evals
            function exclude_coverage_start_read_output_evals() {} // solhint-disable-line no-empty-blocks
            function read_output_evals(
                builder_ptr, alpha, beta, partial_dlog_zero_sum_constraint_eval, num_group_by_columns, num_sum_columns
            ) -> evaluations_ptr, output_chi_eval {
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

                output_chi_eval := builder_consume_chi_evaluation(builder_ptr)

                let g_out_star_eval :=
                    compute_g_out_star_eval(
                        builder_ptr, alpha, beta, output_chi_eval, num_group_by_columns, add(evaluations_ptr, WORD_SIZE)
                    )

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
            function exclude_coverage_stop_read_output_evals() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_exprs/PlaceholderExpr.pre.sol::placeholder_expr_evaluate
            function exclude_coverage_start_placeholder_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            function placeholder_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                let placeholder_index := shr(UINT64_PADDING_BITS, calldataload(expr_ptr))
                expr_ptr := add(expr_ptr, UINT64_SIZE)

                // Read column type using read_data_type
                let column_type
                expr_ptr, column_type := read_data_type(expr_ptr)

                // Get the placeholder parameter value from the builder
                let parameter_value := builder_get_placeholder_parameter(builder_ptr, placeholder_index)

                // Multiply by chi_eval (similar to how literals work)
                eval := mulmod_bn254(parameter_value, chi_eval)

                expr_ptr_out := expr_ptr
            }
            function exclude_coverage_stop_placeholder_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            // slither-disable-start cyclomatic-complexity
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
                case 10 {
                    case_const(10, INEQUALITY_EXPR_VARIANT)
                    expr_ptr_out, eval := inequality_expr_evaluate(expr_ptr, builder_ptr, chi_eval)
                }
                case 11 {
                    case_const(11, PLACEHOLDER_EXPR_VARIANT)
                    expr_ptr_out, eval := placeholder_expr_evaluate(expr_ptr, builder_ptr, chi_eval)
                }
                default { err(ERR_UNSUPPORTED_PROOF_EXPR_VARIANT) }
            }
            function exclude_coverage_stop_proof_expr_evaluate() {} // solhint-disable-line no-empty-blocks
            // slither-disable-end cyclomatic-complexity
            // IMPORTED-YUL ../proof_gadgets/FoldUtil.pre.sol::fold_expr_evals
            function exclude_coverage_start_fold_expr_evals() {} // solhint-disable-line no-empty-blocks
            function fold_expr_evals(plan_ptr, builder_ptr, input_chi_eval, beta, column_count) -> plan_ptr_out, fold {
                fold := 0
                for {} column_count { column_count := sub(column_count, 1) } {
                    let expr_eval
                    plan_ptr, expr_eval := proof_expr_evaluate(plan_ptr, builder_ptr, input_chi_eval)
                    fold := addmod_bn254(mulmod_bn254(fold, beta), expr_eval)
                }
                plan_ptr_out := plan_ptr
            }
            function exclude_coverage_stop_fold_expr_evals() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_gadgets/FoldUtil.pre.sol::fold_column_expr_evals
            function exclude_coverage_start_fold_column_expr_evals() {} // solhint-disable-line no-empty-blocks
            function fold_column_expr_evals(plan_ptr, builder_ptr, beta, column_count) -> plan_ptr_out, fold {
                fold := 0
                for {} column_count { column_count := sub(column_count, 1) } {
                    let expr_eval
                    plan_ptr, expr_eval := column_expr_evaluate(plan_ptr, builder_ptr)
                    fold := addmod_bn254(mulmod_bn254(fold, beta), expr_eval)
                }
                plan_ptr_out := plan_ptr
            }
            function exclude_coverage_stop_fold_column_expr_evals() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_gadgets/FoldUtil.pre.sol::fold_final_round_mles
            function exclude_coverage_start_fold_final_round_mles() {} // solhint-disable-line no-empty-blocks
            function fold_final_round_mles(builder_ptr, beta, column_count) -> fold, evaluations_ptr {
                evaluations_ptr := mload(FREE_PTR)
                mstore(evaluations_ptr, column_count)
                evaluations_ptr := add(evaluations_ptr, WORD_SIZE)
                fold := 0
                for { let i := column_count } i { i := sub(i, 1) } {
                    let mle := builder_consume_final_round_mle(builder_ptr)
                    fold := addmod_bn254(mulmod_bn254(fold, beta), mle)
                    mstore(evaluations_ptr, mle)
                    evaluations_ptr := add(evaluations_ptr, WORD_SIZE)
                }
                evaluations_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(evaluations_ptr, add(WORD_SIZE, mul(column_count, WORD_SIZE))))
            }
            function exclude_coverage_stop_fold_final_round_mles() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_plans/FilterExec.pre.sol::compute_filter_folds
            function exclude_coverage_start_compute_filter_folds() {} // solhint-disable-line no-empty-blocks
            function compute_filter_folds(plan_ptr, builder_ptr, input_chi_eval, beta) ->
                plan_ptr_out,
                c_fold,
                d_fold,
                evaluations_ptr
            {
                let column_count := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                plan_ptr := add(plan_ptr, UINT64_SIZE)

                plan_ptr, c_fold := fold_expr_evals(plan_ptr, builder_ptr, input_chi_eval, beta, column_count)
                d_fold, evaluations_ptr := fold_final_round_mles(builder_ptr, beta, column_count)
                plan_ptr_out := plan_ptr
            }
            function exclude_coverage_stop_compute_filter_folds() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_plans/EmptyExec.pre.sol::empty_exec_evaluate
            function exclude_coverage_start_empty_exec_evaluate() {} // solhint-disable-line no-empty-blocks
            function empty_exec_evaluate(builder_ptr) -> evaluations_ptr, output_chi_eval {
                evaluations_ptr := mload(FREE_PTR)
                mstore(evaluations_ptr, 0)
                mstore(FREE_PTR, add(evaluations_ptr, WORD_SIZE))
                output_chi_eval := builder_get_singleton_chi_evaluation(builder_ptr)
            }
            function exclude_coverage_stop_empty_exec_evaluate() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_plans/FilterExec.pre.sol::verify_filter
            function exclude_coverage_start_verify_filter() {} // solhint-disable-line no-empty-blocks
            function verify_filter(builder_ptr, c_fold, d_fold, input_chi_eval, output_chi_eval, selection_eval) {
                let c_star := builder_consume_final_round_mle(builder_ptr)
                let d_star := builder_consume_final_round_mle(builder_ptr)

                builder_produce_zerosum_constraint(
                    builder_ptr, submod_bn254(mulmod_bn254(c_star, selection_eval), d_star), 2
                )
                builder_produce_identity_constraint(
                    builder_ptr, submod_bn254(mulmod_bn254(addmod_bn254(1, c_fold), c_star), input_chi_eval), 2
                )
                builder_produce_identity_constraint(
                    builder_ptr, submod_bn254(mulmod_bn254(addmod_bn254(1, d_fold), d_star), output_chi_eval), 2
                )
                builder_produce_identity_constraint(
                    builder_ptr, mulmod_bn254(d_fold, submod_bn254(output_chi_eval, 1)), 2
                )
            }
            function exclude_coverage_stop_verify_filter() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_plans/FilterExec.pre.sol::filter_exec_evaluate
            function exclude_coverage_start_filter_exec_evaluate() {} // solhint-disable-line no-empty-blocks
            function filter_exec_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr, output_chi_eval {
                let alpha := builder_consume_challenge(builder_ptr)
                let input_chi_eval, selection_eval, c_fold, d_fold
                {
                    let beta := builder_consume_challenge(builder_ptr)

                    input_chi_eval :=
                        builder_get_table_chi_evaluation(builder_ptr, shr(UINT64_PADDING_BITS, calldataload(plan_ptr)))
                    plan_ptr := add(plan_ptr, UINT64_SIZE)

                    plan_ptr, selection_eval := proof_expr_evaluate(plan_ptr, builder_ptr, input_chi_eval)

                    plan_ptr, c_fold, d_fold, evaluations_ptr :=
                        compute_filter_folds(plan_ptr, builder_ptr, input_chi_eval, beta)
                }
                output_chi_eval := builder_consume_chi_evaluation(builder_ptr)

                verify_filter(
                    builder_ptr,
                    mulmod_bn254(alpha, c_fold),
                    mulmod_bn254(alpha, d_fold),
                    input_chi_eval,
                    output_chi_eval,
                    selection_eval
                )

                plan_ptr_out := plan_ptr
            }
            function exclude_coverage_stop_filter_exec_evaluate() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_plans/TableExec.pre.sol::table_exec_evaluate
            function exclude_coverage_start_table_exec_evaluate() {} // solhint-disable-line no-empty-blocks
            function table_exec_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr, output_chi_eval {
                let table_number := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                plan_ptr := add(plan_ptr, UINT64_SIZE)
                output_chi_eval := builder_get_table_chi_evaluation(builder_ptr, table_number)

                // Get the number of columns in the schema
                let column_count := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                let copy_size := add(WORD_SIZE, mul(column_count, WORD_SIZE))
                plan_ptr := add(plan_ptr, UINT64_SIZE)

                // Initialize evaluations array to store column evaluations
                evaluations_ptr := mload(FREE_PTR)
                mstore(evaluations_ptr, column_count)

                // Read column evaluations for each field in the schema
                for {} column_count { column_count := sub(column_count, 1) } {
                    evaluations_ptr := add(evaluations_ptr, WORD_SIZE)
                    // For each column in schema, get its column number/index
                    let column_num := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                    plan_ptr := add(plan_ptr, UINT64_SIZE)

                    // Get the column evaluation from the builder
                    let column_eval := builder_get_column_evaluation(builder_ptr, column_num)

                    // Store the column evaluation in the result
                    mstore(evaluations_ptr, column_eval)
                }

                // Reset evaluations_ptr to the beginning of the array
                evaluations_ptr := mload(FREE_PTR)
                // Update free memory pointer
                mstore(FREE_PTR, add(evaluations_ptr, copy_size))

                plan_ptr_out := plan_ptr
            }
            function exclude_coverage_stop_table_exec_evaluate() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_plans/ProjectionExec.pre.sol::projection_exec_evaluate
            function exclude_coverage_start_projection_exec_evaluate() {} // solhint-disable-line no-empty-blocks
            function projection_exec_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr, output_chi_eval {
                let input_evaluations_ptr
                plan_ptr, input_evaluations_ptr, output_chi_eval := proof_plan_evaluate(plan_ptr, builder_ptr)

                let column_count := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                plan_ptr := add(plan_ptr, UINT64_SIZE)

                evaluations_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(evaluations_ptr, add(WORD_SIZE, mul(column_count, WORD_SIZE))))
                let target_ptr := evaluations_ptr
                mstore(target_ptr, column_count)

                for {} column_count { column_count := sub(column_count, 1) } {
                    target_ptr := add(target_ptr, WORD_SIZE)
                    let evaluation
                    plan_ptr, evaluation := proof_expr_evaluate(plan_ptr, builder_ptr, output_chi_eval)

                    mstore(target_ptr, evaluation)
                }
                plan_ptr_out := plan_ptr
            }
            function exclude_coverage_stop_projection_exec_evaluate() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../proof_plans/GroupByExec.pre.sol::group_by_exec_evaluate
            function exclude_coverage_start_group_by_exec_evaluate() {} // solhint-disable-line no-empty-blocks
            function group_by_exec_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr, output_chi_eval {
                let alpha := builder_consume_challenge(builder_ptr)
                let beta := builder_consume_challenge(builder_ptr)

                let partial_dlog_zero_sum_constraint_eval, num_group_by_columns, num_sum_columns
                plan_ptr_out, partial_dlog_zero_sum_constraint_eval, num_group_by_columns, num_sum_columns :=
                    read_input_evals(plan_ptr, builder_ptr, alpha, beta)

                // Read output
                evaluations_ptr, output_chi_eval :=
                    read_output_evals(
                        builder_ptr,
                        alpha,
                        beta,
                        partial_dlog_zero_sum_constraint_eval,
                        num_group_by_columns,
                        num_sum_columns
                    )
            }
            function exclude_coverage_stop_group_by_exec_evaluate() {} // solhint-disable-line no-empty-blocks
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
                    round_evaluation := mulmod_bn254(round_evaluation, challenge)
                    round_evaluation := addmod_bn254(round_evaluation, coefficient)
                    actual_sum := addmod_bn254(actual_sum, coefficient)
                }
                actual_sum := addmod_bn254(actual_sum, coefficient)
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
            function proof_plan_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr, output_chi_eval {
                let proof_plan_variant := shr(UINT32_PADDING_BITS, calldataload(plan_ptr))
                plan_ptr := add(plan_ptr, UINT32_SIZE)

                switch proof_plan_variant
                case 0 {
                    case_const(0, FILTER_EXEC_VARIANT)
                    plan_ptr_out, evaluations_ptr, output_chi_eval := filter_exec_evaluate(plan_ptr, builder_ptr)
                }
                case 1 {
                    case_const(1, EMPTY_EXEC_VARIANT)
                    evaluations_ptr, output_chi_eval := empty_exec_evaluate(builder_ptr)
                    plan_ptr_out := plan_ptr
                }
                case 2 {
                    case_const(2, TABLE_EXEC_VARIANT)
                    plan_ptr_out, evaluations_ptr, output_chi_eval := table_exec_evaluate(plan_ptr, builder_ptr)
                }
                case 3 {
                    case_const(3, PROJECTION_EXEC_VARIANT)
                    plan_ptr_out, evaluations_ptr, output_chi_eval := projection_exec_evaluate(plan_ptr, builder_ptr)
                }
                case 5 {
                    case_const(5, GROUP_BY_EXEC_VARIANT)
                    plan_ptr_out, evaluations_ptr, output_chi_eval := group_by_exec_evaluate(plan_ptr, builder_ptr)
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

                proof_ptr, array_ptr := read_uint64_array_as_uint512_array(proof_ptr)
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
                builder_set_aggregate_evaluation(builder_ptr, mulmod_bn254(MODULUS_MINUS_ONE, expected_evaluation))
                builder_set_max_degree(builder_ptr, sumcheck_degree)
            }
            // IMPORTED-YUL ../base/LagrangeBasisEvaluation.pre.sol::compute_chi_evaluations
            function exclude_coverage_start_compute_chi_evaluations() {} // solhint-disable-line no-empty-blocks
            function compute_chi_evaluations(evaluation_point_ptr, array_ptr) {
                let num_vars := mload(evaluation_point_ptr)
                let x := add(evaluation_point_ptr, WORD_SIZE)
                let array_len := mload(array_ptr)
                array_ptr := add(array_ptr, WORD_SIZE)
                for {} array_len { array_len := sub(array_len, 1) } {
                    mstore(array_ptr, compute_truncated_lagrange_basis_sum(mload(array_ptr), x, num_vars))
                    array_ptr := add(array_ptr, WORD_SIZE)
                }
            }
            function exclude_coverage_stop_compute_chi_evaluations() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../base/LagrangeBasisEvaluation.pre.sol::compute_rho_evaluations
            function exclude_coverage_start_compute_rho_evaluations() {} // solhint-disable-line no-empty-blocks
            function compute_rho_evaluations(evaluation_point_ptr, array_ptr) {
                let array_len := mload(array_ptr)
                for {} array_len { array_len := sub(array_len, 1) } {
                    array_ptr := add(array_ptr, WORD_SIZE)
                    let length := mload(array_ptr)
                    let evaluation_vec := compute_evaluation_vec(length, evaluation_point_ptr)
                    let product := 0
                    for {} length {} {
                        let i := sub(length, 1)
                        product := addmod_bn254(product, mulmod_bn254(i, mload(add(evaluation_vec, mul(i, WORD_SIZE)))))
                        length := i
                    }
                    mstore(array_ptr, product)
                }
            }
            function exclude_coverage_stop_compute_rho_evaluations() {} // solhint-disable-line no-empty-blocks
            // IMPORTED-YUL ../base/LagrangeBasisEvaluation.pre.sol::compute_evaluations_with_length
            function exclude_coverage_start_compute_evaluations_with_length() {} // solhint-disable-line no-empty-blocks
            function compute_evaluations_with_length(evaluation_point_ptr, array_ptr) {
                let num_vars := mload(evaluation_point_ptr)
                let x := add(evaluation_point_ptr, WORD_SIZE)
                let array_len := mload(array_ptr)
                array_ptr := add(array_ptr, WORD_SIZE)
                for {} array_len { array_len := sub(array_len, 1) } {
                    mstore(
                        add(array_ptr, WORD_SIZE), compute_truncated_lagrange_basis_sum(mload(array_ptr), x, num_vars)
                    )
                    array_ptr := add(array_ptr, WORDX2_SIZE)
                }
            }
            function exclude_coverage_stop_compute_evaluations_with_length() {} // solhint-disable-line no-empty-blocks
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
                    batch_eval := addmod_bn254(batch_eval, mulmod_bn254(mload(evaluations_ptr), challenge))
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

            // slither-disable-start cyclomatic-complexity
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
                    entry := mod(entry, MODULUS)
                }
                case 2 {
                    case_const(2, DATA_TYPE_TINYINT_VARIANT)
                    entry :=
                        add(MODULUS, signextend(INT8_SIZE_MINUS_ONE, shr(INT8_PADDING_BITS, calldataload(result_ptr))))
                    result_ptr_out := add(result_ptr, INT8_SIZE)
                    entry := mod(entry, MODULUS)
                }
                case 3 {
                    case_const(3, DATA_TYPE_SMALLINT_VARIANT)
                    entry :=
                        add(MODULUS, signextend(INT16_SIZE_MINUS_ONE, shr(INT16_PADDING_BITS, calldataload(result_ptr))))
                    result_ptr_out := add(result_ptr, INT16_SIZE)
                    entry := mod(entry, MODULUS)
                }
                case 4 {
                    case_const(4, DATA_TYPE_INT_VARIANT)
                    entry :=
                        add(MODULUS, signextend(INT32_SIZE_MINUS_ONE, shr(INT32_PADDING_BITS, calldataload(result_ptr))))
                    result_ptr_out := add(result_ptr, INT32_SIZE)
                    entry := mod(entry, MODULUS)
                }
                case 5 {
                    case_const(5, DATA_TYPE_BIGINT_VARIANT)
                    entry :=
                        add(MODULUS, signextend(INT64_SIZE_MINUS_ONE, shr(INT64_PADDING_BITS, calldataload(result_ptr))))
                    result_ptr_out := add(result_ptr, INT64_SIZE)
                    entry := mod(entry, MODULUS)
                }
                case 7 {
                    case_const(7, DATA_TYPE_VARCHAR_VARIANT)
                    result_ptr_out, entry := read_binary(result_ptr)
                }
                case 8 {
                    case_const(8, DATA_TYPE_DECIMAL75_VARIANT)
                    entry := calldataload(result_ptr)
                    result_ptr_out := add(result_ptr, WORD_SIZE)
                    entry := mod(entry, MODULUS)
                }
                case 9 {
                    case_const(9, DATA_TYPE_TIMESTAMP_VARIANT)
                    entry :=
                        add(MODULUS, signextend(INT64_SIZE_MINUS_ONE, shr(INT64_PADDING_BITS, calldataload(result_ptr))))
                    result_ptr_out := add(result_ptr, INT64_SIZE)
                    entry := mod(entry, MODULUS)
                }
                case 10 {
                    case_const(10, DATA_TYPE_SCALAR_VARIANT)
                    entry := calldataload(result_ptr)
                    result_ptr_out := add(result_ptr, WORD_SIZE)
                    entry := mod(entry, MODULUS)
                }
                case 11 {
                    case_const(11, DATA_TYPE_VARBINARY_VARIANT)
                    result_ptr_out, entry := read_binary(result_ptr)
                }
                default { err(ERR_UNSUPPORTED_DATA_TYPE_VARIANT) }
            }
            function exclude_coverage_stop_read_entry() {} // solhint-disable-line no-empty-blocks
            // slither-disable-end cyclomatic-complexity
            // IMPORTED-YUL ../base/DataType.pre.sol::read_binary
            function exclude_coverage_start_read_binary() {} // solhint-disable-line no-empty-blocks
            function read_binary(result_ptr) -> result_ptr_out, entry {
                let free_ptr := mload(FREE_PTR)
                let len := shr(UINT64_PADDING_BITS, calldataload(result_ptr))
                result_ptr := add(result_ptr, UINT64_SIZE)

                // temps with their empty‐slice defaults
                entry := 0

                // only run this when len != 0
                if len {
                    calldatacopy(free_ptr, result_ptr, len)
                    let hash_val := keccak256(free_ptr, len)

                    // endian-swap steps
                    hash_val :=
                        or(
                            shr(128, and(hash_val, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF00000000000000000000000000000000)),
                            shl(128, and(hash_val, 0x00000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF))
                        )
                    hash_val :=
                        or(
                            shr(64, and(hash_val, 0xFFFFFFFFFFFFFFFF0000000000000000FFFFFFFFFFFFFFFF0000000000000000)),
                            shl(64, and(hash_val, 0x0000000000000000FFFFFFFFFFFFFFFF0000000000000000FFFFFFFFFFFFFFFF))
                        )
                    hash_val :=
                        or(
                            shr(32, and(hash_val, 0xFFFFFFFF00000000FFFFFFFF00000000FFFFFFFF00000000FFFFFFFF00000000)),
                            shl(32, and(hash_val, 0x00000000FFFFFFFF00000000FFFFFFFF00000000FFFFFFFF00000000FFFFFFFF))
                        )
                    hash_val :=
                        or(
                            shr(16, and(hash_val, 0xFFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000)),
                            shl(16, and(hash_val, 0x0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF))
                        )
                    hash_val :=
                        or(
                            shr(8, and(hash_val, 0xFF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00)),
                            shl(8, and(hash_val, 0x00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF))
                        )

                    entry := and(hash_val, MODULUS_MASK)
                }

                // single assign to named returns
                result_ptr_out := add(result_ptr, len)
            }
            function exclude_coverage_stop_read_binary() {} // solhint-disable-line no-empty-blocks

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
                case 7 { case_const(7, DATA_TYPE_VARCHAR_VARIANT) }
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
                case 10 { case_const(10, DATA_TYPE_SCALAR_VARIANT) }
                case 11 { case_const(11, DATA_TYPE_VARBINARY_VARIANT) }
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
                        value := addmod_bn254(value, mulmod_bn254(entry, mload(add(eval_vec, mul(i, WORD_SIZE)))))
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

            function verify_proof(
                result_ptr, plan_ptr, proof_ptr, table_lengths_ptr, commitments_ptr, placeholder_params_ptr
            ) -> evaluation_point_ptr, evaluations_ptr {
                let transcript_ptr := make_transcript(result_ptr, plan_ptr, table_lengths_ptr, commitments_ptr)
                let builder_ptr := builder_new()
                builder_set_table_chi_evaluations(builder_ptr, table_lengths_ptr)
                builder_set_placeholder_parameters(builder_ptr, placeholder_params_ptr)

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

                compute_evaluations_with_length(evaluation_point_ptr, builder_get_table_chi_evaluations(builder_ptr))
                compute_evaluations_with_length(evaluation_point_ptr, builder_get_chi_evaluations(builder_ptr))
                builder_set_singleton_chi_evaluation(
                    builder_ptr, compute_truncated_lagrange_basis_sum(1, add(evaluation_point_ptr, WORD_SIZE), num_vars)
                )
                compute_rho_evaluations(evaluation_point_ptr, builder_get_rho_evaluations(builder_ptr))

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
                let output_chi_eval
                plan_ptr, evaluations_ptr, output_chi_eval := proof_plan_evaluate(plan_ptr, builder_ptr)
                builder_check_aggregate_evaluation(builder_ptr)
            }

            function verify_query(
                result_ptr, plan_ptr, placeholder_params_ptr, proof_ptr, table_lengths_ptr, commitments_ptr
            ) {
                let evaluation_point_ptr, evaluations_ptr :=
                    verify_proof(
                        result_ptr, plan_ptr, proof_ptr, table_lengths_ptr, commitments_ptr, placeholder_params_ptr
                    )
                verify_result_evaluations(result_ptr, evaluation_point_ptr, evaluations_ptr)
            }

            // Revert if the commitments array has an odd length
            let commitments_len := mload(__commitments)
            if mod(commitments_len, 2) { err(ERR_COMMITMENT_ARRAY_ODD_LENGTH) }
            mstore(__commitments, div(commitments_len, 2))
            verify_query(
                __result.offset, __plan.offset, __placeholderParameters, __proof.offset, __tableLengths, __commitments
            )
        }
    }
}
