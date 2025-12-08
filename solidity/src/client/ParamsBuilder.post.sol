// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";

/// @title ParamsBuilder
/// @dev Library for constructing an array of sql params
library ParamsBuilder {
    /// @dev Returns a serialized array of parameters
    /// @param arrayOfSerializedParamElements An array of serialized parameters
    /// @return serializedParams The serialized array of parameters
    function serializeParamArray(bytes[] memory arrayOfSerializedParamElements)
        internal
        pure
        returns (bytes memory serializedParams)
    {
        uint256 uncastLength = arrayOfSerializedParamElements.length;
        if (uncastLength > MAX_UINT64) {
            revert Errors.TooManyParameters();
        } else {
            uint64 length = uint64(uncastLength);
            serializedParams = abi.encodePacked(length);
            for (uint64 i = 0; i < length; ++i) {
                serializedParams = abi.encodePacked(serializedParams, arrayOfSerializedParamElements[i]);
            }
        }
    }

    /// @dev Returns an array of parameters
    /// @param serializedParams The serialized parameters
    /// @return params The parameters as scalars
    function deserializeParamArray(bytes calldata serializedParams) internal pure returns (uint256[] memory params) {
        uint64 length;
        assembly {
            length := shr(UINT64_PADDING_BITS, calldataload(serializedParams.offset))
        }
        params = new uint256[](length);

        assembly {
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
            function exclude_coverage_start_err() {} // solhint-disable-line no-empty-blocks
            function err(code) {
                mstore(0, code)
                revert(28, 4)
            }
            function exclude_coverage_stop_err() {} // solhint-disable-line no-empty-blocks
            function exclude_coverage_start_case_const() {} // solhint-disable-line no-empty-blocks
            function case_const(lhs, rhs) {
                if sub(lhs, rhs) { err(ERR_INCORRECT_CASE_CONST) }
            }
            function exclude_coverage_stop_case_const() {} // solhint-disable-line no-empty-blocks
            function exclude_coverage_start_read_entry() {} // solhint-disable-line no-empty-blocks
            // slither-disable-start cyclomatic-complexity
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
                    entry := add(
                        MODULUS,
                        signextend(INT8_SIZE_MINUS_ONE, shr(INT8_PADDING_BITS, calldataload(result_ptr)))
                    )
                    result_ptr_out := add(result_ptr, INT8_SIZE)
                    entry := mod(entry, MODULUS)
                }
                case 3 {
                    case_const(3, DATA_TYPE_SMALLINT_VARIANT)
                    entry := add(
                        MODULUS,
                        signextend(INT16_SIZE_MINUS_ONE, shr(INT16_PADDING_BITS, calldataload(result_ptr)))
                    )
                    result_ptr_out := add(result_ptr, INT16_SIZE)
                    entry := mod(entry, MODULUS)
                }
                case 4 {
                    case_const(4, DATA_TYPE_INT_VARIANT)
                    entry := add(
                        MODULUS,
                        signextend(INT32_SIZE_MINUS_ONE, shr(INT32_PADDING_BITS, calldataload(result_ptr)))
                    )
                    result_ptr_out := add(result_ptr, INT32_SIZE)
                    entry := mod(entry, MODULUS)
                }
                case 5 {
                    case_const(5, DATA_TYPE_BIGINT_VARIANT)
                    entry := add(
                        MODULUS,
                        signextend(INT64_SIZE_MINUS_ONE, shr(INT64_PADDING_BITS, calldataload(result_ptr)))
                    )
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
                    entry := add(
                        MODULUS,
                        signextend(INT64_SIZE_MINUS_ONE, shr(INT64_PADDING_BITS, calldataload(result_ptr)))
                    )
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
            // slither-disable-end cyclomatic-complexity
            function exclude_coverage_stop_read_entry() {} // solhint-disable-line no-empty-blocks
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
                    hash_val := or(
                        shr(128, and(hash_val, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF00000000000000000000000000000000)),
                        shl(128, and(hash_val, 0x00000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF))
                    )
                    hash_val := or(
                        shr(64, and(hash_val, 0xFFFFFFFFFFFFFFFF0000000000000000FFFFFFFFFFFFFFFF0000000000000000)),
                        shl(64, and(hash_val, 0x0000000000000000FFFFFFFFFFFFFFFF0000000000000000FFFFFFFFFFFFFFFF))
                    )
                    hash_val := or(
                        shr(32, and(hash_val, 0xFFFFFFFF00000000FFFFFFFF00000000FFFFFFFF00000000FFFFFFFF00000000)),
                        shl(32, and(hash_val, 0x00000000FFFFFFFF00000000FFFFFFFF00000000FFFFFFFF00000000FFFFFFFF))
                    )
                    hash_val := or(
                        shr(16, and(hash_val, 0xFFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000)),
                        shl(16, and(hash_val, 0x0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF))
                    )
                    hash_val := or(
                        shr(8, and(hash_val, 0xFF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00)),
                        shl(8, and(hash_val, 0x00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF))
                    )

                    entry := and(hash_val, MODULUS_MASK)
                }

                // single assign to named returns
                result_ptr_out := add(result_ptr, len)
            }
            function exclude_coverage_stop_read_binary() {} // solhint-disable-line no-empty-blocks


            let paramsOffset := add(serializedParams.offset, UINT64_SIZE)
            let paramsArray := add(params, WORD_SIZE)
            for {} length { length := sub(length, 1) } {
                let data_type
                paramsOffset, data_type := read_data_type(paramsOffset)
                let entry
                paramsOffset, entry := read_entry(paramsOffset, data_type)
                mstore(paramsArray, entry)
                paramsArray := add(paramsArray, WORD_SIZE)
            }
        }
    }

    /// @dev Serializes a bool parameter
    /// @param param The parameter
    /// @return serializedParam The serialized parameter
    function boolParam(bool param) internal pure returns (bytes memory serializedParam) {
        serializedParam = abi.encodePacked(DATA_TYPE_BOOLEAN_VARIANT, param);
    }

    /// @dev Serializes a tinyint parameter
    /// @param param The parameter
    /// @return serializedParam The serialized parameter
    function tinyIntParam(int8 param) internal pure returns (bytes memory serializedParam) {
        serializedParam = abi.encodePacked(DATA_TYPE_TINYINT_VARIANT, param);
    }

    /// @dev Serializes a smallint parameter
    /// @param param The parameter
    /// @return serializedParam The serialized parameter
    function smallIntParam(int16 param) internal pure returns (bytes memory serializedParam) {
        serializedParam = abi.encodePacked(DATA_TYPE_SMALLINT_VARIANT, param);
    }

    /// @dev Serializes an int32 parameter
    /// @param param The parameter
    /// @return serializedParam The serialized parameter
    function intParam(int32 param) internal pure returns (bytes memory serializedParam) {
        serializedParam = abi.encodePacked(DATA_TYPE_INT_VARIANT, param);
    }

    /// @dev Serializes a bigint parameter
    /// @param param The parameter
    /// @return serializedParam The serialized parameter
    function bigIntParam(int64 param) internal pure returns (bytes memory serializedParam) {
        serializedParam = abi.encodePacked(DATA_TYPE_BIGINT_VARIANT, param);
    }

    /// @dev Serializes a string parameter
    /// @param param The parameter
    /// @return serializedParam The serialized parameter
    function varCharParam(string memory param) internal pure returns (bytes memory serializedParam) {
        uint64 length = uint64(bytes(param).length);
        serializedParam = abi.encodePacked(DATA_TYPE_VARCHAR_VARIANT, length, param);
    }

    /// @dev Serializes a decimal parameter
    /// @param param The parameter
    /// @param precision The precision of the decimal
    /// @param scale The scale of the decimal
    /// @return serializedParam The serialized parameter
    function decimal75Param(uint256 param, uint8 precision, int8 scale)
        internal
        pure
        returns (bytes memory serializedParam)
    {
        serializedParam = abi.encodePacked(DATA_TYPE_DECIMAL75_VARIANT, precision, scale, param);
    }

    /// @dev Serializes a timestamp parameter
    /// @param param The parameter
    /// @return serializedParam The serialized parameter
    function unixTimestampMillisParam(int64 param) internal pure returns (bytes memory serializedParam) {
        serializedParam =
            abi.encodePacked(DATA_TYPE_TIMESTAMP_VARIANT, TIMEUNIT_MILLISECOND_VARIANT, TIMEZONE_UTC, param);
    }

    /// @dev Serializes a varbinary parameter
    /// @param param The parameter
    /// @return serializedParam The serialized parameter
    function varBinaryParam(bytes memory param) internal pure returns (bytes memory serializedParam) {
        uint64 length = uint64(param.length);
        serializedParam = abi.encodePacked(DATA_TYPE_VARBINARY_VARIANT, length, param);
    }
}
