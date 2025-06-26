// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {Errors} from "../../src/base/Errors.sol";
import {VerificationBuilder} from "../../src/builder/VerificationBuilder.pre.sol";
import {GroupByExec} from "../../src/proof_plans/GroupByExec.pre.sol";

contract GroupByExecTest is Test {
    function testUnprovableGroupByExec() public {
        bytes memory plan = abi.encodePacked(
            uint64(0), // table_number
            uint64(3), // total_column_count
            uint64(2), // group_by_count
            abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BOOLEAN_VARIANT, uint8(1)), // where clause
            abi.encodePacked(COLUMN_EXPR_VARIANT, uint64(0)), // group_by_expr[0] - column 0
            abi.encodePacked(COLUMN_EXPR_VARIANT, uint64(1)), // group_by_expr[1] - column 1
            uint64(0), // sum_count
            uint64(7), // count_alias_length (unused in verification)
            "count_0" // count_alias (unused in verification)
        );
        VerificationBuilder.Builder memory builder;
        builder.maxDegree = 3;
        builder.finalRoundMLEs = new uint256[](0);
        builder.constraintMultipliers = new uint256[](0);
        builder.challenges = new uint256[](2);
        builder.challenges[0] = 501; // alpha
        builder.challenges[1] = 502; // beta
        builder.aggregateEvaluation = 0;
        builder.rowMultipliersEvaluation = 601;
        builder.chiEvaluations = new uint256[](1);
        builder.chiEvaluations[0] = 701; // output_chi_eval
        builder.tableChiEvaluations = new uint256[](1);
        builder.tableChiEvaluations[0] = 801; // input_chi_eval
        // Define column evaluations for ColumnExpr
        builder.columnEvaluations = new uint256[](2);
        builder.columnEvaluations[0] = 102;
        builder.columnEvaluations[1] = 103;

        uint256[] memory evals;
        uint256 outputChiEval;
        vm.expectRevert(Errors.UnprovableGroupBy.selector);
        (plan, builder, evals, outputChiEval) = GroupByExec.__groupByExecEvaluate(plan, builder);
    }

    // Shared configuration for the builder in different tests
    function _configureBuilderForGroupByExec(VerificationBuilder.Builder memory builder)
        internal
        pure
        returns (VerificationBuilder.Builder memory configuredBuilder)
    {
        // table chi evals
        builder.tableChiEvaluations = new uint256[](1);
        builder.tableChiEvaluations[0] = 1;

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
                // Monotonicity check
                builder.firstRoundMLEs[i * 3 + 1] = shiftedGOut[i];
                builder.finalRoundMLEs[i * 5 + 1] = cStarEval[i];
                builder.finalRoundMLEs[i * 5 + 2] = dStarEval[i];
                builder.finalRoundMLEs[i * 5 + 3] = sign[i];
                // Continue with group by output
                builder.finalRoundMLEs[i * 5 + 4] = gOutStarColumn[i];
                builder.firstRoundMLEs[i * 3 + 2] = count[i];
            }
        }
        configuredBuilder = builder;
    }

    function _configureForSimpleGroupByExec(VerificationBuilder.Builder memory builder)
        internal
        pure
        returns (VerificationBuilder.Builder memory configuredBuilder)
    {
        // column evaluations
        builder.columnEvaluations = new uint256[](2);

        uint256[4] memory gOut = [MODULUS_MINUS_ONE, 1, 0, 0];
        uint256[4] memory sumOut = [MODULUS - 2, 5, 0, 0];
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
        builder.firstRoundMLEs = new uint256[](16); // 4 mles times 4 rows
        builder.finalRoundMLEs = new uint256[](20); // 5 mles times 4 rows
        {
            uint256 inv4 = 16416182153879456416684804308942956316411273300312025757773653139931856371713;
            uint256 invNegative2 = 10944121435919637611123202872628637544274182200208017171849102093287904247808;
            uint256[4] memory gInStarColumn = [invNegative2, inv4, inv4, invNegative2];
            uint256[4] memory gOutStarColumn = [invNegative2, inv4, 0, 0];

            for (uint8 i = 0; i < 4; ++i) {
                builder.finalRoundMLEs[i * 5] = gInStarColumn[i];
                builder.firstRoundMLEs[i * 4] = gOut[i];
                // Monotonicity check
                builder.firstRoundMLEs[i * 4 + 1] = shiftedGOut[i];
                builder.finalRoundMLEs[i * 5 + 1] = cStarEval[i];
                builder.finalRoundMLEs[i * 5 + 2] = dStarEval[i];
                builder.finalRoundMLEs[i * 5 + 3] = sign[i];
                // Continue with group by output
                builder.finalRoundMLEs[i * 5 + 4] = gOutStarColumn[i];
                builder.firstRoundMLEs[i * 4 + 2] = sumOut[i];
                builder.firstRoundMLEs[i * 4 + 3] = count[i];
            }
        }
        configuredBuilder = builder;
    }

    function _configureForComplexGroupByExec(VerificationBuilder.Builder memory builder)
        internal
        pure
        returns (VerificationBuilder.Builder memory configuredBuilder)
    {
        // column evaluations
        builder.columnEvaluations = new uint256[](3);

        uint256[4] memory gOut = [MODULUS_MINUS_ONE, 1, 0, 0];
        uint256[4] memory sumOut0 = [MODULUS - 2, 5, 0, 0];
        uint256[4] memory sumOut1 = [MODULUS - 5, 7, 0, 0];
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
        builder.firstRoundMLEs = new uint256[](20); // 5 mles times 4 rows
        builder.finalRoundMLEs = new uint256[](20); // 5 mles times 4 rows
        {
            uint256 inv4 = 16416182153879456416684804308942956316411273300312025757773653139931856371713;
            uint256 invNegative2 = 10944121435919637611123202872628637544274182200208017171849102093287904247808;
            uint256[4] memory gInStarColumn = [invNegative2, inv4, inv4, invNegative2];
            uint256[4] memory gOutStarColumn = [invNegative2, inv4, 0, 0];

            for (uint8 i = 0; i < 4; ++i) {
                builder.finalRoundMLEs[i * 5] = gInStarColumn[i];
                builder.firstRoundMLEs[i * 5] = gOut[i];
                // Monotonicity check
                builder.firstRoundMLEs[i * 5 + 1] = shiftedGOut[i];
                builder.finalRoundMLEs[i * 5 + 1] = cStarEval[i];
                builder.finalRoundMLEs[i * 5 + 2] = dStarEval[i];
                builder.finalRoundMLEs[i * 5 + 3] = sign[i];
                // Continue with group by output
                builder.finalRoundMLEs[i * 5 + 4] = gOutStarColumn[i];
                builder.firstRoundMLEs[i * 5 + 2] = sumOut0[i];
                builder.firstRoundMLEs[i * 5 + 3] = sumOut1[i];
                builder.firstRoundMLEs[i * 5 + 4] = count[i];
            }
        }
        configuredBuilder = builder;
    }

    function testMinimalGroupByExec() public pure {
        bytes memory plan = abi.encodePacked(
            uint64(0), // table_number
            uint64(2), // total_column_count
            uint64(1), // group_by_count
            uint64(0), // group_by_expr[0] - column 0
            abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BOOLEAN_VARIANT, uint8(1)), // where clause
            uint64(0), // sum_count
            uint64(7), // count_alias_length (unused in verification)
            "count_0" // count_alias (unused in verification)
        );
        VerificationBuilder.Builder memory builder;
        builder = _configureBuilderForGroupByExec(builder);
        builder = _configureForMinimalGroupByExec(builder);
        uint256[4] memory gIn = [MODULUS_MINUS_ONE, 1, 1, MODULUS_MINUS_ONE];
        for (uint8 i = 0; i < 4; ++i) {
            uint256[] memory evals;
            uint256 outputChiEval;
            bytes memory planOut;
            if (i == 0) {
                builder.singletonChiEvaluation = 1; // singleton_chi_eval for first test
            } else {
                builder.singletonChiEvaluation = 0;
            }
            builder.columnEvaluations[0] = gIn[i];
            (planOut, builder, evals, outputChiEval) = GroupByExec.__groupByExecEvaluate(plan, builder);
            assert(planOut.length == 0);
        }
        assert(builder.aggregateEvaluation == 0);
    }

    function testSimpleGroupByExec() public pure {
        bytes memory plan = abi.encodePacked(
            uint64(0), // table_number
            uint64(3), // total_column_count
            uint64(1), // group_by_count
            uint64(0), // group_by_expr[0] - column 0
            abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BOOLEAN_VARIANT, uint8(1)), // where clause
            uint64(1), // sum_count
            abi.encodePacked(COLUMN_EXPR_VARIANT, uint64(1)), // sum_expr[0] - column 1;
            uint64(7), // count_alias_length (unused in verification)
            "count_0" // count_alias (unused in verification)
        );
        VerificationBuilder.Builder memory builder;
        builder = _configureBuilderForGroupByExec(builder);
        builder = _configureForSimpleGroupByExec(builder);
        uint256[4] memory gIn = [MODULUS_MINUS_ONE, 1, 1, MODULUS_MINUS_ONE];
        uint256[4] memory sumIn = [MODULUS_MINUS_ONE, 2, 3, MODULUS_MINUS_ONE];
        for (uint8 i = 0; i < 4; ++i) {
            uint256[] memory evals;
            uint256 outputChiEval;
            bytes memory planOut;
            if (i == 0) {
                builder.singletonChiEvaluation = 1; // singleton_chi_eval for first test
            } else {
                builder.singletonChiEvaluation = 0;
            }
            builder.columnEvaluations[0] = gIn[i];
            builder.columnEvaluations[1] = sumIn[i];
            (planOut, builder, evals, outputChiEval) = GroupByExec.__groupByExecEvaluate(plan, builder);
            assert(planOut.length == 0);
        }
        assert(builder.aggregateEvaluation == 0);
    }

    function testComplexGroupByExec() public pure {
        bytes memory plan = abi.encodePacked(
            uint64(0), // table_number
            uint64(4), // total_column_count
            uint64(1), // group_by_count
            uint64(0), // group_by_expr[0] - column 0
            abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BOOLEAN_VARIANT, uint8(1)), // where clause
            uint64(2), // sum_count
            abi.encodePacked(COLUMN_EXPR_VARIANT, uint64(1)), // sum_expr[0] - column 1;
            abi.encodePacked(COLUMN_EXPR_VARIANT, uint64(2)), // sum_expr[1] - column 2;
            uint64(7), // count_alias_length (unused in verification)
            "count_0" // count_alias (unused in verification)
        );
        VerificationBuilder.Builder memory builder;
        builder = _configureBuilderForGroupByExec(builder);
        builder = _configureForComplexGroupByExec(builder);
        uint256[4] memory gIn = [MODULUS_MINUS_ONE, 1, 1, MODULUS_MINUS_ONE];
        uint256[4] memory sumIn0 = [MODULUS_MINUS_ONE, 2, 3, MODULUS_MINUS_ONE];
        uint256[4] memory sumIn1 = [MODULUS_MINUS_ONE, 5, 7, MODULUS_MINUS_ONE];
        for (uint8 i = 0; i < 4; ++i) {
            uint256[] memory evals;
            uint256 outputChiEval;
            bytes memory planOut;
            if (i == 0) {
                builder.singletonChiEvaluation = 1; // singleton_chi_eval for first test
            } else {
                builder.singletonChiEvaluation = 0;
            }
            builder.columnEvaluations[0] = gIn[i];
            builder.columnEvaluations[1] = sumIn0[i];
            builder.columnEvaluations[2] = sumIn1[i];
            (planOut, builder, evals, outputChiEval) = GroupByExec.__groupByExecEvaluate(plan, builder);
            assert(planOut.length == 0);
        }
    }
}
