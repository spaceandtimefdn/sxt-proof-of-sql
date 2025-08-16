// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";

/// @title ParamsBuilder
/// @dev Library for constructing an array of sql params
library ParamsBuilder {
    struct Builder {
        bytes params;
        uint64 length;
    }

    function __builderNew() external pure returns (Builder memory builder) {
        assembly {
            builder := mload(FREE_PTR)
            mstore(FREE_PTR, add(builder, WORDX2_SIZE))
        }
        builder.length = 0;
        builder.params = hex"";
    }

    function __buildParams(Builder memory builder) external pure returns (bytes memory serializedParams) {
        serializedParams = abi.encodePacked(builder.length, builder.params);
    }

    function __incrementLength(Builder memory builder) internal pure {
        builder.length = builder.length + 1;
    }

    function __addBool(Builder memory builder, bool param) external pure returns(Builder memory builderOut) {
        __incrementLength(builder);
        builder.params = abi.encodePacked(builder.params, LITERAL_VALUE_BOOLEAN_VARIANT, param);
        builderOut = builder;
    }

    function __addTinyInt(Builder memory builder, int8 param) external pure returns(Builder memory builderOut) {
        __incrementLength(builder);
        builder.params = abi.encodePacked(builder.params, LITERAL_VALUE_TINYINT_VARIANT, param);
        builderOut = builder;
    }

    function __addSmallInt(Builder memory builder, int16 param) external pure returns(Builder memory builderOut) {
        __incrementLength(builder);
        builder.params = abi.encodePacked(builder.params, LITERAL_VALUE_SMALLINT_VARIANT, param);
        builderOut = builder;
    }

    function __addInt(Builder memory builder, int32 param) external pure returns(Builder memory builderOut) {
        __incrementLength(builder);
        builder.params = abi.encodePacked(builder.params, LITERAL_VALUE_INT_VARIANT, param);
        builderOut = builder;
    }

    function __addBigInt(Builder memory builder, int64 param) external pure returns(Builder memory builderOut) {
        __incrementLength(builder);
        builder.params = abi.encodePacked(builder.params, LITERAL_VALUE_BIGINT_VARIANT, param);
        builderOut = builder;
    }

    function __addVarChar(Builder memory builder, string memory param) external pure returns(Builder memory builderOut) {
        __incrementLength(builder);
        uint64 length = uint64(bytes(param).length);
        builder.params = abi.encodePacked(builder.params, LITERAL_VALUE_VARCHAR_VARIANT, length, param);
        builderOut = builder;
    }

    function __addDecimal75(Builder memory builder, uint256 param, uint8 precision, int8 scale) external pure returns(Builder memory builderOut) {
        __incrementLength(builder);
        builder.params = abi.encodePacked(builder.params, LITERAL_VALUE_DECIMAL75_VARIANT, precision, scale, param);
        builderOut = builder;
    }

    function __addTimeStamp(Builder memory builder, int64 param) external pure returns(Builder memory builderOut) {
        __incrementLength(builder);
        builder.params = abi.encodePacked(builder.params, LITERAL_VALUE_TIMESTAMP_VARIANT, uint32(1), int32(0), param);
        builderOut = builder;
    }

    function __addScalar(Builder memory builder, uint256 param) external pure returns(Builder memory builderOut) {
        __incrementLength(builder);
        builder.params = abi.encodePacked(builder.params, LITERAL_VALUE_SCALAR_VARIANT, param);
        builderOut = builder;
    }

    function __addVarBinary(Builder memory builder, bytes memory param) external pure returns(Builder memory builderOut) {
        __incrementLength(builder);
        builder.params = abi.encodePacked(builder.params, LITERAL_VALUE_SCALAR_VARIANT, param);
        builderOut = builder;
    }
}
