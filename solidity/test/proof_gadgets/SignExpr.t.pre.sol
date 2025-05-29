// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {Errors} from "../../src/base/Errors.sol";
import {VerificationBuilder} from "../../src/builder/VerificationBuilder.pre.sol";
import {SignExpr} from "../../src/proof_gadgets/SignExpr.pre.sol";
import {F} from "../base/FieldUtil.sol";

contract SignExprTest is Test {
    function testSimpleSignExpr() public pure {
        VerificationBuilder.Builder memory builder;
        uint256[] memory bitDistribution = new uint256[](2);
        bitDistribution[0] = 0x800000000000000000000000000000000000000000000000000000000000007D;
        bitDistribution[1] = 0x8000000000000000000000000000000000000000000000000000000000000002;
        VerificationBuilder.__setBitDistributions(builder, bitDistribution);
        builder.maxDegree = 3;
        builder.constraintMultipliers = new uint256[](7);
        builder.constraintMultipliers[0] = 5;
        builder.constraintMultipliers[1] = 5;
        builder.constraintMultipliers[2] = 5;
        builder.constraintMultipliers[3] = 5;
        builder.constraintMultipliers[4] = 5;
        builder.constraintMultipliers[5] = 5;
        builder.constraintMultipliers[6] = 5;
        builder.aggregateEvaluation = 0;
        builder.rowMultipliersEvaluation = addmod(MODULUS, mulmod(MODULUS_MINUS_ONE, 2, MODULUS), MODULUS);

        int64[4] memory evaluationVector = [int64(700), -6, 3007, 134562844];

        int64[4][10] memory vectorsToEvaluate = [
            [int64(106), 23, -60, -76],
            [int64(1), 1, 1, 1],
            [int64(0), 1, 0, 0],
            [int64(0), 1, 1, 1],
            [int64(1), 0, 0, 0],
            [int64(0), 1, 0, 1],
            [int64(1), 0, 0, 1],
            [int64(1), 0, 1, 0],
            [int64(1), 1, 0, 0],
            [int64(1), 1, 0, 0]
        ];

        uint256[] memory evaluations = new uint256[](10);

        for (uint8 i = 0; i < 10; ++i) {
            int64 evaluation = 0;
            for (uint8 j = 0; j < 4; ++j) {
                evaluation += evaluationVector[j] * vectorsToEvaluate[i][j];
            }
            evaluations[i] = F.from(evaluation).into();
        }

        uint256[] memory finalRoundMles = new uint256[](7);
        for (uint8 i = 2; i < 9; ++i) {
            finalRoundMles[i - 2] = evaluations[i];
        }

        VerificationBuilder.__setFinalRoundMLEs(builder, finalRoundMles);

        uint256 signEval;
        (builder, signEval) = SignExpr.__signExprEvaluate(evaluations[0], builder, evaluations[1]);
        assert(signEval == evaluations[9]);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testIncorrectBitDecomposition() public {
        VerificationBuilder.Builder memory builder;
        uint256[] memory bitDistribution = new uint256[](2);
        bitDistribution[0] = 0x800000000000000000000000000000000000000000000000000000000000007D;
        bitDistribution[1] = 0x8000000000000000000000000000000000000000000000000000000000000002;
        VerificationBuilder.__setBitDistributions(builder, bitDistribution);
        builder.maxDegree = 3;
        builder.constraintMultipliers = new uint256[](7);
        builder.constraintMultipliers[0] = 5;
        builder.constraintMultipliers[1] = 5;
        builder.constraintMultipliers[2] = 5;
        builder.constraintMultipliers[3] = 5;
        builder.constraintMultipliers[4] = 5;
        builder.constraintMultipliers[5] = 5;
        builder.constraintMultipliers[6] = 5;
        builder.aggregateEvaluation = 0;
        builder.rowMultipliersEvaluation = addmod(MODULUS, mulmod(MODULUS_MINUS_ONE, 2, MODULUS), MODULUS);

        int64[4] memory evaluationVector = [int64(700), -6, 3007, 134562844];

        int64[4][10] memory vectorsToEvaluate = [
            [int64(106), 23, -60, -76],
            [int64(1), 1, 1, 1],
            [int64(0), 2, 0, 0],
            [int64(0), 1, 1, 1],
            [int64(1), 0, 0, 0],
            [int64(0), 1, 0, 1],
            [int64(1), 0, 0, 1],
            [int64(1), 0, 1, 0],
            [int64(1), 1, 0, 0],
            [int64(1), 1, 0, 0]
        ];

        uint256[] memory evaluations = new uint256[](10);

        for (uint8 i = 0; i < 10; ++i) {
            int64 evaluation = 0;
            for (uint8 j = 0; j < 4; ++j) {
                evaluation += evaluationVector[j] * vectorsToEvaluate[i][j];
            }
            evaluations[i] = F.from(evaluation).into();
        }

        uint256[] memory finalRoundMles = new uint256[](7);
        for (uint8 i = 2; i < 9; ++i) {
            finalRoundMles[i - 2] = evaluations[i];
        }

        VerificationBuilder.__setFinalRoundMLEs(builder, finalRoundMles);

        vm.expectRevert(Errors.BitDecompositionInvalid.selector);
        SignExpr.__signExprEvaluate(evaluations[0], builder, evaluations[1]);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testBitDecompositionWithInvalidBits() public {
        VerificationBuilder.Builder memory builder;
        uint256[] memory bitDistribution = new uint256[](2);
        bitDistribution[0] = 0xFF00000000000000000000000000000000000000000000000000000000000000;
        bitDistribution[1] = 0x8000000000000000000000000000000000000000000000000000000000000002;
        VerificationBuilder.__setBitDistributions(builder, bitDistribution);
        builder.maxDegree = 3;
        builder.constraintMultipliers = new uint256[](7);
        builder.constraintMultipliers[0] = 5;
        builder.constraintMultipliers[1] = 5;
        builder.constraintMultipliers[2] = 5;
        builder.constraintMultipliers[3] = 5;
        builder.constraintMultipliers[4] = 5;
        builder.constraintMultipliers[5] = 5;
        builder.constraintMultipliers[6] = 5;
        builder.aggregateEvaluation = 0;
        builder.rowMultipliersEvaluation = addmod(MODULUS, mulmod(MODULUS_MINUS_ONE, 2, MODULUS), MODULUS);

        int64[4] memory evaluationVector = [int64(700), -6, 3007, 134562844];

        int64[4][10] memory vectorsToEvaluate = [
            [int64(106), 23, -60, -76],
            [int64(1), 1, 1, 1],
            [int64(0), 2, 0, 0],
            [int64(0), 1, 1, 1],
            [int64(1), 0, 0, 0],
            [int64(0), 1, 0, 1],
            [int64(1), 0, 0, 1],
            [int64(1), 0, 1, 0],
            [int64(1), 1, 0, 0],
            [int64(1), 1, 0, 0]
        ];

        uint256[] memory evaluations = new uint256[](10);

        for (uint8 i = 0; i < 10; ++i) {
            int64 evaluation = 0;
            for (uint8 j = 0; j < 4; ++j) {
                evaluation += evaluationVector[j] * vectorsToEvaluate[i][j];
            }
            evaluations[i] = F.from(evaluation).into();
        }

        uint256[] memory finalRoundMles = new uint256[](7);
        for (uint8 i = 2; i < 9; ++i) {
            finalRoundMles[i - 2] = evaluations[i];
        }

        VerificationBuilder.__setFinalRoundMLEs(builder, finalRoundMles);

        vm.expectRevert(Errors.InvalidVaryingBits.selector);
        SignExpr.__signExprEvaluate(evaluations[0], builder, evaluations[1]);
    }
}
