// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";

library Commitment {
    struct TableCommitment {
        uint256 commitmentsPtr;
        uint64 tableLength;
        bytes32[] columnNameHashes;
    }

    // slither-disable-next-line cyclomatic-complexity
    function deserializeTableCommitment(bytes calldata tableCommitment)
        internal
        pure
        returns (Commitment.TableCommitment memory result)
    {
        uint256 commitmentsPtr;
        uint64 tableLength;
        // columnNameHashes[columnId] = columnNameHash
        bytes32[] memory columnNameHashes;
        assembly {
            function err(code) {
                mstore(0, code)
                revert(28, 4)
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
                // column_metadata[i].Ident.value.len() (usize) is the number of characters in the column name
                let name_len := shr(UINT64_PADDING_BITS, calldataload(ptr))
                ptr := add(ptr, UINT64_SIZE)

                // column_metadata[i].Ident.value (usize) is the column name. We hash it and store it in the columnNameHashes array
                calldatacopy(free_ptr, ptr, name_len)
                mstore(free_ptr, keccak256(free_ptr, name_len))
                ptr := add(ptr, name_len)
                free_ptr := add(free_ptr, WORD_SIZE)

                // column_metadata[i].Ident.quote_style (Option<char>) must be None, i.e. 0
                if shr(UINT8_PADDING_BITS, calldataload(ptr)) { err(ERR_TABLE_COMMITMENT_UNSUPPORTED) }
                ptr := add(ptr, UINT8_SIZE)

                // column_metadata[i].ColumnCommitmentMetadata.column_type (ColumnType)
                switch shr(UINT32_PADDING_BITS, calldataload(ptr))
                // ColumnType::Decimal75
                case 8 { ptr := add(ptr, add(UINT32_SIZE, add(UINT8_SIZE, UINT8_SIZE))) }
                // ColumnType::TimestampTZ
                case 9 { ptr := add(ptr, add(UINT32_SIZE, add(UINT32_SIZE, UINT32_SIZE))) }
                default { ptr := add(ptr, UINT32_SIZE) }

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
        result = Commitment.TableCommitment(commitmentsPtr, tableLength, columnNameHashes);
    }

    function deserializeTableCommitments(bytes[] calldata tableCommitments)
        internal
        pure
        returns (
            // tableCommitments[tableId] = TableCommitment
            Commitment.TableCommitment[] memory result
        )
    {
        uint256 numTableCommitments = tableCommitments.length;
        result = new Commitment.TableCommitment[](numTableCommitments);
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
            for {} num_tables { num_tables := sub(num_tables, 1) } {
                // table[i].len() (usize) is the number of characters in the table name
                let name_len := shr(UINT64_PADDING_BITS, calldataload(ptr))
                ptr := add(ptr, UINT64_SIZE)

                // table[i] is the table name. We hash it and store it in the tableNameHashes array
                calldatacopy(free_ptr, ptr, name_len)
                mstore(free_ptr, keccak256(free_ptr, name_len))
                ptr := add(ptr, name_len)
                free_ptr := add(free_ptr, WORD_SIZE)
            }
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

                // column[i].1.len() (usize) is number of characters in the column name
                let name_len := shr(UINT64_PADDING_BITS, calldataload(ptr))
                ptr := add(ptr, UINT64_SIZE)

                // column[i].1 (usize) is the column name. We hash it and store it in the columnNameHashes array
                calldatacopy(free_ptr, ptr, name_len)
                mstore(free_ptr, keccak256(free_ptr, name_len))
                ptr := add(ptr, name_len)
                free_ptr := add(free_ptr, WORD_SIZE)

                // column[i].2 (ColumnType)
                switch shr(UINT32_PADDING_BITS, calldataload(ptr))
                // ColumnType::Decimal75
                case 8 { ptr := add(ptr, add(UINT32_SIZE, add(UINT8_SIZE, UINT8_SIZE))) }
                // ColumnType::TimestampTZ
                case 9 { ptr := add(ptr, add(UINT32_SIZE, add(UINT16_SIZE, UINT16_SIZE))) }
                default { ptr := add(ptr, UINT32_SIZE) }
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
        internal
        pure
        returns (uint256[] memory tableLengths)
    {
        uint256 numTables = tableCommitments.length;
        tableLengths = new uint256[](numTables);
        for (uint256 i = 0; i < numTables; ++i) {
            tableLengths[i] = tableCommitments[i].tableLength;
        }
    }

    function getCommitmentsAndLength(bytes calldata queryPlan, TableCommitment[] memory tableCommitments)
        internal
        pure
        returns (uint256[] memory __tableLengths, uint256[] memory __commitments)
    {
        (, uint64[] memory columnTableIndexes, bytes32[] memory columnNameHashes) =
            deserializeProofPlanPrefix(queryPlan);

        // construct `uint256[] memory commitments` and validate that all commitments are found
        uint256[] memory commitments = getRelevantCommitments(columnTableIndexes, columnNameHashes, tableCommitments);

        // construct `uint256[] memory tableLengths`
        uint256[] memory tableLengths = getTableLengths(tableCommitments);
        __tableLengths = tableLengths;
        __commitments = commitments;
    }
}
