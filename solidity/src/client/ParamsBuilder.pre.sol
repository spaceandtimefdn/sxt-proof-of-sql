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
        external
        pure
        returns (bytes memory serializedParams)
    {
        uint64 length = uint64(arrayOfSerializedParamElements.length);
        serializedParams = abi.encodePacked(length);
        for (uint64 i = 0; i < length; ++i) {
            serializedParams = abi.encodePacked(serializedParams, arrayOfSerializedParamElements[i]);
        }
    }

    /// @dev Returns an array of parameters
    /// @param serializedParams The serialized parameters
    /// @return params The parameters as scalars
    function deserializeParamArray(bytes calldata serializedParams) external pure returns (uint256[] memory params) {
        uint64 length;
        assembly {
            length := shr(UINT64_PADDING_BITS, calldataload(serializedParams.offset))
        }
        params = new uint256[](length);

        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/SwitchUtil.pre.sol
            function case_const(lhs, rhs) {
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
            // skip length of byte array
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
    function boolParam(bool param) external pure returns (bytes memory serializedParam) {
        serializedParam = abi.encodePacked(DATA_TYPE_BOOLEAN_VARIANT, param);
    }

    /// @dev Serializes a tinyint parameter
    /// @param param The parameter
    /// @return serializedParam The serialized parameter
    function tinyIntParam(int8 param) external pure returns (bytes memory serializedParam) {
        serializedParam = abi.encodePacked(DATA_TYPE_TINYINT_VARIANT, param);
    }

    /// @dev Serializes a smallint parameter
    /// @param param The parameter
    /// @return serializedParam The serialized parameter
    function smallIntParam(int16 param) external pure returns (bytes memory serializedParam) {
        serializedParam = abi.encodePacked(DATA_TYPE_SMALLINT_VARIANT, param);
    }

    /// @dev Serializes an int32 parameter
    /// @param param The parameter
    /// @return serializedParam The serialized parameter
    function intParam(int32 param) external pure returns (bytes memory serializedParam) {
        serializedParam = abi.encodePacked(DATA_TYPE_INT_VARIANT, param);
    }

    /// @dev Serializes a bigint parameter
    /// @param param The parameter
    /// @return serializedParam The serialized parameter
    function bigIntParam(int64 param) external pure returns (bytes memory serializedParam) {
        serializedParam = abi.encodePacked(DATA_TYPE_BIGINT_VARIANT, param);
    }

    /// @dev Serializes a string parameter
    /// @param param The parameter
    /// @return serializedParam The serialized parameter
    function varCharParam(string memory param) external pure returns (bytes memory serializedParam) {
        uint64 length = uint64(bytes(param).length);
        serializedParam = abi.encodePacked(DATA_TYPE_VARCHAR_VARIANT, length, param);
    }

    /// @dev Serializes a decimal parameter
    /// @param param The parameter
    /// @param precision The precision of the decimal
    /// @param scale The scale of the decimal
    /// @return serializedParam The serialized parameter
    function decimal75Param(uint256 param, uint8 precision, int8 scale)
        external
        pure
        returns (bytes memory serializedParam)
    {
        serializedParam = abi.encodePacked(DATA_TYPE_DECIMAL75_VARIANT, precision, scale, param);
    }

    /// @dev Serializes a timestamp parameter
    /// @param param The parameter
    /// @return serializedParam The serialized parameter
    function timeStampParam(int64 param) external pure returns (bytes memory serializedParam) {
        serializedParam = abi.encodePacked(DATA_TYPE_TIMESTAMP_VARIANT, uint32(1), int32(0), param);
    }

    /// @dev Serializes a scalar parameter
    /// @param param The parameter
    /// @return serializedParam The serialized parameter
    function scalarParam(uint256 param) external pure returns (bytes memory serializedParam) {
        serializedParam = abi.encodePacked(DATA_TYPE_SCALAR_VARIANT, param);
    }

    /// @dev Serializes a varbinary parameter
    /// @param param The parameter
    /// @return serializedParam The serialized parameter
    function varBinaryParam(bytes memory param) external pure returns (bytes memory serializedParam) {
        uint64 length = uint64(param.length);
        serializedParam = abi.encodePacked(DATA_TYPE_VARBINARY_VARIANT, length, param);
    }
}
