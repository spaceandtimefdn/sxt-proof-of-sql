// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "forge-std/Test.sol";
import {VerificationBuilder} from "../../src/builder/VerificationBuilder.pre.sol";
import "../../src/proof_exprs/PlaceholderExpr.pre.sol";

/// @title PlaceholderExprTest
/// @dev Test contract for PlaceholderExpr library
contract PlaceholderExprTest is Test {
    /// @dev Tests that a placeholder expression is properly parsed and evaluates using placeholder parameters
    function testPlaceholderExprEvaluate() public pure {
        // Encode a placeholder expression: index=5 (uint64), column_type=4 (INT, uint32)
        bytes memory expr = abi.encodePacked(uint64(5), uint32(4));
        uint256 chiEval = 12345; // Arbitrary chi value

        // Create builder with placeholder parameters
        VerificationBuilder.Builder memory builder;
        uint256[] memory placeholderParams = new uint256[](10);
        placeholderParams[5] = 42; // Set parameter for index 5
        builder.placeholderParameters = placeholderParams;

        uint256[] memory values = new uint256[](10);
        builder.tableChiEvaluations = values;

        (bytes memory exprOut,, uint256 eval) = PlaceholderExpr.__placeholderExprEvaluate(expr, builder, chiEval);

        // Placeholder evaluation should return parameter * chi_eval
        assertEq(eval, (42 * chiEval) % 21888242871839275222246405745257275088548364400416034343698204186575808495617);

        // Expression should be fully consumed (empty output)
        assertEq(exprOut.length, 0);
    }

    /// @dev Tests placeholder expression with different index and type values
    function testPlaceholderExprEvaluateVariousValues() public pure {
        // Create builder with placeholder parameters
        VerificationBuilder.Builder memory builder = VerificationBuilder.__builderNew();
        uint256[] memory placeholderParams = new uint256[](101);
        placeholderParams[100] = 999;
        placeholderParams[42] = 123;
        VerificationBuilder.__setPlaceholderParameters(builder, placeholderParams);

        // Test with index=100, column_type=0 (BOOLEAN)
        bytes memory expr1 = abi.encodePacked(uint64(100), uint32(0));
        (bytes memory exprOut1,, uint256 eval1) = PlaceholderExpr.__placeholderExprEvaluate(expr1, builder, 1);
        assertEq(eval1, 999); // parameter * chi_eval (999 * 1)
        assertEq(exprOut1.length, 0);

        // Test with index=42, column_type=5 (BIGINT)
        bytes memory expr2 = abi.encodePacked(uint64(42), uint32(5));
        (bytes memory exprOut2,, uint256 eval2) = PlaceholderExpr.__placeholderExprEvaluate(expr2, builder, 2);
        assertEq(eval2, (123 * 2) % 21888242871839275222246405745257275088548364400416034343698204186575808495617);
        assertEq(exprOut2.length, 0);
    }

    /// @dev Tests placeholder expression with additional data after
    function testPlaceholderExprEvaluateWithRemainingData() public pure {
        // Create builder with placeholder parameters
        VerificationBuilder.Builder memory builder = VerificationBuilder.__builderNew();
        uint256[] memory placeholderParams = new uint256[](11);
        placeholderParams[10] = 555;
        VerificationBuilder.__setPlaceholderParameters(builder, placeholderParams);

        // Encode placeholder expression followed by additional data
        bytes memory expr = abi.encodePacked(
            uint64(10), // placeholder index
            uint32(7), // VARCHAR column type
            hex"deadbeef" // additional data that should remain
        );
        uint256 chiEval = 1;

        (bytes memory exprOut,, uint256 eval) = PlaceholderExpr.__placeholderExprEvaluate(expr, builder, chiEval);

        // Evaluation should return parameter * chi_eval
        assertEq(eval, 555);

        // Remaining data should be preserved
        assertEq(exprOut.length, 4);
        assertEq(bytes4(exprOut), hex"deadbeef");
    }
}
