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
        serializedParams = abi.encodePacked(builder.params.length, builder.params);
    }

    function __incrementLength(Builder memory builder) internal pure {
        builder.length += 1;
    }

    function __addBool(Builder memory builder, bool param) external pure {
        __incrementLength(builder);
        builder.params = abi.encodePacked(builder.params, DATA_TYPE_BOOLEAN_VARIANT, param);
    }

    function __addTinyInt(Builder memory builder, int8 param) external pure {
        __incrementLength(builder);
        builder.params = abi.encodePacked(builder.params, DATA_TYPE_TINYINT_VARIANT, param);
    }

    function __addSmallInt(Builder memory builder, int16 param) external pure {
        __incrementLength(builder);
        builder.params = abi.encodePacked(builder.params, DATA_TYPE_SMALLINT_VARIANT, param);
    }

    function __addInt(Builder memory builder, int32 param) external pure {
        __incrementLength(builder);
        builder.params = abi.encodePacked(builder.params, DATA_TYPE_INT_VARIANT, param);
    }

    function __addBigInt(Builder memory builder, int64 param) external pure {
        __incrementLength(builder);
        builder.params = abi.encodePacked(builder.params, DATA_TYPE_BIGINT_VARIANT, param);
    }

    function __addVarChar(Builder memory builder, string memory param) external pure {
        __incrementLength(builder);
        builder.params = abi.encodePacked(builder.params, DATA_TYPE_VARCHAR_VARIANT, param);
    }

    function __addDecimal75(Builder memory builder, uint256 param, uint8 precision, int8 scale) external pure {
        __incrementLength(builder);
        builder.params = abi.encodePacked(builder.params, DATA_TYPE_DECIMAL75_VARIANT, precision, scale, param);
    }

    function __addTimeStamp(Builder memory builder, int64 param) external pure {
        __incrementLength(builder);
        builder.params = abi.encodePacked(builder.params, DATA_TYPE_TIMESTAMP_VARIANT, uint32(1), int32(0), param);
    }

    function __addScalar(Builder memory builder, uint256 param) external pure {
        __incrementLength(builder);
        builder.params = abi.encodePacked(builder.params, DATA_TYPE_SCALAR_VARIANT, param);
    }

    function __addVarBinary(Builder memory builder, bytes memory param) external pure {
        __incrementLength(builder);
        builder.params = abi.encodePacked(builder.params, DATA_TYPE_SCALAR_VARIANT, param);
    }
}
