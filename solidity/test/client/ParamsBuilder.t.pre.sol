// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {ParamsBuilder} from "../../src/client/ParamsBuilder.pre.sol";

contract ParamsBuilderTest is Test {
    function testSerializationOfAllSupportedTypes() public pure {
        ParamsBuilder.Builder memory builder = ParamsBuilder.__builderNew();
        ParamsBuilder.__addBool(builder, true);
        ParamsBuilder.__addTinyInt(builder, 2);
        ParamsBuilder.__addSmallInt(builder, 3);
        ParamsBuilder.__addInt(builder, 4);
        ParamsBuilder.__addBigInt(builder, 5);
        ParamsBuilder.__addVarChar(builder, "6");
        ParamsBuilder.__addScalar(builder, 7);
        ParamsBuilder.__addDecimal75(builder, 8, 10, 0);
        ParamsBuilder.__addVarBinary(builder, hex"09");
        ParamsBuilder.__addTimeStamp(builder, 10);
    }
}
