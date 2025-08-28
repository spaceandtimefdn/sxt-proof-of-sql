// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";

library Verifier {
    function verify(
        bytes calldata __result,
        bytes calldata __plan,
        uint256[] memory __placeholderParameters,
        bytes calldata __proof,
        bytes[] calldata __tableCommitments
    ) public view {
        (uint256[] memory tableLengths, uint256[] memory commitments) =
            getCommitmentsAndLength(__plan, __tableCommitments);
        __internalVerify({
            __result: __result,
            __plan: __plan,
            __placeholderParameters: __placeholderParameters,
            __proof: __proof,
            __tableLengths: tableLengths,
            __commitments: commitments
        });
    }

    struct TableCommitment {
        uint256 commitmentsPtr;
        uint64 tableLength;
        bytes32[] columnNameHashes;
    }

    // slither-disable-next-line cyclomatic-complexity
    function deserializeTableCommitment(bytes calldata tableCommitment)
        internal
        pure
        returns (TableCommitment memory result)
    {
        uint256 commitmentsPtr;
        uint64 tableLength;
        // columnNameHashes[columnId] = columnNameHash
        bytes32[] memory columnNameHashes;
        assembly {
            // IMPORT-YUL ../base/Hash.pre.sol
            function hash_string(ptr, free_ptr) -> ptr_out, free_ptr_out {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/SwitchUtil.pre.sol
            function case_const(lhs, rhs) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/DataType.pre.sol
            function read_data_type(ptr) -> ptr_out, data_type {
                revert(0, 0)
            }

            let ptr := tableCommitment.offset

            // range.start (usize) must be 0
            if shr(UINT64_PADDING_BITS, calldataload(ptr)) { err(ERR_TABLE_COMMITMENT_UNSUPPORTED) }
            ptr := add(ptr, UINT64_SIZE)

            // range.end *usize) is the table length
            tableLength := shr(UINT64_PADDING_BITS, calldataload(ptr))
            ptr := add(ptr, UINT64_SIZE)

            // commitments.len() (usize) is the number of columns
            let num_columns := shr(UINT64_PADDING_BITS, calldataload(ptr))
            ptr := add(ptr, UINT64_SIZE)

            // each commitment is a 2-word commitment
            commitmentsPtr := ptr
            ptr := add(ptr, mul(num_columns, WORDX2_SIZE))

            // column_metadata.len() (usize) must match the number of columns
            if sub(num_columns, shr(UINT64_PADDING_BITS, calldataload(ptr))) { err(ERR_TABLE_COMMITMENT_UNSUPPORTED) }
            ptr := add(ptr, UINT64_SIZE)

            // allocating space for column namess
            let free_ptr := mload(FREE_PTR)
            columnNameHashes := free_ptr

            // initializing length of column names
            mstore(free_ptr, num_columns)
            free_ptr := add(free_ptr, WORD_SIZE)

            // for each entry in column_metadata
            for {} num_columns { num_columns := sub(num_columns, 1) } {
                ptr, free_ptr := hash_string(ptr, free_ptr)

                // column_metadata[i].Ident.quote_style (Option<char>) must be None, i.e. 0
                if shr(UINT8_PADDING_BITS, calldataload(ptr)) { err(ERR_TABLE_COMMITMENT_UNSUPPORTED) }
                ptr := add(ptr, UINT8_SIZE)

                let data_type
                ptr, data_type := read_data_type(ptr)

                // column_metadata[i].ColumnCommitmentMetadata.bounds (ColumnBounds)
                let variant := shr(UINT32_PADDING_BITS, calldataload(ptr))
                ptr := add(ptr, UINT32_SIZE)
                function skip_bounds(data_size, ptr_in) -> ptr_out {
                    let bounds_variant := shr(UINT32_PADDING_BITS, calldataload(ptr_in))
                    ptr_out := add(ptr_in, UINT32_SIZE)
                    if bounds_variant { ptr_out := add(ptr_out, mul(data_size, 2)) }
                }
                switch variant
                // ColumnBounds::NoOrder
                case 0 {}
                // ColumnBounds::Uint8
                case 1 { ptr := skip_bounds(UINT8_SIZE, ptr) }
                // ColumnBounds::TinyInt
                case 2 { ptr := skip_bounds(UINT8_SIZE, ptr) }
                // ColumnBounds::SmallInt
                case 3 { ptr := skip_bounds(UINT16_SIZE, ptr) }
                // ColumnBounds::Int
                case 4 { ptr := skip_bounds(UINT32_SIZE, ptr) }
                // ColumnBounds::BigInt
                case 5 { ptr := skip_bounds(UINT64_SIZE, ptr) }
                // ColumnBounds::Int128
                case 6 { ptr := skip_bounds(UINT128_SIZE, ptr) }
                // ColumnBounds::TimestampTZ
                case 7 { ptr := skip_bounds(UINT64_SIZE, ptr) }
                default { err(ERR_TABLE_COMMITMENT_UNSUPPORTED) }
            }

            // done allocating space for column names
            mstore(FREE_PTR, free_ptr)
        }
        result = TableCommitment(commitmentsPtr, tableLength, columnNameHashes);
    }

    function deserializeTableCommitments(bytes[] calldata tableCommitments)
        internal
        pure
        returns (
            // tableCommitments[tableId] = TableCommitment
            TableCommitment[] memory result
        )
    {
        uint256 numTableCommitments = tableCommitments.length;
        result = new TableCommitment[](numTableCommitments);
        for (uint256 i = 0; i < numTableCommitments; ++i) {
            result[i] = deserializeTableCommitment(tableCommitments[i]);
        }
    }

    function deserializeProofPlanPrefix(bytes calldata plan)
        internal
        pure
        returns (
            // tableNameHashes[tableId] = tableNameHash
            bytes32[] memory tableNameHashes,
            // columnTableIndexes[columnId] = tableId
            uint64[] memory columnTableIndexes,
            // columnNameHashes[columnId] = columnNameHash
            bytes32[] memory columnNameHashes
        )
    {
        assembly {
            // IMPORT-YUL ../base/Hash.pre.sol
            function hash_string(ptr, free_ptr) -> ptr_out, free_ptr_out {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/SwitchUtil.pre.sol
            function case_const(lhs, rhs) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/DataType.pre.sol
            function read_data_type(ptr) -> ptr_out, data_type {
                revert(0, 0)
            }

            let ptr := plan.offset

            let free_ptr := mload(FREE_PTR)

            // tables.len() (usize) is the number of tables
            let num_tables := shr(UINT64_PADDING_BITS, calldataload(ptr))
            ptr := add(ptr, UINT64_SIZE)

            // allocating space for table names
            tableNameHashes := free_ptr
            mstore(free_ptr, num_tables)
            free_ptr := add(free_ptr, WORD_SIZE)

            // for each table
            for {} num_tables { num_tables := sub(num_tables, 1) } { ptr, free_ptr := hash_string(ptr, free_ptr) }
            // done allocating space for table names

            // columns.len() (usize) is the number of columns
            let num_columns := shr(UINT64_PADDING_BITS, calldataload(ptr))
            ptr := add(ptr, UINT64_SIZE)

            // allocating space for column table indexes
            columnTableIndexes := free_ptr
            let index_ptr := free_ptr

            // initializing length of column table indexes
            mstore(index_ptr, num_columns)
            index_ptr := add(index_ptr, WORD_SIZE)

            free_ptr := add(index_ptr, mul(num_columns, WORD_SIZE))
            // done allocating space for column table indexes

            // allocating space for column names
            columnNameHashes := free_ptr

            // initializing length of column names
            mstore(free_ptr, num_columns)
            free_ptr := add(free_ptr, WORD_SIZE)

            // for each column
            for {} num_columns { num_columns := sub(num_columns, 1) } {
                // column[i].0 (usize) is the table id. We store it in the columnTableIndexes array
                mstore(index_ptr, shr(UINT64_PADDING_BITS, calldataload(ptr)))
                ptr := add(ptr, UINT64_SIZE)
                index_ptr := add(index_ptr, WORD_SIZE)

                ptr, free_ptr := hash_string(ptr, free_ptr)

                let data_type
                ptr, data_type := read_data_type(ptr)
            }

            // done allocating space for column names
            mstore(FREE_PTR, free_ptr)
        }
    }

    /// @notice Internal function to get the relevant commitments
    /// @dev validates that all commitments are found
    /// @return commitments the commitments in the order of the columns
    function getRelevantCommitments(
        uint64[] memory columnTableIndexes,
        bytes32[] memory columnNameHashes,
        TableCommitment[] memory tableCommitments
    ) internal pure returns (uint256[] memory commitments) {
        uint256 numColumns = columnTableIndexes.length;
        commitments = new uint256[](numColumns * 2);
        uint256 commitmentsFreePtr;
        assembly {
            commitmentsFreePtr := add(commitments, 0x20)
        }

        for (uint256 i = 0; i < numColumns; ++i) {
            uint64 columnTableIndex = columnTableIndexes[i];
            bytes32 columnNameHash = columnNameHashes[i];

            if (!(columnTableIndex < tableCommitments.length)) {
                revert Errors.CommitmentsNotFound();
            }
            TableCommitment memory tableCommitment = tableCommitments[columnTableIndex];
            uint256 commitmentsPtr = tableCommitment.commitmentsPtr;
            bool found = false;
            uint256 columnNameHashesLength = tableCommitment.columnNameHashes.length;
            for (uint256 j = 0; j < columnNameHashesLength; ++j) {
                if (tableCommitment.columnNameHashes[j] == columnNameHash) {
                    assembly {
                        calldatacopy(commitmentsFreePtr, add(commitmentsPtr, mul(j, WORDX2_SIZE)), WORDX2_SIZE)
                        commitmentsFreePtr := add(commitmentsFreePtr, WORDX2_SIZE)
                    }
                    found = true;
                    break;
                }
            }
            if (!found) {
                revert Errors.CommitmentsNotFound();
            }
        }
    }

    function getTableLengths(TableCommitment[] memory tableCommitments)
        private
        pure
        returns (uint256[] memory tableLengths)
    {
        uint256 numTables = tableCommitments.length;
        tableLengths = new uint256[](numTables);
        for (uint256 i = 0; i < numTables; ++i) {
            tableLengths[i] = tableCommitments[i].tableLength;
        }
    }

    function getCommitmentsAndLength(bytes calldata queryPlan, bytes[] calldata tableCommitmentsAsBytes)
        internal
        pure
        returns (uint256[] memory __tableLengths, uint256[] memory __commitments)
    {
        TableCommitment[] memory tableCommitments = deserializeTableCommitments(tableCommitmentsAsBytes);
        (, uint64[] memory columnTableIndexes, bytes32[] memory columnNameHashes) =
            deserializeProofPlanPrefix(queryPlan);

        // construct `uint256[] memory commitments` and validate that all commitments are found
        uint256[] memory commitments = getRelevantCommitments(columnTableIndexes, columnNameHashes, tableCommitments);

        // construct `uint256[] memory tableLengths`
        uint256[] memory tableLengths = getTableLengths(tableCommitments);
        __tableLengths = tableLengths;
        __commitments = commitments;
    }

    function __internalVerify(
        bytes calldata __result,
        bytes calldata __plan,
        uint256[] memory __placeholderParameters,
        bytes calldata __proof,
        uint256[] memory __tableLengths,
        uint256[] memory __commitments
    ) public view {
        assembly {
            // IMPORT-YUL ../base/Array.pre.sol
            function get_array_element(arr_ptr, index) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Array.pre.sol
            function read_uint64_array(proof_ptr_init) -> proof_ptr, array_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Array.pre.sol
            function read_word_array(proof_ptr_init) -> proof_ptr, array_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Array.pre.sol
            function read_wordx2_array(proof_ptr_init) -> proof_ptr, array_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Array.pre.sol
            function read_uint64_array_as_uint512_array(source_ptr) -> source_ptr_out, array_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Array.pre.sol
            function read_word_array_as_uint512_array(input_array_ptr) -> array_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/ECPrecompiles.pre.sol
            function calldata_ec_add_assign(args_ptr, c_ptr) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL ../base/ECPrecompiles.pre.sol
            function calldata_ec_mul_add_assign(args_ptr, c_ptr, scalar) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL ../base/ECPrecompiles.pre.sol
            function constant_ec_mul_add_assign(args_ptr, c_x, c_y, scalar) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL ../base/ECPrecompiles.pre.sol
            function ec_add(args_ptr) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL ../base/ECPrecompiles.pre.sol
            function ec_add_assign(args_ptr, c_ptr) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL ../base/ECPrecompiles.pre.sol
            function ec_mul(args_ptr) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL ../base/ECPrecompiles.pre.sol
            function ec_mul_assign(args_ptr, scalar) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL ../base/ECPrecompiles.pre.sol
            function ec_pairing_x2(args_ptr) -> success {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
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
            function min(a, b) -> minimum {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/LagrangeBasisEvaluation.pre.sol
            function compute_truncated_lagrange_basis_inner_product(length, x_ptr, y_ptr, num_vars) -> result {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/LagrangeBasisEvaluation.pre.sol
            function compute_truncated_lagrange_basis_sum(length, x_ptr, num_vars) -> result {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/MathUtil.pre.sol
            function log2_up(value) -> exponent {
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
            function dequeue_uint512(queue_ptr) -> upper, lower {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/SwitchUtil.pre.sol
            function case_const(lhs, rhs) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Transcript.sol
            function append_array(transcript_ptr, array_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Transcript.sol
            function append_calldata(transcript_ptr, offset, size) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Transcript.sol
            function draw_challenge(transcript_ptr) -> result {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Transcript.sol
            function draw_challenges(transcript_ptr, count) -> result_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_challenge(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_chi_evaluation(builder_ptr) -> value {
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
            function builder_get_aggregate_evaluation(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_chi_evaluations(builder_ptr) -> values_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_singleton_chi_evaluation(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_rho_evaluations(builder_ptr) -> values_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_rho_evaluation(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_column_evaluation(builder_ptr, column_num) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_column_evaluations(builder_ptr) -> values_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_final_round_commitments(builder_ptr) -> values_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_final_round_mles(builder_ptr) -> values_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_first_round_commitments(builder_ptr) -> values_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_first_round_mles(builder_ptr) -> values_ptr {
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
            function builder_get_table_chi_evaluations(builder_ptr) -> values_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_placeholder_parameter(builder_ptr, index) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_new() -> builder_ptr {
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
            function builder_set_aggregate_evaluation(builder_ptr, value) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_bit_distributions(builder_ptr, values_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_challenges(builder_ptr, challenges_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_chi_evaluations(builder_ptr, values_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_column_evaluations(builder_ptr, values_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_constraint_multipliers(builder_ptr, values_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_final_round_commitments(builder_ptr, values_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_final_round_mles(builder_ptr, values_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_first_round_commitments(builder_ptr, values_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_first_round_mles(builder_ptr, values_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_max_degree(builder_ptr, value) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_rho_evaluations(builder_ptr, values_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_row_multipliers_evaluation(builder_ptr, value) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_singleton_chi_evaluation(builder_ptr, value) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_table_chi_evaluations(builder_ptr, values_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_placeholder_parameters(builder_ptr, values_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_bit_distribution(builder_ptr) -> vary_mask, leading_bit_mask {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_check_aggregate_evaluation(builder_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_chi_evaluation_with_length(builder_ptr) -> length, chi_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_table_chi_evaluation_with_length(builder_ptr, table_num) -> length, chi_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../hyperkzg/HyperKZGHelpers.pre.sol
            function bivariate_evaluation(v_ptr, q, d, ell) -> b {
                revert(0, 0)
            }
            // IMPORT-YUL ../hyperkzg/HyperKZGHelpers.pre.sol
            function check_v_consistency(v_ptr, r, x, y) {
                revert(0, 0)
            }
            // IMPORT-YUL ../hyperkzg/HyperKZGHelpers.pre.sol
            function compute_gl_msm(com_ptr, length, w_ptr, commitment_ptr, r, q, d, b, scratch) {
                revert(0, 0)
            }
            // IMPORT-YUL ../hyperkzg/HyperKZGHelpers.pre.sol
            function run_transcript(com_ptr, v_ptr, w_ptr, transcript_ptr, ell) -> r, q, d {
                revert(0, 0)
            }
            // IMPORT-YUL ../hyperkzg/HyperKZGHelpers.pre.sol
            function univariate_group_evaluation(g_ptr, e, length, scratch) {
                revert(0, 0)
            }
            // IMPORT-YUL ../hyperkzg/HyperKZGVerifier.pre.sol
            function verify_hyperkzg(proof_ptr, transcript_ptr, commitment_ptr, x, y) {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/Shift.pre.sol
            function compute_shift_identity_constraint(star, chi_plus_one, fold) -> constraint {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/Shift.pre.sol
            function compute_shift_fold(alpha, beta, eval, rho) -> fold {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/Shift.pre.sol
            function shift_evaluate(builder_ptr, alpha, beta, expr_eval, chi_eval) ->
                shifted_expr_eval,
                chi_plus_one_eval
            {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/Monotonic.pre.sol
            function monotonic_verify(builder_ptr, alpha, beta, column_eval, chi_eval, strict, asc) {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/SignExpr.pre.sol
            function sign_expr_evaluate(expr_eval, builder_ptr, chi_eval) -> result_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/ColumnExpr.pre.sol
            function column_expr_evaluate(expr_ptr, builder_ptr) -> expr_ptr_out, eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/EqualsExpr.pre.sol
            function equals_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/LiteralExpr.pre.sol
            function literal_expr_evaluate(expr_ptr, chi_eval) -> expr_ptr_out, eval {
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
            // IMPORT-YUL ../proof_exprs/InequalityExpr.pre.sol
            function inequality_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_plans/GroupByExec.pre.sol
            function compute_g_in_star_eval(plan_ptr, builder_ptr, alpha, beta, input_chi_eval) ->
                plan_ptr_out,
                g_in_fold,
                g_in_star_eval_times_selection_eval,
                num_group_by_columns
            {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_plans/GroupByExec.pre.sol
            function compute_sum_in_fold_eval(plan_ptr, builder_ptr, alpha, beta, input_chi_eval) ->
                plan_ptr_out,
                sum_in_fold_eval,
                num_sum_columns
            {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_plans/GroupByExec.pre.sol
            function compute_g_out_star_eval(builder_ptr, alpha, beta, output_chi_eval, evaluations_ptr) ->
                g_out_star_eval
            {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_plans/GroupByExec.pre.sol
            function compute_sum_out_fold_eval(
                builder_ptr, alpha, beta, output_chi_eval, num_sum_columns, evaluations_ptr
            ) -> sum_out_fold_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_plans/GroupByExec.pre.sol
            function read_input_evals(plan_ptr, builder_ptr, alpha, beta) ->
                plan_ptr_out,
                partial_dlog_zero_sum_constraint_eval,
                num_group_by_columns,
                num_sum_columns
            {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_plans/GroupByExec.pre.sol
            function read_output_evals(
                builder_ptr, alpha, beta, partial_dlog_zero_sum_constraint_eval, num_group_by_columns, num_sum_columns
            ) -> evaluations_ptr, output_chi_eval {
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
            // IMPORT-YUL ../proof_gadgets/FoldUtil.pre.sol
            function fold_expr_evals(plan_ptr, builder_ptr, input_chi_eval, beta, column_count) -> plan_ptr_out, fold {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/FoldUtil.pre.sol
            function fold_column_expr_evals(plan_ptr, builder_ptr, beta, column_count) -> plan_ptr_out, fold {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/FoldUtil.pre.sol
            function fold_first_round_mles(builder_ptr, beta, column_count) -> fold, evaluations_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_plans/FilterExec.pre.sol
            function compute_filter_folds(plan_ptr, builder_ptr, input_chi_eval, beta) ->
                plan_ptr_out,
                c_fold,
                d_fold,
                evaluations_ptr
            {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_plans/EmptyExec.pre.sol
            function empty_exec_evaluate(builder_ptr) -> evaluations_ptr, output_length, output_chi_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/FoldLogExpr.pre.sol
            function fold_log_star_evaluate_from_fold(builder_ptr, fold, chi_eval) -> star {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/FoldLogExpr.pre.sol
            function fold_log_evaluate_from_mles(builder_ptr, alpha, beta, column_count, chi_eval) ->
                fold,
                star,
                evaluations_ptr
            {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/FoldLogExpr.pre.sol
            function fold_log_star_evaluate_from_mles(builder_ptr, alpha, beta, column_count, chi_eval) ->
                star,
                evaluations_ptr
            {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/FoldLogExpr.pre.sol
            function fold_log_star_evaluate(builder_ptr, alpha, beta, column_evals, chi_eval) -> star {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/FoldLogExpr.pre.sol
            function fold_log_star_evaluate_from_column_exprs(
                plan_ptr, builder_ptr, alpha, beta, column_count, chi_eval
            ) -> plan_ptr_out, star {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_gadgets/FilterBase.pre.sol
            function verify_filter(builder_ptr, c_fold, d_fold, input_chi_eval, output_chi_eval, selection_eval) {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_plans/FilterExec.pre.sol
            function filter_exec_evaluate(plan_ptr, builder_ptr) ->
                plan_ptr_out,
                evaluations_ptr,
                output_length,
                output_chi_eval
            {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_plans/TableExec.pre.sol
            function table_exec_evaluate(plan_ptr, builder_ptr) ->
                plan_ptr_out,
                evaluations_ptr,
                output_length,
                output_chi_eval
            {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_plans/ProjectionExec.pre.sol
            function projection_exec_evaluate(plan_ptr, builder_ptr) ->
                plan_ptr_out,
                evaluations_ptr,
                output_length,
                output_chi_eval
            {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_plans/GroupByExec.pre.sol
            function group_by_exec_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr, output_chi_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../sumcheck/Sumcheck.pre.sol
            function process_round(proof_ptr, degree, challenge) -> proof_ptr_out, round_evaluation, actual_sum {
                revert(0, 0)
            }
            // IMPORT-YUL ../sumcheck/Sumcheck.pre.sol
            function verify_sumcheck_proof(transcript_ptr, proof_ptr, num_vars) ->
                proof_ptr_out,
                evaluation_point_ptr,
                expected_evaluation,
                degree
            {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_plans/SliceExec.pre.sol
            function get_and_verify_slice_length(plan_ptr, builder_ptr, input_length) ->
                plan_ptr_out,
                output_length,
                output_chi_eval,
                selection_eval
            {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_plans/SliceExec.pre.sol
            function compute_slice_folds(builder_ptr, input_evaluations_ptr) -> c_fold, d_fold, evaluations_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_plans/SliceExec.pre.sol
            function slice_exec_evaluate(plan_ptr, builder_ptr) ->
                plan_ptr_out,
                evaluations_ptr,
                output_length,
                output_chi_eval
            {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_plans/UnionExec.pre.sol
            function union_input_evaluate(plan_ptr, builder_ptr, gamma, beta) ->
                plan_ptr_out,
                output_length,
                num_columns,
                zerosum_constraint
            {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_plans/UnionExec.pre.sol
            function union_exec_evaluate(plan_ptr, builder_ptr) ->
                plan_ptr_out,
                evaluations_ptr,
                output_length,
                output_chi_eval
            {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_plans/ProofPlan.pre.sol
            function proof_plan_evaluate(plan_ptr, builder_ptr) ->
                plan_ptr_out,
                evaluations_ptr,
                output_length,
                output_chi_eval
            {
                revert(0, 0)
            }

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
            // IMPORT-YUL ../base/LagrangeBasisEvaluation.pre.sol
            function compute_chi_evaluations(evaluation_point_ptr, array_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/LagrangeBasisEvaluation.pre.sol
            function compute_rho_evaluations(evaluation_point_ptr, array_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/LagrangeBasisEvaluation.pre.sol
            function compute_evaluations_with_length(evaluation_point_ptr, array_ptr) {
                revert(0, 0)
            }
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
            // IMPORT-YUL PlanUtil.pre.sol
            function skip_plan_names(plan_ptr) -> plan_ptr_out {
                revert(0, 0)
            }

            // IMPORT-YUL ../hyperkzg/HyperKZGBatch.pre.sol
            function batch_pcs(args_ptr, transcript_ptr, commitments_ptr, evaluations_ptr, batch_eval) -> batch_eval_out
            {
                revert(0, 0)
            }

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

            // IMPORT-YUL ../base/LagrangeBasisEvaluation.pre.sol
            function compute_evaluation_vec(length, evaluation_point_ptr) -> evaluations_ptr {
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

            // slither-disable-start cyclomatic-complexity
            // IMPORT-YUL ../base/DataType.pre.sol
            function read_data_type(ptr) -> ptr_out, data_type {
                revert(0, 0)
            }
            // slither-disable-end cyclomatic-complexity

            // IMPORT-YUL ResultVerifier.pre.sol
            function verify_result_evaluations(result_ptr, evaluation_point_ptr, evaluations_ptr) {
                revert(0, 0)
            }

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

                table_lengths_ptr := read_word_array_as_uint512_array(table_lengths_ptr)
                builder_set_table_chi_evaluations(builder_ptr, table_lengths_ptr)
                compute_evaluations_with_length(evaluation_point_ptr, table_lengths_ptr)
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

                {
                    let output_length, output_chi_eval
                    plan_ptr, evaluations_ptr, output_length, output_chi_eval :=
                        proof_plan_evaluate(plan_ptr, builder_ptr)
                }
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
