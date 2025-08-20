// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";

/// @title ParamsBuilder
/// @dev Library for constructing an array of sql params
library ParamsBuilder {
    struct Builder {
        bytes params;
        uint64 length;
    }

    /// @dev Returns an array of parameters
    /// @param builder The parameter builder
    /// @return serializedParams The serialized parameters
    /// @return params The parameters as scalars
    function buildParams(Builder calldata builder)
        external
        pure
        returns (bytes memory serializedParams, uint256[] memory params)
    {
        bytes calldata paramsAsBytes = builder.params;
        uint64 length = builder.length;
        serializedParams = abi.encodePacked(length, paramsAsBytes);
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
            let paramsOffset := paramsAsBytes.offset
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

    function incrementLength(Builder memory builder) internal pure {
        ++builder.length;
    }

    /// @dev Adds a bool to the builder array
    /// @param builder The parameter builder
    /// @param param The parameter to push to the array
    /// @return builderOut The updated builder
    function addBool(Builder memory builder, bool param) external pure returns (Builder memory builderOut) {
        incrementLength(builder);
        builder.params = abi.encodePacked(builder.params, DATA_TYPE_BOOLEAN_VARIANT, param);
        builderOut = builder;
    }

    /// @dev Adds a int8 to the builder array
    /// @param builder The parameter builder
    /// @param param The parameter to push to the array
    /// @return builderOut The updated builder
    function addTinyInt(Builder memory builder, int8 param) external pure returns (Builder memory builderOut) {
        incrementLength(builder);
        builder.params = abi.encodePacked(builder.params, DATA_TYPE_TINYINT_VARIANT, param);
        builderOut = builder;
    }

    /// @dev Adds a int16 to the builder array
    /// @param builder The parameter builder
    /// @param param The parameter to push to the array
    /// @return builderOut The updated builder
    function addSmallInt(Builder memory builder, int16 param) external pure returns (Builder memory builderOut) {
        incrementLength(builder);
        builder.params = abi.encodePacked(builder.params, DATA_TYPE_SMALLINT_VARIANT, param);
        builderOut = builder;
    }

    /// @dev Adds a int32 to the builder array
    /// @param builder The parameter builder
    /// @param param The parameter to push to the array
    /// @return builderOut The updated builder
    function addInt(Builder memory builder, int32 param) external pure returns (Builder memory builderOut) {
        incrementLength(builder);
        builder.params = abi.encodePacked(builder.params, DATA_TYPE_INT_VARIANT, param);
        builderOut = builder;
    }

    /// @dev Adds a int64 to the builder array
    /// @param builder The parameter builder
    /// @param param The parameter to push to the array
    /// @return builderOut The updated builder
    function addBigInt(Builder memory builder, int64 param) external pure returns (Builder memory builderOut) {
        incrementLength(builder);
        builder.params = abi.encodePacked(builder.params, DATA_TYPE_BIGINT_VARIANT, param);
        builderOut = builder;
    }

    /// @dev Adds a string to the builder array
    /// @param builder The parameter builder
    /// @param param The parameter to push to the array
    /// @return builderOut The updated builder
    function addVarChar(Builder memory builder, string memory param)
        external
        pure
        returns (Builder memory builderOut)
    {
        incrementLength(builder);
        uint64 length = uint64(bytes(param).length);
        builder.params = abi.encodePacked(builder.params, DATA_TYPE_VARCHAR_VARIANT, length, param);
        builderOut = builder;
    }

    /// @dev Adds a decimal to the builder array
    /// @param builder The parameter builder
    /// @param param The parameter to push to the array
    /// @return builderOut The updated builder
    function addDecimal75(Builder memory builder, uint256 param, uint8 precision, int8 scale)
        external
        pure
        returns (Builder memory builderOut)
    {
        incrementLength(builder);
        builder.params = abi.encodePacked(builder.params, DATA_TYPE_DECIMAL75_VARIANT, precision, scale, param);
        builderOut = builder;
    }

    /// @dev Adds a timestamp to the builder array
    /// @param builder The parameter builder
    /// @param param The parameter to push to the array
    /// @return builderOut The updated builder
    function addTimeStamp(Builder memory builder, int64 param) external pure returns (Builder memory builderOut) {
        incrementLength(builder);
        builder.params = abi.encodePacked(builder.params, DATA_TYPE_TIMESTAMP_VARIANT, uint32(1), int32(0), param);
        builderOut = builder;
    }

    /// @dev Adds a scalar to the builder array
    /// @param builder The parameter builder
    /// @param param The parameter to push to the array
    /// @return builderOut The updated builder
    function addScalar(Builder memory builder, uint256 param) external pure returns (Builder memory builderOut) {
        incrementLength(builder);
        builder.params = abi.encodePacked(builder.params, DATA_TYPE_SCALAR_VARIANT, param);
        builderOut = builder;
    }

    /// @dev Adds a varbinary to the builder array
    /// @param builder The parameter builder
    /// @param param The parameter to push to the array
    /// @return builderOut The updated builder
    function addVarBinary(Builder memory builder, bytes memory param)
        external
        pure
        returns (Builder memory builderOut)
    {
        incrementLength(builder);
        uint64 length = uint64(param.length);
        builder.params = abi.encodePacked(builder.params, DATA_TYPE_VARBINARY_VARIANT, length, param);
        builderOut = builder;
    }
}
