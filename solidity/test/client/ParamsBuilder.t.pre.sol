// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {ParamsBuilder} from "../../src/client/ParamsBuilder.pre.sol";

function areByteArraysEqual(bytes memory left, bytes memory right) pure returns (bool areEqual) {
    uint256 leftLength = left.length;
    areEqual = leftLength == right.length;
    for (uint8 i = 0; i < leftLength; ++i) {
        areEqual = areEqual && (left[i] == right[i]);
    }
}

contract ParamsBuilderTest is Test {
    function testSerializationOfAllSupportedTypes() public pure {
        bytes memory expectedBytes = hex"000000000000000a000000000100000002020000000300030000000400000004000000050000000000000005000000060000000000000001360000000a0000000000000007000000000000000000000000000000000000000000000000000000090a000000000000000008000000000000000000000000000000000000000000000000000000070000000000000001090000000b0000000100000000000000000000000a";
        ParamsBuilder.Builder memory builder = ParamsBuilder.__builderNew();
        builder = ParamsBuilder.__addBool(builder, true);
        builder = ParamsBuilder.__addTinyInt(builder, 2);
        builder = ParamsBuilder.__addSmallInt(builder, 3);
        builder = ParamsBuilder.__addInt(builder, 4);
        builder = ParamsBuilder.__addBigInt(builder, 5);
        builder = ParamsBuilder.__addVarChar(builder, "6");
        builder = ParamsBuilder.__addScalar(builder, 7);
        builder = ParamsBuilder.__addDecimal75(builder, 8, 10, 0);
        builder = ParamsBuilder.__addVarBinary(builder, hex"09");
        builder = ParamsBuilder.__addTimeStamp(builder, 10);
        bytes memory actualBytes = ParamsBuilder.__buildParams(builder);
        assert(areByteArraysEqual(actualBytes, expectedBytes));
    }
}
