// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {PlanUtil} from "../../src/verifier/PlanUtil.pre.sol";

contract PlanUtilTest is Test {
    /* solhint-disable gas-struct-packing */
    struct ColumnMetadata {
        uint64 tableIndex;
        uint32 columnVariant;
        bytes name;
        uint8 precision;
        int8 scale;
        uint32 timeunit;
        int32 timezone;
    }

    function generatePlanPrefix(
        bytes[] memory tableNames,
        ColumnMetadata[] memory columns,
        bytes[] memory outputColumnNames
    ) public pure returns (bytes memory result) {
        uint64 numberOfTables = uint64(tableNames.length);
        result = abi.encodePacked(numberOfTables);
        for (uint256 i = 0; i < numberOfTables; ++i) {
            result = bytes.concat(result, abi.encodePacked(uint64(tableNames[i].length), tableNames[i]));
        }
        uint64 numberOfColumns = uint64(columns.length);
        result = bytes.concat(result, abi.encodePacked(numberOfColumns));
        for (uint256 i = 0; i < numberOfColumns; ++i) {
            result = bytes.concat(
                result,
                abi.encodePacked(
                    columns[i].tableIndex, uint64(columns[i].name.length), columns[i].name, columns[i].columnVariant
                )
            );

            // Add additional metadata based on column variant
            if (columns[i].columnVariant == DATA_TYPE_DECIMAL75_VARIANT) {
                result = bytes.concat(result, abi.encodePacked(columns[i].precision, columns[i].scale));
            } else if (columns[i].columnVariant == DATA_TYPE_TIMESTAMP_VARIANT) {
                result = bytes.concat(result, abi.encodePacked(columns[i].timeunit, columns[i].timezone));
            }
        }
        uint64 numberOfOutputColumns = uint64(outputColumnNames.length);
        result = bytes.concat(result, abi.encodePacked(numberOfOutputColumns));
        for (uint256 i = 0; i < numberOfOutputColumns; ++i) {
            result = bytes.concat(result, abi.encodePacked(uint64(outputColumnNames[i].length), outputColumnNames[i]));
        }
    }

    function testSkipSimplePlanPrefix() public pure {
        bytes[] memory tableNames = new bytes[](4);
        tableNames[0] = "A";
        tableNames[1] = "B2";
        tableNames[2] = "Decimal";
        tableNames[3] = "Timestamp";
        ColumnMetadata[] memory columns = new ColumnMetadata[](4);
        // Standard bigint type
        columns[0] = ColumnMetadata({
            tableIndex: 0,
            columnVariant: 5,
            name: "A",
            precision: 0,
            scale: 0,
            timeunit: 0,
            timezone: 0
        });
        // Standard bigint type
        columns[1] = ColumnMetadata({
            tableIndex: 1,
            columnVariant: 5,
            name: "B2",
            precision: 0,
            scale: 0,
            timeunit: 0,
            timezone: 0
        });
        // Decimal type with precision and scale
        columns[2] = ColumnMetadata({
            tableIndex: 0,
            columnVariant: 8,
            name: "Decimal",
            precision: 10,
            scale: -2,
            timeunit: 0,
            timezone: 0
        });
        // Timestamp type with timeunit and timezone
        columns[3] = ColumnMetadata({
            tableIndex: 1,
            columnVariant: 9,
            name: "Timestamp",
            precision: 0,
            scale: 0,
            timeunit: 3,
            timezone: -18000
        });

        bytes[] memory outputColumnNames = new bytes[](2);
        outputColumnNames[0] = "A";
        outputColumnNames[1] = "B2";
        bytes memory planPrefix = generatePlanPrefix(tableNames, columns, outputColumnNames);
        bytes memory planPostfix = hex"abcdef";
        bytes memory plan = bytes.concat(planPrefix, planPostfix);
        bytes memory resultingPlan = PlanUtil.__skipPlanNames(plan);
        assertEq(resultingPlan.length, planPostfix.length);
        uint256 length = resultingPlan.length;
        for (uint256 i = 0; i < length; ++i) {
            assertEq(resultingPlan[i], planPostfix[i]);
        }
    }

    function testFuzzSkipPlanPrefix(
        bytes[] memory tableNames,
        ColumnMetadata[] memory columns,
        bytes[] memory outputColumnNames,
        bytes memory planPostfix
    ) public pure {
        // scan for invalid variants
        uint256 numColumns = columns.length;
        for (uint256 i = 0; i < numColumns; ++i) {
            uint32 v = columns[i].columnVariant;
            if (v > 9 || v == 1 || v == 6 || v == 7) {
                return; // 💤 silently succeed and move on
            }
        }
        bytes memory planPrefix = generatePlanPrefix(tableNames, columns, outputColumnNames);
        bytes memory plan = bytes.concat(planPrefix, planPostfix);
        bytes memory resultingPlan = PlanUtil.__skipPlanNames(plan);
        assertEq(resultingPlan.length, planPostfix.length);
        uint256 length = resultingPlan.length;
        for (uint256 i = 0; i < length; ++i) {
            assertEq(resultingPlan[i], planPostfix[i]);
        }
    }
}
