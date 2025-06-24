// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {Errors} from "../../src/base/Errors.sol";
import {VerificationBuilder} from "../../src/builder/VerificationBuilder.pre.sol";
import {ProofExpr} from "../../src/proof_exprs/ProofExpr.pre.sol";
import {F} from "../base/FieldUtil.sol";

contract ProofExprTest is Test {
    function testColumnExprVariant() public pure {
        VerificationBuilder.Builder memory builder;
        uint256[] memory values = new uint256[](2);
        values[0] = 0x11111111;
        values[1] = 0x22222222;
        builder.columnEvaluations = values;

        bytes memory expr = abi.encodePacked(COLUMN_EXPR_VARIANT, uint64(1), hex"abcdef");
        bytes memory expectedExprOut = hex"abcdef";

        uint256 eval;
        (expr, builder, eval) = ProofExpr.__proofExprEvaluate(expr, builder, 0);

        assert(eval == 0x22222222);
        assert(expr.length == expectedExprOut.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == expectedExprOut[i]);
        }
    }

    function testLiteralExprVariant() public pure {
        VerificationBuilder.Builder memory builder;
        bytes memory expr = abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(2), hex"abcdef");
        bytes memory expectedExprOut = hex"abcdef";

        uint256 eval;
        (expr, builder, eval) = ProofExpr.__proofExprEvaluate(expr, builder, 3);

        assert(eval == 6); // 2 * 3
        assert(expr.length == expectedExprOut.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == expectedExprOut[i]);
        }
    }

    function testEqualsExprVariant() public pure {
        VerificationBuilder.Builder memory builder;
        builder.finalRoundMLEs = new uint256[](2);
        builder.finalRoundMLEs[1] = 123;
        builder.constraintMultipliers = new uint256[](2);
        builder.constraintMultipliers[0] = 456;
        builder.rowMultipliersEvaluation = 789;
        builder.maxDegree = 3;

        bytes memory expr = abi.encodePacked(
            EQUALS_EXPR_VARIANT,
            abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(2)),
            abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(2)),
            hex"abcdef"
        );
        bytes memory expectedExprOut = hex"abcdef";

        uint256 eval;
        (expr, builder, eval) = ProofExpr.__proofExprEvaluate(expr, builder, 999);

        assert(eval == 123);
        assert(builder.aggregateEvaluation == 0);
        assert(expr.length == expectedExprOut.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == expectedExprOut[i]);
        }
    }

    function testAddExprVariant() public pure {
        VerificationBuilder.Builder memory builder;
        bytes memory expr = abi.encodePacked(
            ADD_EXPR_VARIANT,
            abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(2)),
            abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(2)),
            hex"abcdef"
        );
        bytes memory expectedExprOut = hex"abcdef";

        uint256 eval;
        (expr, builder, eval) = ProofExpr.__proofExprEvaluate(expr, builder, 3);

        assert(eval == 12); // 2 * 3 + 2 * 3
        assert(expr.length == expectedExprOut.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == expectedExprOut[i]);
        }
    }

    function testSubtractExprVariant() public pure {
        VerificationBuilder.Builder memory builder;
        bytes memory expr = abi.encodePacked(
            SUBTRACT_EXPR_VARIANT,
            abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(3)),
            abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(2)),
            hex"abcdef"
        );
        bytes memory expectedExprOut = hex"abcdef";

        uint256 eval;
        (expr, builder, eval) = ProofExpr.__proofExprEvaluate(expr, builder, 3);

        assert(eval == 3); // 3 * 3 - 2 * 3
        assert(expr.length == expectedExprOut.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == expectedExprOut[i]);
        }
    }

    function testMultiplyExprVariant() public pure {
        VerificationBuilder.Builder memory builder;
        builder.finalRoundMLEs = new uint256[](1);
        builder.finalRoundMLEs[0] = 405;
        builder.constraintMultipliers = new uint256[](1);
        builder.constraintMultipliers[0] = 20;
        builder.rowMultipliersEvaluation = 2;
        builder.maxDegree = 3;

        bytes memory expr = abi.encodePacked(
            MULTIPLY_EXPR_VARIANT,
            abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(2)),
            abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(2)),
            hex"abcdef"
        );
        bytes memory expectedExprOut = hex"abcdef";

        uint256 eval;
        (expr, builder, eval) = ProofExpr.__proofExprEvaluate(expr, builder, 10);

        assert(eval == 405);
        assert(builder.aggregateEvaluation == 200);
        assert(expr.length == expectedExprOut.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == expectedExprOut[i]);
        }
    }

    function testAndExprVariant() public pure {
        VerificationBuilder.Builder memory builder;
        builder.finalRoundMLEs = new uint256[](1);
        builder.finalRoundMLEs[0] = 105;
        builder.constraintMultipliers = new uint256[](1);
        builder.constraintMultipliers[0] = 20;
        builder.rowMultipliersEvaluation = 2;
        builder.maxDegree = 3;

        bytes memory expr = abi.encodePacked(
            AND_EXPR_VARIANT,
            abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(1)),
            abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(1)),
            hex"abcdef"
        );
        bytes memory expectedExprOut = hex"abcdef";

        uint256 eval;
        (expr, builder, eval) = ProofExpr.__proofExprEvaluate(expr, builder, 10);

        assert(eval == 105);
        assert(builder.aggregateEvaluation == 200);
        assert(expr.length == expectedExprOut.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == expectedExprOut[i]);
        }
    }

    function testOrExprVariant() public pure {
        VerificationBuilder.Builder memory builder;
        builder.finalRoundMLEs = new uint256[](1);
        builder.finalRoundMLEs[0] = 105;
        builder.constraintMultipliers = new uint256[](1);
        builder.constraintMultipliers[0] = 20;
        builder.rowMultipliersEvaluation = 2;
        builder.maxDegree = 3;

        bytes memory expr = abi.encodePacked(
            OR_EXPR_VARIANT,
            abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(1)),
            abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(1)),
            hex"abcdef"
        );
        bytes memory expectedExprOut = hex"abcdef";

        uint256 eval;
        (expr, builder, eval) = ProofExpr.__proofExprEvaluate(expr, builder, 10);

        assert(eval == addmod(MODULUS, mulmod(MODULUS_MINUS_ONE, 85, MODULUS), MODULUS));
        assert(builder.aggregateEvaluation == 200);
        assert(expr.length == expectedExprOut.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == expectedExprOut[i]);
        }
    }

    function testNotExprVariant() public pure {
        VerificationBuilder.Builder memory builder;
        bytes memory expr = abi.encodePacked(
            NOT_EXPR_VARIANT, abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(0)), hex"abcdef"
        );
        bytes memory expectedExprOut = hex"abcdef";
        uint256 eval;
        (expr, builder, eval) = ProofExpr.__proofExprEvaluate(expr, builder, 3);

        assert(eval == 3); // 1 * 3 - 0 * 3
        assert(expr.length == expectedExprOut.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == expectedExprOut[i]);
        }
    }

    function testCastExprVariant() public pure {
        VerificationBuilder.Builder memory builder;
        bytes memory expr = abi.encodePacked(
            CAST_EXPR_VARIANT,
            DATA_TYPE_BIGINT_VARIANT,
            abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_INT_VARIANT, int32(2)),
            hex"abcdef"
        );
        bytes memory expectedExprOut = hex"abcdef";

        uint256 eval;
        (expr, builder, eval) = ProofExpr.__proofExprEvaluate(expr, builder, 3);

        assert(eval == 6); // 2 * 3
        assert(expr.length == expectedExprOut.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == expectedExprOut[i]);
        }
    }

    function testInequalityExprVariant() public pure {
        bytes memory expr = abi.encodePacked(
            INEQUALITY_EXPR_VARIANT,
            abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(7)),
            abi.encodePacked(COLUMN_EXPR_VARIANT, uint64(0)),
            true,
            hex"abcdef"
        );
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
            [int64(-99), -16, 67, 83],
            [int64(1), 1, 1, 1],
            [int64(0), 1, 0, 0],
            [int64(0), 1, 1, 1],
            [int64(1), 0, 0, 0],
            [int64(0), 1, 0, 1],
            [int64(1), 0, 0, 1],
            [int64(1), 0, 1, 0],
            [int64(1), 1, 0, 0],
            [int64(0), 0, 1, 1]
        ];

        uint256[] memory evaluations = new uint256[](10);

        for (uint8 i = 0; i < 10; ++i) {
            int64 evaluation = 0;
            for (uint8 j = 0; j < 4; ++j) {
                evaluation += evaluationVector[j] * vectorsToEvaluate[i][j];
            }
            evaluations[i] = F.from(evaluation).into();
        }

        builder.columnEvaluations = new uint256[](1);
        builder.columnEvaluations[0] = evaluations[0];

        uint256[] memory finalRoundMles = new uint256[](7);
        for (uint8 i = 2; i < 9; ++i) {
            finalRoundMles[i - 2] = evaluations[i];
        }

        VerificationBuilder.__setFinalRoundMLEs(builder, finalRoundMles);
        bytes memory expectedExprOut = hex"abcdef";

        uint256 eval;
        (expr, builder, eval) = ProofExpr.__proofExprEvaluate(expr, builder, evaluations[1]);

        assert(eval == evaluations[9]); // 2 * 3
        assert(expr.length == expectedExprOut.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == expectedExprOut[i]);
        }
    }

    function testPlaceholderExprVariant() public pure {
        VerificationBuilder.Builder memory builder = VerificationBuilder.__builderNew();
        uint256[] memory placeholderParams = new uint256[](5);
        placeholderParams[3] = 42; // Set parameter for index 3
        builder.placeholderParameters = placeholderParams;

        // Encode a placeholder expression: index=3 (uint64), column_type=4 (INT, uint32)
        bytes memory expr = abi.encodePacked(PLACEHOLDER_EXPR_VARIANT, uint64(3), uint32(4), hex"abcdef");
        bytes memory expectedExprOut = hex"abcdef";

        uint256 eval;
        (expr, builder, eval) = ProofExpr.__proofExprEvaluate(expr, builder, 5);

        assert(eval == (42 * 5) % MODULUS); // placeholder_param * chi_eval
        assert(expr.length == expectedExprOut.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == expectedExprOut[i]);
        }
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testUnsupportedVariant() public {
        VerificationBuilder.Builder memory builder;
        bytes memory exprIn = abi.encodePacked(uint32(25), hex"abcdef");
        vm.expectRevert(Errors.UnsupportedProofExprVariant.selector);
        ProofExpr.__proofExprEvaluate(exprIn, builder, 0);
    }

    function testVariantsMatchEnum() public pure {
        assert(uint32(ProofExpr.ExprVariant.Column) == COLUMN_EXPR_VARIANT);
        assert(uint32(ProofExpr.ExprVariant.Literal) == LITERAL_EXPR_VARIANT);
        assert(uint32(ProofExpr.ExprVariant.Equals) == EQUALS_EXPR_VARIANT);
        assert(uint32(ProofExpr.ExprVariant.Add) == ADD_EXPR_VARIANT);
        assert(uint32(ProofExpr.ExprVariant.Subtract) == SUBTRACT_EXPR_VARIANT);
        assert(uint32(ProofExpr.ExprVariant.Multiply) == MULTIPLY_EXPR_VARIANT);
        assert(uint32(ProofExpr.ExprVariant.And) == AND_EXPR_VARIANT);
        assert(uint32(ProofExpr.ExprVariant.Or) == OR_EXPR_VARIANT);
        assert(uint32(ProofExpr.ExprVariant.Not) == NOT_EXPR_VARIANT);
        assert(uint32(ProofExpr.ExprVariant.Cast) == CAST_EXPR_VARIANT);
        assert(uint32(ProofExpr.ExprVariant.Inequality) == INEQUALITY_EXPR_VARIANT);
        assert(uint32(ProofExpr.ExprVariant.Placeholder) == PLACEHOLDER_EXPR_VARIANT);
    }
}
