// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {Errors} from "../../src/base/Errors.sol";
import {VerificationBuilder} from "../../src/builder/VerificationBuilder.pre.sol";
import {ProofPlan} from "../../src/proof_plans/ProofPlan.pre.sol";
import {FF, F} from "../base/FieldUtil.sol";

contract ProofPlanTest is Test {
    function testFilterExecVariant() public pure {
        bytes memory plan = abi.encodePacked(
            FILTER_EXEC_VARIANT,
            uint64(0), // table_number
            abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(101)), // where clause
            abi.encodePacked( // select clause
                uint64(3),
                abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(102)),
                abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(103)),
                abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(104))
            ),
            hex"abcdef"
        );
        VerificationBuilder.Builder memory builder;
        builder.maxDegree = 3;
        builder.finalRoundMLEs = new uint256[](5);
        builder.finalRoundMLEs[0] = 202;
        builder.finalRoundMLEs[1] = 203;
        builder.finalRoundMLEs[2] = 204;
        builder.finalRoundMLEs[3] = 301;
        builder.finalRoundMLEs[4] = 302;
        builder.constraintMultipliers = new uint256[](4);
        builder.constraintMultipliers[0] = 401;
        builder.constraintMultipliers[1] = 402;
        builder.constraintMultipliers[2] = 403;
        builder.constraintMultipliers[3] = 404;
        builder.challenges = new uint256[](2);
        builder.challenges[0] = 501;
        builder.challenges[1] = 502;
        builder.aggregateEvaluation = 0;
        builder.rowMultipliersEvaluation = 601;
        builder.chiEvaluations = new uint256[](1);
        builder.chiEvaluations[0] = 701;
        builder.tableChiEvaluations = new uint256[](1);
        builder.tableChiEvaluations[0] = 801;

        uint256[] memory evals;
        uint256 outputChiEval;
        (plan, builder, evals, outputChiEval) = ProofPlan.__proofPlanEvaluate(plan, builder);

        FF cFold = FF.wrap(502 * 502) * FF.wrap(102 * 801) + FF.wrap(502) * FF.wrap(103 * 801) + FF.wrap(104 * 801);
        FF dFold = FF.wrap(502 * 502) * FF.wrap(202) + FF.wrap(502) * FF.wrap(203) + FF.wrap(204);

        FF zeroSumConstraint0 = FF.wrap(301) * FF.wrap(101 * 801) - FF.wrap(302);
        FF identityConstraint1 = (F.ONE + FF.wrap(501) * cFold) * FF.wrap(301) - FF.wrap(801);
        FF identityConstraint2 = (F.ONE + FF.wrap(501) * dFold) * FF.wrap(302) - FF.wrap(701);
        FF identityConstraint3 = FF.wrap(501) * dFold * (FF.wrap(701) - F.ONE);

        FF expectedAggregateEvaluation = zeroSumConstraint0 * FF.wrap(401) + identityConstraint1 * FF.wrap(402 * 601)
            + identityConstraint2 * FF.wrap(403 * 601) + identityConstraint3 * FF.wrap(404 * 601);

        assert(evals.length == 3);
        assert(evals[0] == 202);
        assert(evals[1] == 203);
        assert(evals[2] == 204);
        assert(builder.aggregateEvaluation == expectedAggregateEvaluation.into());
        assert(builder.finalRoundMLEs.length == 0);
        assert(builder.constraintMultipliers.length == 0);

        bytes memory expectedExprOut = hex"abcdef";
        assert(plan.length == expectedExprOut.length);
        uint256 exprOutLength = plan.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(plan[i] == expectedExprOut[i]);
        }
    }

    function testProjectionExecVariant() public pure {
        bytes memory inputPlan = abi.encodePacked(
            FILTER_EXEC_VARIANT,
            uint64(0), // table_number
            abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(101)), // where clause
            abi.encodePacked( // select clause
                uint64(3),
                abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(102)),
                abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(103)),
                abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(104))
            )
        );
        VerificationBuilder.Builder memory builder;
        builder.maxDegree = 3;
        builder.finalRoundMLEs = new uint256[](5);
        builder.finalRoundMLEs[0] = 202;
        builder.finalRoundMLEs[1] = 203;
        builder.finalRoundMLEs[2] = 204;
        builder.finalRoundMLEs[3] = 301;
        builder.finalRoundMLEs[4] = 302;
        builder.constraintMultipliers = new uint256[](4);
        builder.constraintMultipliers[0] = 401;
        builder.constraintMultipliers[1] = 402;
        builder.constraintMultipliers[2] = 403;
        builder.constraintMultipliers[3] = 404;
        builder.challenges = new uint256[](2);
        builder.challenges[0] = 501;
        builder.challenges[1] = 502;
        builder.aggregateEvaluation = 0;
        builder.rowMultipliersEvaluation = 601;
        builder.chiEvaluations = new uint256[](1);
        builder.chiEvaluations[0] = 701;
        builder.tableChiEvaluations = new uint256[](1);
        builder.tableChiEvaluations[0] = 801;

        bytes memory plan = abi.encodePacked(
            PROJECTION_EXEC_VARIANT,
            inputPlan, // inputPlan
            abi.encodePacked( // select clause
                uint64(3), // It doesn't really make sense to select literals from a subquery, but this is just what's easiest for testing right now
                abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(102)),
                abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(103)),
                abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(104))
            ),
            hex"abcdef"
        );

        uint256[] memory evals;
        uint256 outputChiEval;
        (plan, builder, evals, outputChiEval) = ProofPlan.__proofPlanEvaluate(plan, builder);

        assert(evals.length == 3);
        assert(evals[0] == 71502);
        assert(evals[1] == 72203);
        assert(evals[2] == 72904);
        assert(outputChiEval == 701);

        bytes memory expectedExprOut = hex"abcdef";
        assert(plan.length == expectedExprOut.length);
        uint256 exprOutLength = plan.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(plan[i] == expectedExprOut[i]);
        }
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testUnsupportedVariant() public {
        VerificationBuilder.Builder memory builder;
        bytes memory plan = abi.encodePacked(uint32(4), hex"abcdef");
        vm.expectRevert(Errors.UnsupportedProofPlanVariant.selector);
        ProofPlan.__proofPlanEvaluate(plan, builder);
    }

    function testEmptyExecVariant() public pure {
        VerificationBuilder.Builder memory builder;
        bytes memory plan = abi.encodePacked(EMPTY_EXEC_VARIANT, hex"abcdef");
        uint256[] memory evals;
        uint256 chiEval;
        (plan, builder, evals, chiEval) = ProofPlan.__proofPlanEvaluate(plan, builder);
        assert(evals.length == 0);
        bytes memory expectedExprOut = hex"abcdef";
        uint256 exprOutLength = plan.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(plan[i] == expectedExprOut[i]);
        }
    }

    function testTableExecVariant() public pure {
        bytes memory plan = abi.encodePacked(
            TABLE_EXEC_VARIANT,
            uint64(0), // table_ref
            uint64(3), // column_count
            uint64(0), // column1_index
            uint64(1), // column2_index
            uint64(2), // column3_index
            hex"abcdef"
        );

        VerificationBuilder.Builder memory builder;
        builder.tableChiEvaluations = new uint256[](1);
        builder.tableChiEvaluations[0] = 801;
        builder.columnEvaluations = new uint256[](3);
        builder.columnEvaluations[0] = 101;
        builder.columnEvaluations[1] = 102;
        builder.columnEvaluations[2] = 103;

        uint256[] memory evals;
        uint256 chiEval;
        (plan, builder, evals, chiEval) = ProofPlan.__proofPlanEvaluate(plan, builder);

        assert(evals.length == 3);
        assert(evals[0] == 101);
        assert(evals[1] == 102);
        assert(evals[2] == 103);

        bytes memory expectedExprOut = hex"abcdef";
        assert(plan.length == expectedExprOut.length);
        uint256 exprOutLength = plan.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(plan[i] == expectedExprOut[i]);
        }
    }

    function testSliceExecVariant() public pure {
        bytes memory plan = abi.encodePacked(
            SLICE_EXEC_VARIANT,
            TABLE_EXEC_VARIANT,
            uint64(0), // table_number
            uint64(2), // column_count
            uint64(0), // column1_index
            uint64(1), // column2_index
            uint64(1), // skip 1
            true, // fetch set
            uint64(2), // fetch 2
            hex"abcdef"
        );

        VerificationBuilder.Builder memory builder;

        // column evaluations
        builder.columnEvaluations = new uint256[](2);
        uint256[2][4] memory column =
            [[uint256(1), MODULUS_MINUS_ONE], [uint256(3), 7], [MODULUS_MINUS_ONE, 0], [uint256(400), 1]];

        // chi evals
        builder.tableChiEvaluations = new uint256[](1);
        builder.tableChiEvaluations[0] = 1;
        builder.chiEvaluations = new uint256[](12); // 3 chi evaluations times 4 rows
        {
            uint256[4] memory outputChiEval = [uint256(1), 1, 0, 0]; // This has length 2 because there are 2 rows selected
            uint256[4] memory offsetChiEval = [uint256(1), 0, 0, 0]; // This has length 1 because 1 row is skipped
            uint256[4] memory maxChiEval = [uint256(1), 1, 1, 0]; // This has length 3 because 1 row is skipped and 2 are fetched
            for (uint8 i = 0; i < 4; ++i) {
                builder.chiEvaluations[i * 3] = outputChiEval[i];
                builder.chiEvaluations[i * 3 + 1] = offsetChiEval[i];
                builder.chiEvaluations[i * 3 + 2] = maxChiEval[i];
            }
        }

        // challenges
        builder.challenges = new uint256[](8);
        builder.challenges[0] = 3; // alpha
        builder.challenges[1] = 8; // beta
        builder.challenges[2] = 3; // alpha
        builder.challenges[3] = 8; // beta
        builder.challenges[4] = 3; // alpha
        builder.challenges[5] = 8; // beta
        builder.challenges[6] = 3; // alpha
        builder.challenges[7] = 8; // beta

        // max degree and row multipliers
        builder.maxDegree = 3;
        builder.rowMultipliersEvaluation = 1;

        // constraint multipliers
        builder.constraintMultipliers = new uint256[](16); // 4 constraints times 4 rows
        for (uint8 i = 0; i < 16; ++i) {
            builder.constraintMultipliers[i] = 1;
        }

        // final round mles
        builder.finalRoundMLEs = new uint256[](16); // 4 mles times 4 rows
        {
            uint256[4] memory filteredColumn1 = [column[1][0], column[2][0], 0, 0];
            uint256[4] memory filteredColumn2 = [column[1][1], column[2][1], 0, 0];

            uint256 inv22 = 14923801958072233106077094826311778469464793909374568870703321036301687610648;
            uint256 inv94 = 12806950616501703587484599106267554573086808957690232860674481172996483694244;
            uint256 invNegative23 = 13323278269815211004845638279721819619116395721992368730946732983133100823419;
            uint256 inv9604 = 11712169941522494311445676710212113356939821392517492762626517213120895445541;
            uint256[4] memory cStarColumn = [inv22, inv94, invNegative23, inv9604];
            uint256[4] memory dStarColumn = [cStarColumn[1], cStarColumn[2], 0, 0];

            for (uint8 i = 0; i < 4; ++i) {
                builder.finalRoundMLEs[i * 4] = filteredColumn1[i];
                builder.finalRoundMLEs[i * 4 + 1] = filteredColumn2[i];
                builder.finalRoundMLEs[i * 4 + 2] = cStarColumn[i];
                builder.finalRoundMLEs[i * 4 + 3] = dStarColumn[i];
            }
        }
        builder.aggregateEvaluation = 0;

        for (uint8 i = 0; i < 4; ++i) {
            bytes memory planOutput;
            uint256[] memory evals;
            uint256 outputChiEval;
            builder.columnEvaluations[0] = column[i][0];
            builder.columnEvaluations[1] = column[i][1];
            (planOutput, builder, evals, outputChiEval) = ProofPlan.__proofPlanEvaluate(plan, builder);
            assert(planOutput.length == 3);
        }

        assert(builder.aggregateEvaluation == 0);
    }

    function testVariantsMatchEnum() public pure {
        assert(uint32(ProofPlan.PlanVariant.Filter) == FILTER_EXEC_VARIANT);
        assert(uint32(ProofPlan.PlanVariant.Empty) == EMPTY_EXEC_VARIANT);
        assert(uint32(ProofPlan.PlanVariant.Table) == TABLE_EXEC_VARIANT);
        assert(uint32(ProofPlan.PlanVariant.Projection) == PROJECTION_EXEC_VARIANT);
        assert(uint32(ProofPlan.PlanVariant.Slice) == SLICE_EXEC_VARIANT);
    }
}
