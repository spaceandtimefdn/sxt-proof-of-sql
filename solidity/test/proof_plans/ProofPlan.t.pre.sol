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
        builder.firstRoundMLEs = new uint256[](3);
        builder.finalRoundMLEs = new uint256[](2);
        builder.firstRoundMLEs[0] = 202;
        builder.firstRoundMLEs[1] = 203;
        builder.firstRoundMLEs[2] = 204;
        builder.finalRoundMLEs[0] = 301;
        builder.finalRoundMLEs[1] = 302;
        builder.constraintMultipliers = new uint256[](4);
        builder.constraintMultipliers[0] = 402;
        builder.constraintMultipliers[1] = 403;
        builder.constraintMultipliers[2] = 401;
        builder.constraintMultipliers[3] = 404;
        builder.challenges = new uint256[](2);
        builder.challenges[0] = 501;
        builder.challenges[1] = 502;
        builder.aggregateEvaluation = 0;
        builder.rowMultipliersEvaluation = 601;
        builder.chiEvaluations = new uint256[](2);
        builder.chiEvaluations[0] = 1;
        builder.chiEvaluations[1] = 701;
        builder.tableChiEvaluations = new uint256[](2);
        builder.tableChiEvaluations[0] = 1;
        builder.tableChiEvaluations[1] = 801;

        uint256[] memory evals;
        uint256 length;
        uint256 outputChiEval;
        (plan, builder, evals, length, outputChiEval) = ProofPlan.__proofPlanEvaluate(plan, builder);

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
        builder.firstRoundMLEs = new uint256[](3);
        builder.finalRoundMLEs = new uint256[](2);
        builder.firstRoundMLEs[0] = 202;
        builder.firstRoundMLEs[1] = 203;
        builder.firstRoundMLEs[2] = 204;
        builder.finalRoundMLEs[0] = 301;
        builder.finalRoundMLEs[1] = 302;
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
        builder.chiEvaluations = new uint256[](2);
        builder.chiEvaluations[0] = 1;
        builder.chiEvaluations[1] = 701;
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
        uint256 length;
        uint256 outputChiEval;
        (plan, builder, evals, length, outputChiEval) = ProofPlan.__proofPlanEvaluate(plan, builder);

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

    function _configureBuilderForGroupByExec(VerificationBuilder.Builder memory builder)
        internal
        pure
        returns (VerificationBuilder.Builder memory configuredBuilder)
    {
        // table chi evals
        builder.tableChiEvaluations = new uint256[](2);
        builder.tableChiEvaluations[0] = 4;
        builder.tableChiEvaluations[1] = 1;

        // rho and chi evals
        builder.rhoEvaluations = new uint256[](8);
        builder.rhoEvaluations[0] = 0;
        builder.rhoEvaluations[1] = 0;
        builder.rhoEvaluations[2] = 1;
        builder.rhoEvaluations[3] = 1;
        builder.rhoEvaluations[4] = 0;
        builder.rhoEvaluations[5] = 2;
        builder.rhoEvaluations[6] = 0;
        builder.rhoEvaluations[7] = 0;
        builder.chiEvaluations = new uint256[](16);
        builder.chiEvaluations[0] = 4;
        builder.chiEvaluations[1] = 1; // output_chi_eval
        builder.chiEvaluations[2] = 4;
        builder.chiEvaluations[3] = 1; // shifted_output_chi_eval
        builder.chiEvaluations[4] = 4;
        builder.chiEvaluations[5] = 1;
        builder.chiEvaluations[6] = 4;
        builder.chiEvaluations[7] = 1;
        builder.chiEvaluations[8] = 4;
        builder.chiEvaluations[9] = 0;
        builder.chiEvaluations[10] = 4;
        builder.chiEvaluations[11] = 1;
        builder.chiEvaluations[12] = 4;
        builder.chiEvaluations[13] = 0;
        builder.chiEvaluations[14] = 4;
        builder.chiEvaluations[15] = 0;

        // bit distributions
        uint256[] memory bitDistribution = new uint256[](8);
        bitDistribution[0] = 0x8000000000000000000000000000000000000000000000000000000000000000;
        bitDistribution[1] = 1;
        bitDistribution[2] = 0x8000000000000000000000000000000000000000000000000000000000000000;
        bitDistribution[3] = 1;
        bitDistribution[4] = 0x8000000000000000000000000000000000000000000000000000000000000000;
        bitDistribution[5] = 1;
        bitDistribution[6] = 0x8000000000000000000000000000000000000000000000000000000000000000;
        bitDistribution[7] = 1;
        VerificationBuilder.__setBitDistributions(builder, bitDistribution);

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
        builder.constraintMultipliers = new uint256[](28); // 7 constraints times 4 rows
        for (uint8 i = 0; i < 28; ++i) {
            builder.constraintMultipliers[i] = 1;
        }

        // Aggregate evaluation
        builder.aggregateEvaluation = 0;
        configuredBuilder = builder;
    }

    function _configureForMinimalGroupByExec(VerificationBuilder.Builder memory builder)
        internal
        pure
        returns (VerificationBuilder.Builder memory configuredBuilder)
    {
        // column evaluations
        builder.columnEvaluations = new uint256[](1);

        uint256[4] memory gOut = [MODULUS_MINUS_ONE, 1, 0, 0];
        uint256[4] memory count = [uint256(2), 2, 0, 0];
        // Check that (-1, 1) is strictly increasing hence the group by is valid
        uint256[4] memory shiftedGOut = [uint256(0), MODULUS_MINUS_ONE, 1, 0];
        // cStar = 1 / (1 + alpha * (column + beta * (rho + chi)))
        uint256[4] memory cStarEval = [
            uint256(14923801958072233106077094826311778469464793909374568870703321036301687610648),
            21467315124303904544895513327079250567614742008100341375550161798372427563009,
            1,
            0
        ];
        // dStar = 1 / (1 + alpha * (shiftedColumn + beta * rhoPlusOne))
        uint256[4] memory dStarEval = [
            uint256(1),
            14923801958072233106077094826311778469464793909374568870703321036301687610648,
            21467315124303904544895513327079250567614742008100341375550161798372427563009,
            0
        ];
        uint256[4] memory sign = [uint256(1), 0, 1, 0]; //Sign of [1, -2, 1];

        // mles
        builder.firstRoundMLEs = new uint256[](12); // 3 mles times 4 rows
        builder.finalRoundMLEs = new uint256[](20); // 5 mles times 4 rows
        {
            uint256 inv4 = 16416182153879456416684804308942956316411273300312025757773653139931856371713;
            uint256 invNegative2 = 10944121435919637611123202872628637544274182200208017171849102093287904247808;
            uint256[4] memory gInStarColumn = [invNegative2, inv4, inv4, invNegative2];
            uint256[4] memory gOutStarColumn = [invNegative2, inv4, 0, 0];

            for (uint8 i = 0; i < 4; ++i) {
                builder.finalRoundMLEs[i * 5] = gInStarColumn[i];
                builder.firstRoundMLEs[i * 3] = gOut[i];
                builder.finalRoundMLEs[i * 5 + 1] = gOutStarColumn[i];
                // Monotonicity check
                builder.firstRoundMLEs[i * 3 + 1] = shiftedGOut[i];
                builder.finalRoundMLEs[i * 5 + 2] = cStarEval[i];
                builder.finalRoundMLEs[i * 5 + 3] = dStarEval[i];
                builder.finalRoundMLEs[i * 5 + 4] = sign[i];
                // Continue with group by output
                builder.firstRoundMLEs[i * 3 + 2] = count[i];
            }
        }
        configuredBuilder = builder;
    }

    function testGroupByExecVariant() public pure {
        bytes memory plan = abi.encodePacked(
            GROUP_BY_EXEC_VARIANT,
            uint64(0), // table_number
            uint64(1), // group_by_count
            uint64(0), // group_by_expr[0] - column 0
            abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BOOLEAN_VARIANT, uint8(1)), // where clause
            uint64(0), // sum_count
            uint64(7), // count_alias_length (unused in verification)
            "count_0" // count_alias (unused in verification)
        );
        VerificationBuilder.Builder memory builder;
        uint256[4] memory gIn = [MODULUS_MINUS_ONE, 1, 1, MODULUS_MINUS_ONE];
        builder = _configureBuilderForGroupByExec(builder);
        builder = _configureForMinimalGroupByExec(builder);
        for (uint8 i = 0; i < 4; ++i) {
            uint256[] memory evals;
            uint256 length;
            uint256 outputChiEval;
            bytes memory planOut;
            if (i == 0) {
                builder.singletonChiEvaluation = 1; // singleton_chi_eval for first test
            } else {
                builder.singletonChiEvaluation = 0;
            }
            builder.columnEvaluations[0] = gIn[i];
            (planOut, builder, evals, length, outputChiEval) = ProofPlan.__proofPlanEvaluate(plan, builder);
            assert(planOut.length == 0);
        }
        assert(builder.aggregateEvaluation == 0);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testUnsupportedVariant() public {
        VerificationBuilder.Builder memory builder;
        bytes memory plan = abi.encodePacked(uint32(100), hex"abcdef");
        vm.expectRevert(Errors.UnsupportedProofPlanVariant.selector);
        ProofPlan.__proofPlanEvaluate(plan, builder);
    }

    function testEmptyExecVariant() public pure {
        VerificationBuilder.Builder memory builder;
        bytes memory plan = abi.encodePacked(EMPTY_EXEC_VARIANT, hex"abcdef");
        uint256[] memory evals;
        uint256 length;
        uint256 chiEval;
        (plan, builder, evals, length, chiEval) = ProofPlan.__proofPlanEvaluate(plan, builder);
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
        builder.tableChiEvaluations = new uint256[](2);
        builder.tableChiEvaluations[0] = 1;
        builder.tableChiEvaluations[1] = 801;
        builder.columnEvaluations = new uint256[](3);
        builder.columnEvaluations[0] = 101;
        builder.columnEvaluations[1] = 102;
        builder.columnEvaluations[2] = 103;

        uint256[] memory evals;
        uint256 length;
        uint256 chiEval;
        (plan, builder, evals, length, chiEval) = ProofPlan.__proofPlanEvaluate(plan, builder);

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
        builder.tableChiEvaluations = new uint256[](2);
        builder.tableChiEvaluations[0] = 4;
        builder.tableChiEvaluations[1] = 1;
        builder.chiEvaluations = new uint256[](24); // 3 chi evaluations times 4 rows times 2 slots per element
        {
            uint256[4] memory outputChiEval = [uint256(1), 1, 0, 0]; // This has length 2 because there are 2 rows selected
            uint256[4] memory offsetChiEval = [uint256(1), 0, 0, 0]; // This has length 1 because 1 row is skipped
            uint256[4] memory maxChiEval = [uint256(1), 1, 1, 0]; // This has length 3 because 1 row is skipped and 2 are fetched
            for (uint8 i = 0; i < 4; ++i) {
                builder.chiEvaluations[i * 6] = 2;
                builder.chiEvaluations[i * 6 + 1] = outputChiEval[i];
                builder.chiEvaluations[i * 6 + 2] = 1;
                builder.chiEvaluations[i * 6 + 3] = offsetChiEval[i];
                builder.chiEvaluations[i * 6 + 4] = 3;
                builder.chiEvaluations[i * 6 + 5] = maxChiEval[i];
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

        // first round mles
        builder.firstRoundMLEs = new uint256[](8); // 2 mle times 4 rows
        for (uint8 i = 0; i < 4; ++i) {
            uint256[4] memory filteredColumn1 = [column[1][0], column[2][0], 0, 0];
            uint256[4] memory filteredColumn2 = [column[1][1], column[2][1], 0, 0];
            builder.firstRoundMLEs[i * 2] = filteredColumn1[i];
            builder.firstRoundMLEs[i * 2 + 1] = filteredColumn2[i];
        }

        // final round mles
        builder.finalRoundMLEs = new uint256[](8); // 2 mles times 4 rows
        {
            uint256 inv22 = 14923801958072233106077094826311778469464793909374568870703321036301687610648;
            uint256 inv94 = 12806950616501703587484599106267554573086808957690232860674481172996483694244;
            uint256 invNegative23 = 13323278269815211004845638279721819619116395721992368730946732983133100823419;
            uint256 inv9604 = 11712169941522494311445676710212113356939821392517492762626517213120895445541;
            uint256[4] memory cStarColumn = [inv22, inv94, invNegative23, inv9604];
            uint256[4] memory dStarColumn = [cStarColumn[1], cStarColumn[2], 0, 0];

            for (uint8 i = 0; i < 4; ++i) {
                builder.finalRoundMLEs[i * 2] = cStarColumn[i];
                builder.finalRoundMLEs[i * 2 + 1] = dStarColumn[i];
            }
        }
        builder.aggregateEvaluation = 0;

        for (uint8 i = 0; i < 4; ++i) {
            bytes memory planOutput;
            uint256[] memory evals;
            uint256 length;
            uint256 outputChiEval;
            builder.columnEvaluations[0] = column[i][0];
            builder.columnEvaluations[1] = column[i][1];
            (planOutput, builder, evals, length, outputChiEval) = ProofPlan.__proofPlanEvaluate(plan, builder);
            assert(planOutput.length == 3);
        }

        assert(builder.aggregateEvaluation == 0);
    }

    function testUnionExecVariant() public pure {
        // Plan for UNION of two tables
        bytes memory plan = abi.encodePacked(
            UNION_EXEC_VARIANT,
            uint64(2), // input_count: 2 tables
            // First input plan (TableExec)
            TABLE_EXEC_VARIANT,
            uint64(0), // table_number
            uint64(1), // column_count
            uint64(0), // column1_index
            // Second input plan (TableExec)
            TABLE_EXEC_VARIANT,
            uint64(1), // table_number
            uint64(1), // column_count
            uint64(1), // column1_index
            hex"abcdef"
        );

        VerificationBuilder.Builder memory builder;

        // column evaluations for both tables
        builder.columnEvaluations = new uint256[](2);
        uint256[4] memory table1Column = [uint256(1), 3, 5, 0];
        uint256[4] memory table2Column = [uint256(7), 0, 0, 0];
        uint256[4] memory unionedColumn = [uint256(1), 3, 5, 7];

        // chi evals for tables
        builder.tableChiEvaluations = new uint256[](4);
        builder.tableChiEvaluations[0] = 3; // table 1 has 3 rows
        builder.tableChiEvaluations[2] = 1; // table 2 has 1 row
        uint256[4] memory table1ChiEval = [uint256(1), 1, 1, 0];
        uint256[4] memory table2ChiEval = [uint256(1), 0, 0, 0];

        // chi evaluations for output
        builder.chiEvaluations = new uint256[](8); // 1 chi evaluation times 4 rows times 2 slots per element
        {
            uint256[4] memory outputChiEval = [uint256(1), 1, 1, 1]; // All 4 rows in output
            for (uint8 i = 0; i < 4; ++i) {
                builder.chiEvaluations[i * 2] = 4;
                builder.chiEvaluations[i * 2 + 1] = outputChiEval[i];
            }
        }

        // challenges for FoldLogExpr (gamma, beta)
        builder.challenges = new uint256[](8);
        builder.challenges[0] = 3; // gamma
        builder.challenges[1] = 8; // beta
        builder.challenges[2] = 3;
        builder.challenges[3] = 8;
        builder.challenges[4] = 3;
        builder.challenges[5] = 8;
        builder.challenges[6] = 3;
        builder.challenges[7] = 8;

        // max degree and row multipliers
        builder.maxDegree = 3;
        builder.rowMultipliersEvaluation = 1;

        // constraint multipliers
        builder.constraintMultipliers = new uint256[](16); // 4 constraint times 4 rows
        for (uint8 i = 0; i < 16; ++i) {
            builder.constraintMultipliers[i] = 1;
        }

        // first round mles for output columns
        builder.firstRoundMLEs = new uint256[](4); // 1 mle times 4 rows
        {
            for (uint8 i = 0; i < 4; ++i) {
                builder.firstRoundMLEs[i] = unionedColumn[i];
            }
        }

        // final round mles (for FoldLogExpr constraints)
        builder.finalRoundMLEs = new uint256[](12); // 3 mles times 4 rows
        {
            // These would be computed by the FoldLogExpr gadget
            uint256 inv4 = 16416182153879456416684804308942956316411273300312025757773653139931856371713;
            uint256 inv10 = 15321770010287492655572484021680092561983855080291224040588742930603065946932;
            uint256 inv16 = 20520227692349320520856005386178695395514091625390032197217066424914820464641;
            uint256 inv22 = 14923801958072233106077094826311778469464793909374568870703321036301687610648;
            uint256[4] memory cStarColumn0 = [inv4, inv10, inv16, 0];
            uint256[4] memory cStarColumn1 = [inv22, 0, 0, 0];
            uint256[4] memory dStarColumn = [inv4, inv10, inv16, inv22];

            for (uint8 i = 0; i < 4; ++i) {
                builder.finalRoundMLEs[i * 3] = cStarColumn0[i];
                builder.finalRoundMLEs[i * 3 + 1] = cStarColumn1[i];
                builder.finalRoundMLEs[i * 3 + 2] = dStarColumn[i];
            }
        }
        builder.aggregateEvaluation = 0;

        for (uint8 i = 0; i < 4; ++i) {
            bytes memory planOutput;
            uint256[] memory evals;
            uint256 outputChiEval;
            uint256 length;
            builder.columnEvaluations[0] = table1Column[i];
            builder.columnEvaluations[1] = table2Column[i];
            builder.tableChiEvaluations[1] = table1ChiEval[i];
            builder.tableChiEvaluations[3] = table2ChiEval[i];
            (planOutput, builder, evals, length, outputChiEval) = ProofPlan.__proofPlanEvaluate(plan, builder);
            assert(planOutput.length == 3);
            assert(length == 4);
        }

        assert(builder.aggregateEvaluation == 0);
    }

    function testVariantsMatchEnum() public pure {
        assert(uint32(ProofPlan.PlanVariant.Filter) == FILTER_EXEC_VARIANT);
        assert(uint32(ProofPlan.PlanVariant.Empty) == EMPTY_EXEC_VARIANT);
        assert(uint32(ProofPlan.PlanVariant.Table) == TABLE_EXEC_VARIANT);
        assert(uint32(ProofPlan.PlanVariant.Projection) == PROJECTION_EXEC_VARIANT);
        assert(uint32(ProofPlan.PlanVariant.Slice) == SLICE_EXEC_VARIANT);
        assert(uint32(ProofPlan.PlanVariant.GroupBy) == GROUP_BY_EXEC_VARIANT);
        assert(uint32(ProofPlan.PlanVariant.Union) == UNION_EXEC_VARIANT);
    }
}
