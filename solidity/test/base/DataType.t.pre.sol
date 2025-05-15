// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "forge-std/Test.sol";
import "../../src/base/DataType.pre.sol";
import "../base/Constants.t.sol";
import {F} from "../base/FieldUtil.sol";

contract DataTypeTest is Test {
    function testReadTrueBooleanEntryExpr() public pure {
        bytes memory exprIn = abi.encodePacked(uint8(1), hex"abcdef");
        bytes memory expectedExprOut = hex"abcdef";
        (bytes memory exprOut, uint256 entry) = DataType.__readEntry(exprIn, DATA_TYPE_BOOLEAN_VARIANT);
        assert(entry == 1);
        assert(exprOut.length == expectedExprOut.length);
        uint256 exprOutLength = exprOut.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(exprOut[i] == expectedExprOut[i]);
        }
    }

    function testReadFalseBooleanEntryExpr() public pure {
        bytes memory exprIn = abi.encodePacked(uint8(0), hex"abcdef");
        bytes memory expectedExprOut = hex"abcdef";
        (bytes memory exprOut, uint256 entry) = DataType.__readEntry(exprIn, DATA_TYPE_BOOLEAN_VARIANT);
        assert(entry == 0);
        assert(exprOut.length == expectedExprOut.length);
        uint256 exprOutLength = exprOut.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(exprOut[i] == expectedExprOut[i]);
        }
    }

    function testReadInvalidBooleanEntryExpr() public {
        bytes memory exprIn = abi.encodePacked(uint8(2), hex"abcdef");
        vm.expectRevert(Errors.InvalidBoolean.selector);
        DataType.__readEntry(exprIn, DATA_TYPE_BOOLEAN_VARIANT);
    }

    function testReadNonnegativeInt8EntryExpr() public pure {
        bytes memory exprIn = abi.encodePacked(int8(127), hex"abcdef");
        bytes memory expectedExprOut = hex"abcdef";
        (bytes memory exprOut, uint256 entry) = DataType.__readEntry(exprIn, DATA_TYPE_TINYINT_VARIANT);
        assert(entry == 127);
        assert(exprOut.length == expectedExprOut.length);
        uint256 exprOutLength = exprOut.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(exprOut[i] == expectedExprOut[i]);
        }
    }

    function testReadNegativeInt8EntryExpr() public pure {
        bytes memory exprIn = abi.encodePacked(int8(-128), hex"abcdef");
        bytes memory expectedExprOut = hex"abcdef";
        (bytes memory exprOut, uint256 entry) = DataType.__readEntry(exprIn, DATA_TYPE_TINYINT_VARIANT);
        assert(entry == MODULUS - 128);
        assert(exprOut.length == expectedExprOut.length);
        uint256 exprOutLength = exprOut.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(exprOut[i] == expectedExprOut[i]);
        }
    }

    function testReadNonnegativeInt16EntryExpr() public pure {
        bytes memory exprIn = abi.encodePacked(int16(32767), hex"abcdef");
        bytes memory expectedExprOut = hex"abcdef";
        (bytes memory exprOut, uint256 entry) = DataType.__readEntry(exprIn, DATA_TYPE_SMALLINT_VARIANT);
        assert(entry == 32767);
        assert(exprOut.length == expectedExprOut.length);
        uint256 exprOutLength = exprOut.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(exprOut[i] == expectedExprOut[i]);
        }
    }

    function testReadNegativeInt16EntryExpr() public pure {
        bytes memory exprIn = abi.encodePacked(int16(-32768), hex"abcdef");
        bytes memory expectedExprOut = hex"abcdef";
        (bytes memory exprOut, uint256 entry) = DataType.__readEntry(exprIn, DATA_TYPE_SMALLINT_VARIANT);
        assert(entry == MODULUS - 32768);
        assert(exprOut.length == expectedExprOut.length);
        uint256 exprOutLength = exprOut.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(exprOut[i] == expectedExprOut[i]);
        }
    }

    function testReadNonnegativeInt32EntryExpr() public pure {
        bytes memory exprIn = abi.encodePacked(int32(2147483647), hex"abcdef");
        bytes memory expectedExprOut = hex"abcdef";
        (bytes memory exprOut, uint256 entry) = DataType.__readEntry(exprIn, DATA_TYPE_INT_VARIANT);
        assert(entry == 2147483647);
        assert(exprOut.length == expectedExprOut.length);
        uint256 exprOutLength = exprOut.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(exprOut[i] == expectedExprOut[i]);
        }
    }

    function testReadNegativeInt32EntryExpr() public pure {
        bytes memory exprIn = abi.encodePacked(int32(-2147483648), hex"abcdef");
        bytes memory expectedExprOut = hex"abcdef";
        (bytes memory exprOut, uint256 entry) = DataType.__readEntry(exprIn, DATA_TYPE_INT_VARIANT);
        assert(entry == MODULUS - 2147483648);
        assert(exprOut.length == expectedExprOut.length);
        uint256 exprOutLength = exprOut.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(exprOut[i] == expectedExprOut[i]);
        }
    }

    function testReadNonnegativeInt64EntryExpr() public pure {
        bytes memory exprIn = abi.encodePacked(int64(9223372036854775807), hex"abcdef");
        bytes memory expectedExprOut = hex"abcdef";
        (bytes memory exprOut, uint256 entry) = DataType.__readEntry(exprIn, DATA_TYPE_BIGINT_VARIANT);
        assert(entry == 9223372036854775807);
        assert(exprOut.length == expectedExprOut.length);
        uint256 exprOutLength = exprOut.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(exprOut[i] == expectedExprOut[i]);
        }
    }

    function testReadNegativeInt64EntryExpr() public pure {
        bytes memory exprIn = abi.encodePacked(int64(-9223372036854775808), hex"abcdef");
        bytes memory expectedExprOut = hex"abcdef";
        (bytes memory exprOut, uint256 entry) = DataType.__readEntry(exprIn, DATA_TYPE_BIGINT_VARIANT);
        assert(entry == MODULUS - 9223372036854775808);
        assert(exprOut.length == expectedExprOut.length);
        uint256 exprOutLength = exprOut.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(exprOut[i] == expectedExprOut[i]);
        }
    }

    function testReadDecimal75EntryExpr() public pure {
        bytes memory exprIn = abi.encodePacked(MODULUS_MINUS_ONE, hex"abcdef");
        bytes memory expectedExprOut = hex"abcdef";
        (bytes memory exprOut, uint256 entry) = DataType.__readEntry(exprIn, DATA_TYPE_DECIMAL75_VARIANT);
        assert(entry == MODULUS_MINUS_ONE);
        assert(exprOut.length == expectedExprOut.length);
        uint256 exprOutLength = exprOut.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(exprOut[i] == expectedExprOut[i]);
        }
    }

    function testReadTimestampEntryExpr() public pure {
        bytes memory exprIn = abi.encodePacked(int64(1746627936), hex"abcdef");
        bytes memory expectedExprOut = hex"abcdef";
        (bytes memory exprOut, uint256 entry) = DataType.__readEntry(exprIn, DATA_TYPE_TIMESTAMP_VARIANT);
        assert(entry == 1746627936);
        assert(exprOut.length == expectedExprOut.length);
        uint256 exprOutLength = exprOut.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(exprOut[i] == expectedExprOut[i]);
        }
    }

    function testReadVarcharEntryExpr() public pure {
        bytes memory exprIn = abi.encodePacked(uint64(3), "sxt", hex"abcdef");
        bytes memory expectedExprOut = hex"abcdef";
        (bytes memory exprOut, uint256 entry) = DataType.__readEntry(exprIn, DATA_TYPE_VARCHAR_VARIANT);
        assert(bytes32(entry) == keccak256(bytes("sxt")) & bytes32(MODULUS_MASK));
        assert(exprOut.length == expectedExprOut.length);
        uint256 exprOutLength = exprOut.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(exprOut[i] == expectedExprOut[i]);
        }
    }

    function testReadNonAsciiVarcharEntryExpr() public pure {
        bytes memory exprIn = abi.encodePacked(uint64(8), unicode"😸🐾", hex"abcdef");
        bytes memory expectedExprOut = hex"abcdef";
        (bytes memory exprOut, uint256 entry) = DataType.__readEntry(exprIn, DATA_TYPE_VARCHAR_VARIANT);
        assert(bytes32(entry) == keccak256(bytes(unicode"😸🐾")) & bytes32(MODULUS_MASK));
        assert(exprOut.length == expectedExprOut.length);
        uint256 exprOutLength = exprOut.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(exprOut[i] == expectedExprOut[i]);
        }
    }

    function testFuzzReadVarcharEntryExpr(string memory literalValue, bytes memory trailingExpr) public pure {
        bytes memory exprIn = abi.encodePacked(uint64(bytes(literalValue).length), bytes(literalValue), trailingExpr);
        (bytes memory exprOut, uint256 entry) = DataType.__readEntry(exprIn, DATA_TYPE_VARCHAR_VARIANT);
        assert(bytes32(entry) == keccak256(bytes(literalValue)) & bytes32(MODULUS_MASK));
        assert(exprOut.length == trailingExpr.length);
        uint256 exprOutLength = exprOut.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(exprOut[i] == trailingExpr[i]);
        }
    }

    function testReadVarbinaryEntryExpr() public pure {
        bytes memory exprIn = abi.encodePacked(uint64(3), bytes("\x01\x02\x03"), hex"abcdef");
        bytes memory expectedExprOut = hex"abcdef";
        (bytes memory exprOut, uint256 entry) = DataType.__readEntry(exprIn, DATA_TYPE_VARBINARY_VARIANT);
        assert(bytes32(entry) == keccak256(bytes("\x01\x02\x03")) & bytes32(MODULUS_MASK));
        assert(exprOut.length == expectedExprOut.length);
        uint256 exprOutLength = exprOut.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(exprOut[i] == expectedExprOut[i]);
        }
    }

    function testFuzzReadVarbinaryEntryExpr(bytes memory literalValue, bytes memory trailingExpr) public pure {
        bytes memory exprIn = abi.encodePacked(uint64(literalValue.length), literalValue, trailingExpr);
        (bytes memory exprOut, uint256 entry) = DataType.__readEntry(exprIn, DATA_TYPE_VARBINARY_VARIANT);
        assert(bytes32(entry) == keccak256(literalValue) & bytes32(MODULUS_MASK));
        assert(exprOut.length == trailingExpr.length);
        uint256 exprOutLength = exprOut.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(exprOut[i] == trailingExpr[i]);
        }
    }

    function testFuzzReadEntryExpr(int64 literalValue, bytes memory trailingExpr) public pure {
        bytes memory exprIn = abi.encodePacked(literalValue, trailingExpr);
        (bytes memory exprOut, uint256 entry) = DataType.__readEntry(exprIn, DATA_TYPE_BIGINT_VARIANT);
        uint256 expected = F.from(literalValue).into();
        assert(entry == expected);
        assert(exprOut.length == trailingExpr.length);
        uint256 exprOutLength = exprOut.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(exprOut[i] == trailingExpr[i]);
        }
    }

    function testReadEntryWithInvalidVariant() public {
        bytes memory exprIn = abi.encodePacked(int64(9223372036854775807), hex"abcdef");
        vm.expectRevert(Errors.UnsupportedDataTypeVariant.selector);
        DataType.__readEntry(exprIn, INVALID_VARIANT);
    }

    function testReadFuzzSimpleDataType(uint32 dataType) public pure {
        vm.assume((dataType < 6 && dataType != 1) || dataType == 7 || dataType == 11);
        bytes memory exprIn = abi.encodePacked(dataType, hex"abcdef");
        bytes memory expectedExprOut = hex"abcdef";
        (bytes memory exprOut, uint32 actualDataType) = DataType.__readDataType(exprIn);
        assert(dataType == actualDataType);
        assert(exprOut.length == expectedExprOut.length);
        uint256 exprOutLength = exprOut.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(exprOut[i] == expectedExprOut[i]);
        }
    }

    function testReadDecimal75DataType() public pure {
        bytes memory exprIn = abi.encodePacked(DATA_TYPE_DECIMAL75_VARIANT, uint8(75), int8(10), hex"abcdef");
        bytes memory expectedExprOut = hex"abcdef";
        (bytes memory exprOut, uint32 dataType) = DataType.__readDataType(exprIn);
        assert(dataType == DATA_TYPE_DECIMAL75_VARIANT);
        assert(exprOut.length == expectedExprOut.length);
        uint256 exprOutLength = exprOut.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(exprOut[i] == expectedExprOut[i]);
        }
    }

    function testReadTimestampDataType() public pure {
        bytes memory exprIn = abi.encodePacked(DATA_TYPE_TIMESTAMP_VARIANT, uint32(1), int32(0), hex"abcdef");
        bytes memory expectedExprOut = hex"abcdef";
        (bytes memory exprOut, uint32 dataType) = DataType.__readDataType(exprIn);
        assert(dataType == DATA_TYPE_TIMESTAMP_VARIANT);
        assert(exprOut.length == expectedExprOut.length);
        uint256 exprOutLength = exprOut.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(exprOut[i] == expectedExprOut[i]);
        }
    }

    function testReadDataTypeWithInvalidVariant() public {
        bytes memory exprIn = abi.encodePacked(INVALID_VARIANT, hex"abcdef");
        vm.expectRevert(Errors.UnsupportedDataTypeVariant.selector);
        DataType.__readDataType(exprIn);
    }
}
