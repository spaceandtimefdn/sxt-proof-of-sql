// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {VerificationBuilder} from "../../src/builder/VerificationBuilder.pre.sol";
import {FoldLogExpr} from "../../src/proof_gadgets/FoldLogExpr.pre.sol";

contract FoldLogExprTest is Test {
    function testSimpleFoldLogExprEvalsWithMultipleColumn() public pure {
        VerificationBuilder.Builder memory builder;
        builder.maxDegree = 3;
        builder.rowMultipliersEvaluation = 1;
        uint256 alpha = 3;
        uint256 beta = 8;
        uint256[2][8] memory column = [
            [uint256(300), 2],
            [uint256(1), 1],
            [uint256(MODULUS_MINUS_ONE), 3],
            [uint256(1), 1],
            [uint256(MODULUS_MINUS_ONE), 3],
            [uint256(1), 1],
            [uint256(5), 0],
            [uint256(2), 1]
        ];
        builder.finalRoundMLEs = new uint256[](8);
        {
            uint256 inv7207 = 5767416846624501685451078744310193616366497016288337618798791418108430738612;
            uint256 inv28 = 11725844395628183154774860220673540226008052357365732684124037957094183122652;
            uint256 invNegative14 = 20324796952422184134943091049167469725080624086100603319148332458963250745930;
            uint256 inv121 = 18632140626441697090011403237698341604301500274734310226453843233200894835112;
            uint256 inv52 = 21467315124303904544895513327079250567614742008100341375550161798372427563009;
            uint256[8] memory star = [inv7207, inv28, invNegative14, inv28, invNegative14, inv28, inv121, inv52];
            for (uint8 i = 0; i < 8; ++i) {
                builder.finalRoundMLEs[i] = star[i];
            }
        }

        builder.constraintMultipliers = new uint256[](24);
        for (uint8 i = 0; i < 8; ++i) {
            builder.constraintMultipliers[i] = 1;
        }

        for (uint8 i = 0; i < 8; ++i) {
            uint256[] memory columnEval = new uint256[](2);
            columnEval[0] = column[i][0];
            columnEval[1] = column[i][1];
            uint256 actualStar;
            (builder, actualStar) = FoldLogExpr.__foldLogStarEvaluate({
                __builder: builder,
                __alpha: alpha,
                __beta: beta,
                __columnEvals: columnEval,
                __chiEval: 1
            });
        }
        assert(builder.aggregateEvaluation == 0);
    }

    function testSimpleFoldLogExprEvalsWithMultipleColumnExprs() public pure {
        bytes memory plan = abi.encodePacked(uint64(0), uint64(1), hex"abcdef");
        VerificationBuilder.Builder memory builder;
        builder.maxDegree = 3;
        builder.rowMultipliersEvaluation = 1;
        uint256[2][8] memory column = [
            [uint256(300), 2],
            [uint256(1), 1],
            [uint256(MODULUS_MINUS_ONE), 3],
            [uint256(1), 1],
            [uint256(MODULUS_MINUS_ONE), 3],
            [uint256(1), 1],
            [uint256(5), 0],
            [uint256(2), 1]
        ];
        builder.finalRoundMLEs = new uint256[](8);
        {
            uint256 inv7207 = 5767416846624501685451078744310193616366497016288337618798791418108430738612;
            uint256 inv28 = 11725844395628183154774860220673540226008052357365732684124037957094183122652;
            uint256 invNegative14 = 20324796952422184134943091049167469725080624086100603319148332458963250745930;
            uint256 inv121 = 18632140626441697090011403237698341604301500274734310226453843233200894835112;
            uint256 inv52 = 21467315124303904544895513327079250567614742008100341375550161798372427563009;
            uint256[8] memory star = [inv7207, inv28, invNegative14, inv28, invNegative14, inv28, inv121, inv52];
            for (uint8 i = 0; i < 8; ++i) {
                builder.finalRoundMLEs[i] = star[i];
            }
        }

        builder.constraintMultipliers = new uint256[](24);
        for (uint8 i = 0; i < 8; ++i) {
            builder.constraintMultipliers[i] = 1;
        }

        for (uint8 i = 0; i < 8; ++i) {
            uint256[] memory columnEval = new uint256[](2);
            columnEval[0] = column[i][0];
            columnEval[1] = column[i][1];
            builder.columnEvaluations = columnEval;
            uint256 actualStar;
            (builder, actualStar) = FoldLogExpr.__foldLogStarEvaluateFromColumnExprs({
                __plan: plan,
                __builder: builder,
                __alpha: 3,
                __beta: 8,
                __columnCount: 2,
                __chiEval: 1
            });
        }
        assert(builder.aggregateEvaluation == 0);
    }

    function testSimpleFoldLogExprEvalsWithMultipleColumnMLEs() public pure {
        VerificationBuilder.Builder memory builder;
        builder.maxDegree = 3;
        builder.rowMultipliersEvaluation = 1;

        builder.finalRoundMLEs = new uint256[](8);
        {
            uint256 inv7207 = 5767416846624501685451078744310193616366497016288337618798791418108430738612;
            uint256 inv28 = 11725844395628183154774860220673540226008052357365732684124037957094183122652;
            uint256 invNegative14 = 20324796952422184134943091049167469725080624086100603319148332458963250745930;
            uint256 inv121 = 18632140626441697090011403237698341604301500274734310226453843233200894835112;
            uint256 inv52 = 21467315124303904544895513327079250567614742008100341375550161798372427563009;
            uint256[8] memory star = [inv7207, inv28, invNegative14, inv28, invNegative14, inv28, inv121, inv52];
            for (uint8 i = 0; i < 8; ++i) {
                builder.finalRoundMLEs[i] = star[i];
            }
        }
        builder.firstRoundMLEs = new uint256[](16);
        {
            uint256[2][8] memory firstRoundMLEs = [
                [uint256(300), 2],
                [uint256(1), 1],
                [uint256(MODULUS_MINUS_ONE), 3],
                [uint256(1), 1],
                [uint256(MODULUS_MINUS_ONE), 3],
                [uint256(1), 1],
                [uint256(5), 0],
                [uint256(2), 1]
            ];
            for (uint8 i = 0; i < 8; ++i) {
                builder.firstRoundMLEs[2 * i] = firstRoundMLEs[i][0];
                builder.firstRoundMLEs[2 * i + 1] = firstRoundMLEs[i][1];
            }
        }

        builder.constraintMultipliers = new uint256[](24);
        for (uint8 i = 0; i < 8; ++i) {
            builder.constraintMultipliers[i] = 1;
        }

        for (uint8 i = 0; i < 8; ++i) {
            uint256[] memory evaluations;
            uint256 actualStar;
            (builder, actualStar, evaluations) = FoldLogExpr.__foldLogStarEvaluateFromMLEs({
                __builder: builder,
                __alpha: 3,
                __beta: 8,
                __columnCount: 2,
                __chiEval: 1
            });
        }
        assert(builder.aggregateEvaluation == 0);
    }

    function testFoldExprEvalsWithMultipleColumnsProofExprs() public pure {
        bytes memory plan = abi.encodePacked(COLUMN_EXPR_VARIANT, uint64(0), COLUMN_EXPR_VARIANT, uint64(1));
        VerificationBuilder.Builder memory builder;
        builder.maxDegree = 3;
        builder.rowMultipliersEvaluation = 1;

        builder.finalRoundMLEs = new uint256[](8);
        {
            uint256 inv7207 = 5767416846624501685451078744310193616366497016288337618798791418108430738612;
            uint256 inv28 = 11725844395628183154774860220673540226008052357365732684124037957094183122652;
            uint256 invNegative14 = 20324796952422184134943091049167469725080624086100603319148332458963250745930;
            uint256 inv121 = 18632140626441697090011403237698341604301500274734310226453843233200894835112;
            uint256 inv52 = 21467315124303904544895513327079250567614742008100341375550161798372427563009;
            uint256[8] memory star = [inv7207, inv28, invNegative14, inv28, invNegative14, inv28, inv121, inv52];
            for (uint8 i = 0; i < 8; ++i) {
                builder.finalRoundMLEs[i] = star[i];
            }
        }

        builder.constraintMultipliers = new uint256[](8);
        for (uint8 i = 0; i < 8; ++i) {
            builder.constraintMultipliers[i] = 1;
        }

        uint256[2][8] memory columnEvals = [
            [uint256(300), 2],
            [uint256(1), 1],
            [uint256(MODULUS_MINUS_ONE), 3],
            [uint256(1), 1],
            [uint256(MODULUS_MINUS_ONE), 3],
            [uint256(1), 1],
            [uint256(5), 0],
            [uint256(2), 1]
        ];
        builder.columnEvaluations = new uint256[](2);
        for (uint8 i = 0; i < 8; ++i) {
            builder.columnEvaluations[0] = columnEvals[i][0];
            builder.columnEvaluations[1] = columnEvals[i][1];
            uint256 actualStar;
            (builder, actualStar) = FoldLogExpr.__foldLogStarEvaluateFromExprEvals({
                __plan: plan,
                __builder: builder,
                __inputChiEval: 1,
                __alpha: 3,
                __beta: 8,
                __columnCount: 2
            });
        }
        assert(builder.aggregateEvaluation == 0);
    }
}
