// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {Errors} from "../../src/base/Errors.sol";
import {VerificationBuilder} from "../../src/builder/VerificationBuilder.pre.sol";
import {UnionExec} from "../../src/proof_plans/UnionExec.pre.sol";

contract UnionExecTest is Test {
    function testSimpleUnionExecTwoTables() public pure {
        // Plan for UNION of two tables
        bytes memory plan = abi.encodePacked(
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

        // challenges
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
            (planOutput, builder, evals, length, outputChiEval) = UnionExec.__unionExecEvaluate(plan, builder);
            assert(planOutput.length == 3);
            assert(length == 4);
        }

        assert(builder.aggregateEvaluation == 0);
    }

    function testCannotUnionNoTables() public {
        // Plan for UNION of no tables
        bytes memory plan = abi.encodePacked(
            uint64(0), // input_count: 0 tables
            hex"abcdef"
        );

        VerificationBuilder.Builder memory builder;

        // challenges
        builder.challenges = new uint256[](2);
        builder.challenges[0] = 3; // gamma
        builder.challenges[1] = 8; // beta

        bytes memory planOutput;
        uint256[] memory evals;
        uint256 outputChiEval;
        uint256 length;

        vm.expectRevert(Errors.UnionNotEnoughInputPlans.selector);
        (planOutput, builder, evals, length, outputChiEval) = UnionExec.__unionExecEvaluate(plan, builder);

        assert(builder.aggregateEvaluation == 0);
    }

    function testCannotUnionOneTable() public {
        // Plan for UNION of one table
        bytes memory plan = abi.encodePacked(
            uint64(1), // input_count: 1 table
            // First input plan (TableExec)
            TABLE_EXEC_VARIANT,
            uint64(0), // table_number
            uint64(1), // column_count
            uint64(0), // column1_index
            hex"abcdef"
        );

        VerificationBuilder.Builder memory builder;

        // challenges
        builder.challenges = new uint256[](2);
        builder.challenges[0] = 3; // gamma
        builder.challenges[1] = 8; // beta

        bytes memory planOutput;
        uint256[] memory evals;
        uint256 outputChiEval;
        uint256 length;

        vm.expectRevert(Errors.UnionNotEnoughInputPlans.selector);
        (planOutput, builder, evals, length, outputChiEval) = UnionExec.__unionExecEvaluate(plan, builder);

        assert(builder.aggregateEvaluation == 0);
    }

    function testCannotUnionTablesIfColumnCountIsZero() public {
        // Plan for UNION of two tables
        bytes memory plan = abi.encodePacked(
            uint64(2), // input_count: 2 tables
            // First input plan (TableExec)
            TABLE_EXEC_VARIANT,
            uint64(0), // table_number
            uint64(0), // column_count
            // Second input plan (TableExec)
            TABLE_EXEC_VARIANT,
            uint64(1), // table_number
            uint64(0), // column_count
            hex"abcdef"
        );

        VerificationBuilder.Builder memory builder;

        // challenges
        builder.challenges = new uint256[](2);
        builder.challenges[0] = 3; // gamma
        builder.challenges[1] = 8; // beta

        // chi evals for tables
        builder.tableChiEvaluations = new uint256[](4);
        builder.tableChiEvaluations[0] = 1; // table 1 has 1 row
        builder.tableChiEvaluations[1] = 1;
        builder.tableChiEvaluations[2] = 1; // table 2 has 1 row
        builder.tableChiEvaluations[3] = 1;

        bytes memory planOutput;
        uint256[] memory evals;
        uint256 outputChiEval;
        uint256 length;

        vm.expectRevert(Errors.UnionInvalidColumnCounts.selector);
        (planOutput, builder, evals, length, outputChiEval) = UnionExec.__unionExecEvaluate(plan, builder);

        assert(builder.aggregateEvaluation == 0);
    }

    function testCannotUnionTablesIfColumnCountsDiffer() public {
        // Plan for UNION of two tables
        bytes memory plan = abi.encodePacked(
            uint64(2), // input_count: 2 tables
            // First input plan (TableExec)
            TABLE_EXEC_VARIANT,
            uint64(0), // table_number
            uint64(1), // column_count
            uint64(0), // column1_index
            // Second input plan (TableExec)
            TABLE_EXEC_VARIANT,
            uint64(1), // table_number
            uint64(2), // column_count
            uint64(1), // column1_index
            uint64(2), // column2_index
            hex"abcdef"
        );

        VerificationBuilder.Builder memory builder;

        // challenges
        builder.challenges = new uint256[](2);
        builder.challenges[0] = 3; // gamma
        builder.challenges[1] = 8; // beta

        // chi evals for tables
        builder.tableChiEvaluations = new uint256[](4);
        builder.tableChiEvaluations[0] = 1; // table 1 has 1 row
        builder.tableChiEvaluations[1] = 1;
        builder.tableChiEvaluations[2] = 1; // table 2 has 1 row
        builder.tableChiEvaluations[3] = 1;

        // column evaluations for both tables
        builder.columnEvaluations = new uint256[](3);
        builder.columnEvaluations[0] = 1;
        builder.columnEvaluations[1] = 1;
        builder.columnEvaluations[2] = 1;

        // constraint multipliers
        builder.constraintMultipliers = new uint256[](1);
        builder.constraintMultipliers[0] = 1;

        builder.finalRoundMLEs = new uint256[](1);
        builder.finalRoundMLEs[0] = 16416182153879456416684804308942956316411273300312025757773653139931856371713; // inv4

        // max degree and row multipliers
        builder.maxDegree = 3;
        builder.rowMultipliersEvaluation = 1;

        bytes memory planOutput;
        uint256[] memory evals;
        uint256 outputChiEval;
        uint256 length;

        vm.expectRevert(Errors.UnionInvalidColumnCounts.selector);
        (planOutput, builder, evals, length, outputChiEval) = UnionExec.__unionExecEvaluate(plan, builder);

        assert(builder.aggregateEvaluation == 0);
    }
}
