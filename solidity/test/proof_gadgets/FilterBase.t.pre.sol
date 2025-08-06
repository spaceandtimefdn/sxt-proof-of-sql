// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {VerificationBuilder} from "../../src/builder/VerificationBuilder.pre.sol";
import {FilterBase} from "../../src/proof_gadgets/FilterBase.pre.sol";

contract FilterBaseTest is Test {
    function testSimpleSliceExec() public pure {
        VerificationBuilder.Builder memory builder;

        // max degree and row multipliers
        builder.maxDegree = 3;
        builder.rowMultipliersEvaluation = 1;

        // constraint multipliers
        builder.constraintMultipliers = new uint256[](16); // 4 constraints times 4 rows
        for (uint8 i = 0; i < 16; ++i) {
            builder.constraintMultipliers[i] = 1;
        }

        // final round mles
        builder.finalRoundMLEs = new uint256[](8); // 2 mles times 4 rows
        {
            uint256 inv4 = 16416182153879456416684804308942956316411273300312025757773653139931856371713;
            uint256 inv10 = 15321770010287492655572484021680092561983855080291224040588742930603065946932;
            uint256 invNegative2 = 10944121435919637611123202872628637544274182200208017171849102093287904247808;
            uint256 inv1201 = 17860514583182755801666509267570465934036134148549303627663813574391583951461;
            uint256[4] memory cStarColumn = [inv4, inv10, invNegative2, inv1201];
            uint256[4] memory dStarColumn = [cStarColumn[0], cStarColumn[2], 0, 0];

            for (uint8 i = 0; i < 4; ++i) {
                builder.finalRoundMLEs[i * 2] = cStarColumn[i];
                builder.finalRoundMLEs[i * 2 + 1] = dStarColumn[i];
            }
        }
        builder.aggregateEvaluation = 0;

        uint256[4] memory inputChiEval = [uint256(1), 1, 1, 1];
        uint256[4] memory outputChiEval = [uint256(1), 1, 0, 0];
        uint256[4] memory selectionEval = [uint256(1), 0, 1, 0];
        uint256[4] memory cFoldEval = [uint256(3), 9, MODULUS - 3, 1200];
        uint256[4] memory dFoldEval = [uint256(3), MODULUS - 3, 0, 0];
        for (uint8 i = 0; i < 4; ++i) {
            builder = FilterBase.__filterBaseEvaluate({
                __builder: builder,
                __cFold: cFoldEval[i],
                __dFold: dFoldEval[i],
                __inputChiEval: inputChiEval[i],
                __outputChiEval: outputChiEval[i],
                __selectionEval: selectionEval[i]
            });
        }

        assert(builder.aggregateEvaluation == 0);
    }

    function testFilterBaseEvaluateFromColumnEvals() public pure {
        VerificationBuilder.Builder memory builder;

        // max degree and row multipliers
        builder.maxDegree = 3;
        builder.rowMultipliersEvaluation = 1;

        // constraint multipliers
        builder.constraintMultipliers = new uint256[](16); // 4 constraints times 4 rows
        for (uint8 i = 0; i < 16; ++i) {
            builder.constraintMultipliers[i] = 1;
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

        uint256[4] memory columnEval = [uint256(1), 3, MODULUS_MINUS_ONE, 400];

        // first round mles
        builder.firstRoundMLEs = new uint256[](4); // 1 mle times 4 rows
        {
            uint256[4] memory filteredColumn = [columnEval[0], columnEval[2], 0, 0];
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
            uint256[4] memory dStarColumn = [cStarColumn[0], cStarColumn[2], 0, 0];

            for (uint8 i = 0; i < 4; ++i) {
                builder.finalRoundMLEs[i * 2] = cStarColumn[i];
                builder.finalRoundMLEs[i * 2 + 1] = dStarColumn[i];
            }
        }
        builder.aggregateEvaluation = 0;
        uint256[4] memory inputChiEval = [uint256(1), 1, 1, 1];
        uint256[4] memory outputChiEval = [uint256(1), 1, 0, 0];
        uint256[4] memory selectionEval = [uint256(1), 0, 1, 0];
        for (uint8 i = 0; i < 4; ++i) {
            uint256[] memory columnEvals = new uint256[](1);
            columnEvals[0] = columnEval[i];
            uint256[] memory filteredColumnEvals;
            (builder, filteredColumnEvals) = FilterBase.__filterBaseEvaluateFromColumnEvals({
                __builder: builder,
                __columnEvals: columnEvals,
                __inputChiEval: inputChiEval[i],
                __outputChiEval: outputChiEval[i],
                __selectionEval: selectionEval[i]
            });
        }

        assert(builder.aggregateEvaluation == 0);
    }

    function testFilterBaseEvaluateFromExprEvals() public pure {
        bytes memory plan = abi.encodePacked(COLUMN_EXPR_VARIANT, uint64(0));
        VerificationBuilder.Builder memory builder;

        // max degree and row multipliers
        builder.maxDegree = 3;
        builder.rowMultipliersEvaluation = 1;

        // constraint multipliers
        builder.constraintMultipliers = new uint256[](16); // 4 constraints times 4 rows
        for (uint8 i = 0; i < 16; ++i) {
            builder.constraintMultipliers[i] = 1;
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

        uint256[4] memory columnEval = [uint256(1), 3, MODULUS_MINUS_ONE, 400];

        // first round mles
        builder.firstRoundMLEs = new uint256[](4); // 1 mle times 4 rows
        {
            uint256[4] memory filteredColumn = [columnEval[0], columnEval[2], 0, 0];
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
            uint256[4] memory dStarColumn = [cStarColumn[0], cStarColumn[2], 0, 0];

            for (uint8 i = 0; i < 4; ++i) {
                builder.finalRoundMLEs[i * 2] = cStarColumn[i];
                builder.finalRoundMLEs[i * 2 + 1] = dStarColumn[i];
            }
        }
        builder.aggregateEvaluation = 0;
        uint256[4] memory inputChiEval = [uint256(1), 1, 1, 1];
        uint256[4] memory outputChiEval = [uint256(1), 1, 0, 0];
        uint256[4] memory selectionEval = [uint256(1), 0, 1, 0];
        builder.columnEvaluations = new uint256[](1);
        for (uint8 i = 0; i < 4; ++i) {
            builder.columnEvaluations[0] = columnEval[i];
            uint256[] memory filteredColumnEvals;
            (builder, filteredColumnEvals) = FilterBase.__filterBaseEvaluateFromExprEvals({
                __plan: plan,
                __builder: builder,
                __numColumns: 1,
                __inputChiEval: inputChiEval[i],
                __outputChiEval: outputChiEval[i],
                __selectionEval: selectionEval[i]
            });
        }

        assert(builder.aggregateEvaluation == 0);
    }
}
