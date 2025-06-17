// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {VerificationBuilder} from "../../src/builder/VerificationBuilder.pre.sol";
import {FoldUtil} from "../../src/proof_gadgets/FoldUtil.pre.sol";
import {F, FF} from "../base/FieldUtil.sol";

contract FoldUtilTest is Test {
    function testFoldExprEvalsWithSingleColumn() public pure {
        VerificationBuilder.Builder memory builder;

        bytes memory plan = abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(5), hex"abcdef");
        bytes memory expectedPlanOut = hex"abcdef";
        uint256 inputChiEval = MODULUS - 3;
        uint256 beta = 7;
        uint256 columnCount = 1;
        uint256 fold;

        (plan, builder, fold) = FoldUtil.__foldExprEvals({
            __plan: plan,
            __builder: builder,
            __inputChiEval: inputChiEval,
            __beta: beta,
            __columnCount: columnCount
        });

        assert(fold == MODULUS - 15); // 5 * (-3) = -15
        assert(plan.length == expectedPlanOut.length);
        uint256 planOutLength = plan.length;
        for (uint256 i = 0; i < planOutLength; ++i) {
            assert(plan[i] == expectedPlanOut[i]);
        }
    }

    function testFoldExprEvalsWithMultipleColumns() public pure {
        VerificationBuilder.Builder memory builder;

        // Create plan with two literal expressions: 2 and 3
        bytes memory plan = abi.encodePacked(
            LITERAL_EXPR_VARIANT,
            DATA_TYPE_BIGINT_VARIANT,
            int64(2),
            LITERAL_EXPR_VARIANT,
            DATA_TYPE_BIGINT_VARIANT,
            int64(3),
            hex"abcdef"
        );
        bytes memory expectedPlanOut = hex"abcdef";
        uint256 inputChiEval = 5;
        uint256 beta = 10;
        uint256 columnCount = 2;
        uint256 fold;

        (plan, builder, fold) = FoldUtil.__foldExprEvals({
            __plan: plan,
            __builder: builder,
            __inputChiEval: inputChiEval,
            __beta: beta,
            __columnCount: columnCount
        });

        // Fold calculation: ((2 * 5) * 10 + (3 * 5) = 115
        assert(fold == 115);
        assert(plan.length == expectedPlanOut.length);
        uint256 planOutLength = plan.length;
        for (uint256 i = 0; i < planOutLength; ++i) {
            assert(plan[i] == expectedPlanOut[i]);
        }
    }

    function testFoldExprEvalsWithColumnExpression() public pure {
        VerificationBuilder.Builder memory builder;
        uint256[] memory columnEvals = new uint256[](2);
        columnEvals[0] = 42;
        columnEvals[1] = 84;
        builder.columnEvaluations = columnEvals;

        bytes memory plan = abi.encodePacked(COLUMN_EXPR_VARIANT, uint64(1), hex"abcdef");
        bytes memory expectedPlanOut = hex"abcdef";
        uint256 inputChiEval = 0; // Not used for column expressions
        uint256 beta = 5;
        uint256 columnCount = 1;
        uint256 fold;

        (plan, builder, fold) = FoldUtil.__foldExprEvals({
            __plan: plan,
            __builder: builder,
            __inputChiEval: inputChiEval,
            __beta: beta,
            __columnCount: columnCount
        });

        assert(fold == 84); // Column 1 evaluation
        assert(plan.length == expectedPlanOut.length);
        uint256 planOutLength = plan.length;
        for (uint256 i = 0; i < planOutLength; ++i) {
            assert(plan[i] == expectedPlanOut[i]);
        }
    }

    function testFoldFinalRoundMlesWithSingleColumn() public pure {
        VerificationBuilder.Builder memory builder;
        uint256[] memory finalRoundMLEs = new uint256[](1);
        finalRoundMLEs[0] = 123;
        builder.finalRoundMLEs = finalRoundMLEs;

        uint256 columnCount = 1;
        uint256 beta = 7;
        uint256 fold;
        uint256[] memory evaluations;

        (builder, fold, evaluations) = FoldUtil.__foldFinalRoundMles(builder, beta, columnCount);

        assert(fold == 123);
        assert(evaluations.length == 1);
        assert(evaluations[0] == 123);
        // Builder should be updated after consuming MLE
        assert(builder.finalRoundMLEs.length == 0);
    }

    function testFoldFinalRoundMlesWithMultipleColumns() public pure {
        VerificationBuilder.Builder memory builder;
        uint256[] memory finalRoundMLEs = new uint256[](3);
        finalRoundMLEs[0] = 10;
        finalRoundMLEs[1] = 20;
        finalRoundMLEs[2] = 30;
        builder.finalRoundMLEs = finalRoundMLEs;

        uint256 columnCount = 3;
        uint256 beta = 5;
        uint256 fold;
        uint256[] memory evaluations;

        (builder, fold, evaluations) = FoldUtil.__foldFinalRoundMles(builder, beta, columnCount);

        // Fold calculation: 10 * 5^2 + 20 * 5 + 30 = 10 * 25 + 20 * 5 + 30 = 250 + 100 + 30 = 380
        assert(fold == 380);
        assert(evaluations.length == 3);
        assert(evaluations[0] == 10);
        assert(evaluations[1] == 20);
        assert(evaluations[2] == 30);
        assert(builder.finalRoundMLEs.length == 0); // All MLEs consumed
    }

    function testFuzzFoldExprEvalsWithLiterals(int64 value1, int64 value2, uint256 chiEval, uint256 beta) public pure {
        vm.assume(beta != 0);
        vm.assume(chiEval < MODULUS);
        vm.assume(beta < MODULUS);

        VerificationBuilder.Builder memory builder;

        bytes memory plan = abi.encodePacked(
            LITERAL_EXPR_VARIANT,
            DATA_TYPE_BIGINT_VARIANT,
            value1,
            LITERAL_EXPR_VARIANT,
            DATA_TYPE_BIGINT_VARIANT,
            value2,
            hex"abcdef"
        );
        bytes memory expectedPlanOut = hex"abcdef";

        uint256 columnCount = 2;
        uint256 fold;

        (plan, builder, fold) = FoldUtil.__foldExprEvals({
            __plan: plan,
            __builder: builder,
            __inputChiEval: chiEval,
            __beta: beta,
            __columnCount: columnCount
        });

        // Expected fold: value1 * chiEval * beta + value2 * chiEval
        FF expectedFold = (F.from(value1) * F.from(chiEval)) * F.from(beta) + (F.from(value2) * F.from(chiEval));

        assert(fold == expectedFold.into());
        assert(plan.length == expectedPlanOut.length);
        uint256 planOutLength = plan.length;
        for (uint256 i = 0; i < planOutLength; ++i) {
            assert(plan[i] == expectedPlanOut[i]);
        }
    }

    function testFuzzFoldFinalRoundMles(uint256[] memory mles, uint256 beta) public pure {
        vm.assume(mles.length > 0);
        vm.assume(mles.length < 11); // Reasonable limit for testing
        vm.assume(beta != 0);
        vm.assume(beta < MODULUS);
        uint256 mlesLength = mles.length;

        // Ensure all MLEs are valid field elements
        for (uint256 i = 0; i < mlesLength; ++i) {
            vm.assume(mles[i] < MODULUS);
        }

        VerificationBuilder.Builder memory builder;
        builder.finalRoundMLEs = mles;
        uint256 fold;
        uint256[] memory evaluations;

        (builder, fold, evaluations) = FoldUtil.__foldFinalRoundMles(builder, beta, mlesLength);

        // Verify evaluations match input MLEs
        assert(evaluations.length == mlesLength);
        int64 zero = 0;
        FF expectedFold = F.from(zero);
        for (uint256 i = 0; i < mlesLength; ++i) {
            expectedFold = expectedFold * F.from(beta) + F.from(mles[i]);
        }

        assert(fold == expectedFold.into());
        assert(builder.finalRoundMLEs.length == 0); // All MLEs consumed
    }

    function testEmptyFoldFinalRoundMles() public pure {
        VerificationBuilder.Builder memory builder;
        uint256[] memory emptyMLEs = new uint256[](0);
        builder.finalRoundMLEs = emptyMLEs;

        uint256 columnCount = 0;
        uint256 beta = 5;
        uint256 fold;
        uint256[] memory evaluations;

        (builder, fold, evaluations) = FoldUtil.__foldFinalRoundMles(builder, beta, columnCount);

        assert(fold == 0);
        assert(evaluations.length == 0);
        assert(builder.finalRoundMLEs.length == 0);
    }

    function testFoldColumnExprEvalsWithSingleColumn() public pure {
        VerificationBuilder.Builder memory builder;
        uint256[] memory columnEvals = new uint256[](2);
        columnEvals[0] = 42;
        columnEvals[1] = 84;
        builder.columnEvaluations = columnEvals;

        bytes memory plan = abi.encodePacked(uint64(1), hex"abcdef");
        bytes memory expectedPlanOut = hex"abcdef";
        uint256 beta = 5;
        uint256 columnCount = 1;
        uint256 fold;

        (plan, builder, fold) =
            FoldUtil.__foldColumnExprEvals({__plan: plan, __builder: builder, __beta: beta, __columnCount: columnCount});

        assert(fold == 84); // Column 1 evaluation
        assert(plan.length == expectedPlanOut.length);
        uint256 planOutLength = plan.length;
        for (uint256 i = 0; i < planOutLength; ++i) {
            assert(plan[i] == expectedPlanOut[i]);
        }
    }

    function testFoldColumnExprEvalsWithMultipleColumns() public pure {
        VerificationBuilder.Builder memory builder;
        uint256[] memory columnEvals = new uint256[](3);
        columnEvals[0] = 10;
        columnEvals[1] = 20;
        columnEvals[2] = 30;
        builder.columnEvaluations = columnEvals;

        bytes memory plan = abi.encodePacked(uint64(0), uint64(2), hex"abcdef");
        bytes memory expectedPlanOut = hex"abcdef";
        uint256 beta = 7;
        uint256 columnCount = 2;
        uint256 fold;

        (plan, builder, fold) =
            FoldUtil.__foldColumnExprEvals({__plan: plan, __builder: builder, __beta: beta, __columnCount: columnCount});

        // Fold calculation: 10 * 7 + 30 = 70 + 30 = 100
        assert(fold == 100);
        assert(plan.length == expectedPlanOut.length);
        uint256 planOutLength = plan.length;
        for (uint256 i = 0; i < planOutLength; ++i) {
            assert(plan[i] == expectedPlanOut[i]);
        }
    }

    function testFuzzFoldColumnExprEvals(uint256[] memory columnEvals, uint256 beta) public pure {
        vm.assume(columnEvals.length > 0);
        vm.assume(columnEvals.length < 6); // Reasonable limit for testing
        vm.assume(beta != 0);
        vm.assume(beta < MODULUS);
        uint256 columnEvalsLength = columnEvals.length;

        // Ensure all evaluations are valid field elements
        for (uint256 i = 0; i < columnEvalsLength; ++i) {
            vm.assume(columnEvals[i] < MODULUS);
        }

        VerificationBuilder.Builder memory builder;
        builder.columnEvaluations = columnEvals;

        // Create plan with column expressions referencing each column
        bytes memory plan = hex"";
        for (uint256 i = 0; i < columnEvalsLength; ++i) {
            plan = abi.encodePacked(plan, uint64(i));
        }
        plan = abi.encodePacked(plan, hex"abcdef");

        bytes memory expectedPlanOut = hex"abcdef";
        uint256 fold;

        (plan, builder, fold) = FoldUtil.__foldColumnExprEvals({
            __plan: plan,
            __builder: builder,
            __beta: beta,
            __columnCount: columnEvalsLength
        });

        // Calculate expected fold: sum of (columnEvals[i] * beta^(n-1-i))
        int64 zero = 0;
        FF expectedFold = F.from(zero);
        for (uint256 i = 0; i < columnEvalsLength; ++i) {
            expectedFold = expectedFold * F.from(beta) + F.from(columnEvals[i]);
        }

        assert(fold == expectedFold.into());
        assert(plan.length == expectedPlanOut.length);
        uint256 planOutLength = plan.length;
        for (uint256 i = 0; i < planOutLength; ++i) {
            assert(plan[i] == expectedPlanOut[i]);
        }
    }

    function testEmptyFoldColumnExprEvals() public pure {
        VerificationBuilder.Builder memory builder;
        uint256[] memory columnEvals = new uint256[](0);
        builder.columnEvaluations = columnEvals;

        bytes memory plan = hex"abcdef";
        bytes memory expectedPlanOut = hex"abcdef";
        uint256 beta = 5;
        uint256 columnCount = 0;
        uint256 fold;

        (plan, builder, fold) =
            FoldUtil.__foldColumnExprEvals({__plan: plan, __builder: builder, __beta: beta, __columnCount: columnCount});

        assert(fold == 0);
        assert(plan.length == expectedPlanOut.length);
        uint256 planOutLength = plan.length;
        for (uint256 i = 0; i < planOutLength; ++i) {
            assert(plan[i] == expectedPlanOut[i]);
        }
    }
}
