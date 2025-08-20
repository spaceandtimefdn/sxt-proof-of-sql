// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {ParamsBuilder} from "../../src/client/ParamsBuilder.pre.sol";

function _areByteArraysEqual(bytes memory left, bytes memory right) pure returns (bool areEqual) {
    uint256 leftLength = left.length;
    areEqual = leftLength == right.length;
    for (uint8 i = 0; i < leftLength; ++i) {
        areEqual = areEqual && (left[i] == right[i]);
    }
}

contract ParamsBuilderTest is Test {
    function testSerializationOfAllSupportedTypes() public pure {
        bytes memory expectedBytes =
            hex"000000000000000a000000000100000002020000000300030000000400000004000000050000000000000005000000070000000000000001360000000a0000000000000000000000000000000000000000000000000000000000000007000000080a0000000000000000000000000000000000000000000000000000000000000000080000000b000000000000000109000000090000000100000000000000000000000a";
        ParamsBuilder.Builder memory builder;
        builder = ParamsBuilder.addBool(builder, true);
        builder = ParamsBuilder.addTinyInt(builder, 2);
        builder = ParamsBuilder.addSmallInt(builder, 3);
        builder = ParamsBuilder.addInt(builder, 4);
        builder = ParamsBuilder.addBigInt(builder, 5);
        builder = ParamsBuilder.addVarChar(builder, "6");
        builder = ParamsBuilder.addScalar(builder, 7);
        builder = ParamsBuilder.addDecimal75(builder, 8, 10, 0);
        builder = ParamsBuilder.addVarBinary(builder, bytes("\x09"));
        builder = ParamsBuilder.addTimeStamp(builder, 10);
        (bytes memory actualBytes, uint256[] memory actualParams) = ParamsBuilder.buildParams(builder);
        assert(_areByteArraysEqual(actualBytes, expectedBytes));
        assert(actualParams.length == 10);
        assert(actualParams[0] == 1);
        assert(actualParams[1] == 2);
        assert(actualParams[2] == 3);
        assert(actualParams[3] == 4);
        assert(actualParams[4] == 5);
        assert(actualParams[5] == 6018613808072455048935921990747708200856747868835246493831327293258478867940);
        assert(actualParams[6] == 7);
        assert(actualParams[7] == 8);
        assert(actualParams[8] == 14368806583393397743267686700701231208279041777806019220663728442259589818290);
        assert(actualParams[9] == 10);
    }
}
