// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {Errors} from "../../src/base/Errors.sol";
import {VerificationBuilder} from "../../src/builder/VerificationBuilder.pre.sol";
import {SliceExec} from "../../src/proof_plans/SliceExec.pre.sol";

contract SliceExecTest is Test {
    function testSimpleSliceExec() public pure {
        bytes memory plan = abi.encodePacked(
            TABLE_EXEC_VARIANT,
            uint64(0), // table_number
            uint64(1), // column_count
            uint64(0), // column1_index
            uint64(2), // skip 2
            false, // fetch not set
            hex"abcdef"
        );

        VerificationBuilder.Builder memory builder;

        // column evaluations
        builder.columnEvaluations = new uint256[](1);
        uint256[4] memory column = [uint256(1), 3, MODULUS_MINUS_ONE, 400];

        // chi evals
        builder.tableChiEvaluations = new uint256[](2);
        builder.tableChiEvaluations[0] = 4;
        builder.tableChiEvaluations[1] = 1;
        builder.chiEvaluations = new uint256[](24); // 3 chi evaluations times 4 rows times 2 slots per element
        {
            uint256[4] memory outputChiEval = [uint256(1), 1, 0, 0];
            // This has length 2 because there are 2 rows selected
            uint256[4] memory offsetChiEval = [uint256(1), 1, 0, 0]; // This has length 2 because 2 are skipped
            uint256[4] memory maxChiEval = [uint256(1), 1, 1, 1]; // This has the full length because fetch is not set
            for (uint8 i = 0; i < 4; ++i) {
                builder.chiEvaluations[i * 6] = 2;
                builder.chiEvaluations[i * 6 + 1] = outputChiEval[i];
                builder.chiEvaluations[i * 6 + 2] = 2;
                builder.chiEvaluations[i * 6 + 3] = offsetChiEval[i];
                builder.chiEvaluations[i * 6 + 4] = 4;
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
        builder.firstRoundMLEs = new uint256[](4); // 1 mle times 4 rows
        {
            uint256[4] memory filteredColumn = [column[2], column[3], 0, 0];
            for (uint8 i = 0; i < 4; ++i) {
                builder.firstRoundMLEs[i] = filteredColumn[i];
            }
        }

        // final round mles
        builder.finalRoundMLEs = new uint256[](8); // 2 mles times 4 rows
        {
            uint256 inv4 = 16416182153879456416684804308942956316411273300312025757773653139931856371713;
            uint256 inv10 = 15321770010287492655572484021680092561983855080291224040588742930603065946932;
            uint256 invNegative2 = 10944121435919637611123202872628637544274182200208017171849102093287904247808;
            uint256 inv1201 = 17860514583182755801666509267570465934036134148549303627663813574391583951461;
            uint256[4] memory cStarColumn = [inv4, inv10, invNegative2, inv1201];
            uint256[4] memory dStarColumn = [cStarColumn[2], cStarColumn[3], 0, 0];

            for (uint8 i = 0; i < 4; ++i) {
                builder.finalRoundMLEs[i * 2] = cStarColumn[i];
                builder.finalRoundMLEs[i * 2 + 1] = dStarColumn[i];
            }
        }
        builder.aggregateEvaluation = 0;

        for (uint8 i = 0; i < 4; ++i) {
            bytes memory planOutput;
            uint256[] memory evals;
            uint256 outputChiEval;
            uint256 length;
            builder.columnEvaluations[0] = column[i];
            (planOutput, builder, evals, length, outputChiEval) = SliceExec.__sliceExecEvaluate(plan, builder);
            assert(planOutput.length == 3);
        }

        assert(builder.aggregateEvaluation == 0);
    }

    function testMultiColumnSliceExec() public pure {
        bytes memory plan = abi.encodePacked(
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
        builder.firstRoundMLEs = new uint256[](8); // 2 mles times 4 rows
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
            uint256 outputChiEval;
            uint256 length;
            builder.columnEvaluations[0] = column[i][0];
            builder.columnEvaluations[1] = column[i][1];
            (planOutput, builder, evals, length, outputChiEval) = SliceExec.__sliceExecEvaluate(plan, builder);
            assert(planOutput.length == 3);
        }

        assert(builder.aggregateEvaluation == 0);
    }

    function testSliceOffsetPlanValueMismatch() public {
        bytes memory plan = abi.encodePacked(
            TABLE_EXEC_VARIANT,
            uint64(0), // table_number
            uint64(1), // column_count
            uint64(0), // column1_index
            uint64(1), // skip 1
            false // fetch not set
        );

        VerificationBuilder.Builder memory builder;

        // column evaluations
        builder.columnEvaluations = new uint256[](1);
        uint256[4] memory column = [uint256(1), 3, MODULUS_MINUS_ONE, 400];

        // chi evals
        builder.tableChiEvaluations = new uint256[](2);
        builder.tableChiEvaluations[0] = 4;
        builder.tableChiEvaluations[1] = 1;
        builder.chiEvaluations = new uint256[](24); // 3 chi evaluations times 4 rows times 2 slots per element
        {
            uint256[4] memory outputChiEval = [uint256(1), 1, 0, 0];
            // This has length 2 because there are 2 rows selected
            uint256[4] memory offsetChiEval = [uint256(1), 1, 0, 0]; // This has length 2 because 2 are skipped
            uint256[4] memory maxChiEval = [uint256(1), 1, 1, 1]; // This has the full length because fetch is not set
            for (uint8 i = 0; i < 4; ++i) {
                builder.chiEvaluations[i * 6] = 2;
                builder.chiEvaluations[i * 6 + 1] = outputChiEval[i];
                builder.chiEvaluations[i * 6 + 2] = 2;
                builder.chiEvaluations[i * 6 + 3] = offsetChiEval[i];
                builder.chiEvaluations[i * 6 + 4] = 4;
                builder.chiEvaluations[i * 6 + 5] = maxChiEval[i];
            }
        }

        // challenges
        builder.challenges = new uint256[](0);

        // max degree and row multipliers
        builder.maxDegree = 3;
        builder.rowMultipliersEvaluation = 1;

        builder.aggregateEvaluation = 0;
        {
            bytes memory planOutput;
            uint256[] memory evals;
            uint256 outputChiEval;
            uint256 length;
            builder.columnEvaluations[0] = column[0];
            vm.expectRevert(Errors.SliceOffsetPlanValueMismatch.selector);
            (planOutput, builder, evals, length, outputChiEval) = SliceExec.__sliceExecEvaluate(plan, builder);
        }
    }

    function testSliceOffsetSelectionSizeMismatch() public {
        bytes memory plan = abi.encodePacked(
            TABLE_EXEC_VARIANT,
            uint64(0), // table_number
            uint64(1), // column_count
            uint64(0), // column1_index
            uint64(2), // skip 2
            false // fetch not set
        );

        VerificationBuilder.Builder memory builder;

        // column evaluations
        builder.columnEvaluations = new uint256[](1);
        uint256[4] memory column = [uint256(1), 3, MODULUS_MINUS_ONE, 400];

        // chi evals
        builder.tableChiEvaluations = new uint256[](2);
        builder.tableChiEvaluations[0] = 4;
        builder.tableChiEvaluations[1] = 1;
        builder.chiEvaluations = new uint256[](24); // 3 chi evaluations times 4 rows times 2 slots per element
        {
            uint256[4] memory outputChiEval = [uint256(1), 1, 0, 0];
            // This has length 2 because there are 2 rows selected
            uint256[4] memory offsetChiEval = [uint256(1), 1, 0, 0]; // This has length 2 because 2 are skipped
            uint256[4] memory maxChiEval = [uint256(1), 1, 1, 1]; // This has the full length because fetch is not set
            for (uint8 i = 0; i < 4; ++i) {
                builder.chiEvaluations[i * 6] = 2;
                builder.chiEvaluations[i * 6 + 1] = outputChiEval[i];
                builder.chiEvaluations[i * 6 + 2] = 2;
                builder.chiEvaluations[i * 6 + 3] = offsetChiEval[i];
                builder.chiEvaluations[i * 6 + 4] = 3;
                builder.chiEvaluations[i * 6 + 5] = maxChiEval[i];
            }
        }

        // challenges
        builder.challenges = new uint256[](0);

        // max degree and row multipliers
        builder.maxDegree = 3;
        builder.rowMultipliersEvaluation = 1;

        builder.aggregateEvaluation = 0;
        {
            bytes memory planOutput;
            uint256[] memory evals;
            uint256 outputChiEval;
            uint256 length;
            builder.columnEvaluations[0] = column[0];
            vm.expectRevert(Errors.SliceOffsetSelectionSizeMismatch.selector);
            (planOutput, builder, evals, length, outputChiEval) = SliceExec.__sliceExecEvaluate(plan, builder);
        }
    }

    function testSliceMaxLengthMismatchNoFetch() public {
        bytes memory plan = abi.encodePacked(
            TABLE_EXEC_VARIANT,
            uint64(0), // table_number
            uint64(1), // column_count
            uint64(0), // column1_index
            uint64(2), // skip 2
            false // fetch not set
        );

        VerificationBuilder.Builder memory builder;

        // column evaluations
        builder.columnEvaluations = new uint256[](1);
        uint256[4] memory column = [uint256(1), 3, MODULUS_MINUS_ONE, 400];

        // chi evals
        builder.tableChiEvaluations = new uint256[](2);
        builder.tableChiEvaluations[0] = 3;
        builder.tableChiEvaluations[1] = 1;
        builder.chiEvaluations = new uint256[](24); // 3 chi evaluations times 4 rows times 2 slots per element
        {
            uint256[4] memory outputChiEval = [uint256(1), 1, 0, 0];
            // This has length 2 because there are 2 rows selected
            uint256[4] memory offsetChiEval = [uint256(1), 1, 0, 0]; // This has length 2 because 2 are skipped
            uint256[4] memory maxChiEval = [uint256(1), 1, 1, 1]; // This has the full length because fetch is not set
            for (uint8 i = 0; i < 4; ++i) {
                builder.chiEvaluations[i * 6] = 2;
                builder.chiEvaluations[i * 6 + 1] = outputChiEval[i];
                builder.chiEvaluations[i * 6 + 2] = 2;
                builder.chiEvaluations[i * 6 + 3] = offsetChiEval[i];
                builder.chiEvaluations[i * 6 + 4] = 4;
                builder.chiEvaluations[i * 6 + 5] = maxChiEval[i];
            }
        }

        // challenges
        builder.challenges = new uint256[](0);

        // max degree and row multipliers
        builder.maxDegree = 3;
        builder.rowMultipliersEvaluation = 1;

        builder.aggregateEvaluation = 0;
        {
            bytes memory planOutput;
            uint256[] memory evals;
            uint256 outputChiEval;
            uint256 length;
            builder.columnEvaluations[0] = column[0];
            vm.expectRevert(Errors.SliceMaxLengthMismatch.selector);
            (planOutput, builder, evals, length, outputChiEval) = SliceExec.__sliceExecEvaluate(plan, builder);
        }
    }

    function testSliceMaxLengthMismatchWithFetch() public {
        bytes memory plan = abi.encodePacked(
            TABLE_EXEC_VARIANT,
            uint64(0), // table_number
            uint64(1), // column_count
            uint64(0), // column1_index
            uint64(2), // skip 2
            true, // fetch set
            uint64(2) // fetch 2
        );

        VerificationBuilder.Builder memory builder;

        // column evaluations
        builder.columnEvaluations = new uint256[](1);
        uint256[4] memory column = [uint256(1), 3, MODULUS_MINUS_ONE, 400];

        // chi evals
        builder.tableChiEvaluations = new uint256[](2);
        builder.tableChiEvaluations[0] = 2;
        builder.tableChiEvaluations[1] = 1;
        builder.chiEvaluations = new uint256[](24); // 3 chi evaluations times 4 rows times 2 slots per element
        {
            uint256[4] memory outputChiEval = [uint256(1), 1, 0, 0];
            // This has length 2 because there are 2 rows selected
            uint256[4] memory offsetChiEval = [uint256(1), 1, 0, 0]; // This has length 2 because 2 are skipped
            uint256[4] memory maxChiEval = [uint256(1), 1, 1, 1]; // This has the full length because fetch is not set
            for (uint8 i = 0; i < 4; ++i) {
                builder.chiEvaluations[i * 6] = 2;
                builder.chiEvaluations[i * 6 + 1] = outputChiEval[i];
                builder.chiEvaluations[i * 6 + 2] = 2;
                builder.chiEvaluations[i * 6 + 3] = offsetChiEval[i];
                builder.chiEvaluations[i * 6 + 4] = 4;
                builder.chiEvaluations[i * 6 + 5] = maxChiEval[i];
            }
        }

        // challenges
        builder.challenges = new uint256[](0);

        // max degree and row multipliers
        builder.maxDegree = 3;
        builder.rowMultipliersEvaluation = 1;

        builder.aggregateEvaluation = 0;
        {
            bytes memory planOutput;
            uint256[] memory evals;
            uint256 outputChiEval;
            uint256 length;
            builder.columnEvaluations[0] = column[0];
            vm.expectRevert(Errors.SliceMaxLengthMismatch.selector);
            (planOutput, builder, evals, length, outputChiEval) = SliceExec.__sliceExecEvaluate(plan, builder);
        }
    }
}
