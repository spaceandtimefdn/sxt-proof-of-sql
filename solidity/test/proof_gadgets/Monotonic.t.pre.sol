// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {Errors} from "../../src/base/Errors.sol";
import {VerificationBuilder} from "../../src/builder/VerificationBuilder.pre.sol";
import {Monotonic} from "../../src/proof_gadgets/Monotonic.pre.sol";

contract MonotonicTest is Test {
    function testStrictlyIncreasingColumn() public pure {
        VerificationBuilder.Builder memory builder;
        builder.maxDegree = 3;
        builder.rowMultipliersEvaluation = 1;
        uint256 alpha = 3;
        uint256 beta = 8;
        uint256[3] memory shiftedColumn = [uint256(0), MODULUS_MINUS_ONE, 1];
        // cStar = 1 / (1 + alpha * (column + beta * (rho + chi)))
        uint256[3] memory cStarEval = [
            uint256(14923801958072233106077094826311778469464793909374568870703321036301687610648),
            21467315124303904544895513327079250567614742008100341375550161798372427563009,
            1
        ];
        // dStar = 1 / (1 + alpha * (shiftedColumn + beta * rhoPlusOne))
        uint256[3] memory dStarEval = [
            uint256(1),
            14923801958072233106077094826311778469464793909374568870703321036301687610648,
            21467315124303904544895513327079250567614742008100341375550161798372427563009
        ];
        uint256[3] memory sign = [uint256(1), 0, 1]; //Sign of [1, -2, 1];

        builder.firstRoundMLEs = new uint256[](3);
        builder.finalRoundMLEs = new uint256[](9);
        for (uint8 i = 0; i < 3; ++i) {
            builder.firstRoundMLEs[i] = shiftedColumn[i];
            builder.finalRoundMLEs[i * 3] = cStarEval[i];
            builder.finalRoundMLEs[i * 3 + 1] = dStarEval[i];
            builder.finalRoundMLEs[i * 3 + 2] = sign[i];
        }

        builder.constraintMultipliers = new uint256[](12);
        for (uint8 i = 0; i < 12; ++i) {
            builder.constraintMultipliers[i] = 1;
        }
        builder.rhoEvaluations = new uint256[](6);
        builder.rhoEvaluations[0] = 0;
        builder.rhoEvaluations[1] = 0;
        builder.rhoEvaluations[2] = 1;
        builder.rhoEvaluations[3] = 1;
        builder.rhoEvaluations[4] = 0;
        builder.rhoEvaluations[5] = 2;
        builder.chiEvaluations = new uint256[](6);
        builder.chiEvaluations[0] = 3;
        builder.chiEvaluations[1] = 1; // shifted_chi_eval
        builder.chiEvaluations[2] = 3;
        builder.chiEvaluations[3] = 1;
        builder.chiEvaluations[4] = 3;
        builder.chiEvaluations[5] = 1;

        uint256[] memory bitDistribution = new uint256[](6);
        bitDistribution[0] = 0x8000000000000000000000000000000000000000000000000000000000000000;
        bitDistribution[1] = 1;
        bitDistribution[2] = 0x8000000000000000000000000000000000000000000000000000000000000000;
        bitDistribution[3] = 1;
        bitDistribution[4] = 0x8000000000000000000000000000000000000000000000000000000000000000;
        bitDistribution[5] = 1;
        VerificationBuilder.__setBitDistributions(builder, bitDistribution);
        uint256[3] memory chi = [uint256(1), 1, 0];
        uint256[3] memory column = [MODULUS_MINUS_ONE, uint256(1), 0];

        for (uint8 i = 0; i < 3; ++i) {
            uint256 chiEval = chi[i];
            if (i == 0) {
                builder.singletonChiEvaluation = 1; // singleton_chi_eval for first test
            } else {
                builder.singletonChiEvaluation = 0;
            }
            Monotonic.__monotonicVerify({
                __builder: builder,
                __alpha: alpha,
                __beta: beta,
                __columnEval: column[i],
                __chiEval: chiEval,
                __strict: 1, // strict
                __asc: 1 // ascending
            });
        }

        assert(builder.aggregateEvaluation == 0);
    }

    function testNonStrictlyIncreasingColumn() public pure {
        VerificationBuilder.Builder memory builder;
        builder.maxDegree = 3;
        builder.rowMultipliersEvaluation = 1;
        uint256 alpha = 3;
        uint256 beta = 8;
        uint256[3] memory shiftedColumn = [uint256(0), 1, 1];
        // cStar = 1 / (1 + alpha * (column + beta * (rho + chi)))
        uint256[3] memory cStarEval = [
            uint256(11725844395628183154774860220673540226008052357365732684124037957094183122652),
            21467315124303904544895513327079250567614742008100341375550161798372427563009,
            1
        ];
        // dStar = 1 / (1 + alpha * (shiftedColumn + beta * rhoPlusOne))
        uint256[3] memory dStarEval = [
            uint256(1),
            11725844395628183154774860220673540226008052357365732684124037957094183122652,
            21467315124303904544895513327079250567614742008100341375550161798372427563009
        ];
        uint256[3] memory trail = [uint256(1), 0, 1]; //Trailing bit of bit mask of [1, 0, -1];
        uint256[3] memory sign = [uint256(1), 1, 0]; //Sign of [1, 0, -1];

        builder.firstRoundMLEs = new uint256[](3);
        builder.finalRoundMLEs = new uint256[](12);
        for (uint8 i = 0; i < 3; ++i) {
            builder.firstRoundMLEs[i] = shiftedColumn[i];
            builder.finalRoundMLEs[i * 4] = cStarEval[i];
            builder.finalRoundMLEs[i * 4 + 1] = dStarEval[i];
            builder.finalRoundMLEs[i * 4 + 2] = trail[i];
            builder.finalRoundMLEs[i * 4 + 3] = sign[i];
        }

        builder.constraintMultipliers = new uint256[](15);
        for (uint8 i = 0; i < 15; ++i) {
            builder.constraintMultipliers[i] = 1;
        }
        builder.rhoEvaluations = new uint256[](6);
        builder.rhoEvaluations[0] = 0;
        builder.rhoEvaluations[1] = 0;
        builder.rhoEvaluations[2] = 1;
        builder.rhoEvaluations[3] = 1;
        builder.rhoEvaluations[4] = 0;
        builder.rhoEvaluations[5] = 2;
        builder.chiEvaluations = new uint256[](3);
        builder.chiEvaluations = new uint256[](6);
        builder.chiEvaluations[0] = 3;
        builder.chiEvaluations[1] = 1; // shifted_chi_eval
        builder.chiEvaluations[2] = 3;
        builder.chiEvaluations[3] = 1;
        builder.chiEvaluations[4] = 3;
        builder.chiEvaluations[5] = 1;

        uint256[] memory bitDistribution = new uint256[](6);
        bitDistribution[0] = 0x8000000000000000000000000000000000000000000000000000000000000001;
        bitDistribution[1] = 0;
        bitDistribution[2] = 0x8000000000000000000000000000000000000000000000000000000000000001;
        bitDistribution[3] = 0;
        bitDistribution[4] = 0x8000000000000000000000000000000000000000000000000000000000000001;
        bitDistribution[5] = 0;
        VerificationBuilder.__setBitDistributions(builder, bitDistribution);
        uint256[3] memory chi = [uint256(1), 1, 0];
        uint256[3] memory column = [uint256(1), 1, 0];

        for (uint8 i = 0; i < 3; ++i) {
            uint256 chiEval = chi[i];
            if (i == 0) {
                builder.singletonChiEvaluation = 1; // singleton_chi_eval for first test
            } else {
                builder.singletonChiEvaluation = 0;
            }
            Monotonic.__monotonicVerify({
                __builder: builder,
                __alpha: alpha,
                __beta: beta,
                __columnEval: column[i],
                __chiEval: chiEval,
                __strict: 0, // non-strict
                __asc: 1 // ascending
            });
        }

        assert(builder.aggregateEvaluation == 0);
    }

    function testStrictlyDecreasingColumn() public pure {
        VerificationBuilder.Builder memory builder;
        builder.maxDegree = 3;
        builder.rowMultipliersEvaluation = 1;
        uint256 alpha = 3;
        uint256 beta = 8;
        uint256[3] memory shiftedColumn = [uint256(0), 1, MODULUS_MINUS_ONE];
        // cStar = 1 / (1 + alpha * (column + beta * (rho + chi)))
        uint256[3] memory cStarEval = [
            uint256(11725844395628183154774860220673540226008052357365732684124037957094183122652),
            4282482301012032108700383732767727734715984339211832806375735601721353836099,
            1
        ];
        // dStar = 1 / (1 + alpha * (shiftedColumn + beta * rhoPlusOne))
        uint256[3] memory dStarEval = [
            uint256(1),
            11725844395628183154774860220673540226008052357365732684124037957094183122652,
            4282482301012032108700383732767727734715984339211832806375735601721353836099
        ];
        uint256[3] memory sign = [uint256(1), 0, 1]; //Sign of [1, -2, 1];

        builder.firstRoundMLEs = new uint256[](3);
        builder.finalRoundMLEs = new uint256[](9);
        for (uint8 i = 0; i < 3; ++i) {
            builder.firstRoundMLEs[i] = shiftedColumn[i];
            builder.finalRoundMLEs[i * 3] = cStarEval[i];
            builder.finalRoundMLEs[i * 3 + 1] = dStarEval[i];
            builder.finalRoundMLEs[i * 3 + 2] = sign[i];
        }

        builder.constraintMultipliers = new uint256[](12);
        for (uint8 i = 0; i < 12; ++i) {
            builder.constraintMultipliers[i] = 1;
        }
        builder.rhoEvaluations = new uint256[](6);
        builder.rhoEvaluations[0] = 0;
        builder.rhoEvaluations[1] = 0;
        builder.rhoEvaluations[2] = 1;
        builder.rhoEvaluations[3] = 1;
        builder.rhoEvaluations[4] = 0;
        builder.rhoEvaluations[5] = 2;
        builder.chiEvaluations = new uint256[](3);
        builder.chiEvaluations = new uint256[](6);
        builder.chiEvaluations[0] = 3;
        builder.chiEvaluations[1] = 1; // shifted_chi_eval
        builder.chiEvaluations[2] = 3;
        builder.chiEvaluations[3] = 1;
        builder.chiEvaluations[4] = 3;
        builder.chiEvaluations[5] = 1;

        uint256[] memory bitDistribution = new uint256[](6);
        bitDistribution[0] = 0x8000000000000000000000000000000000000000000000000000000000000000;
        bitDistribution[1] = 1;
        bitDistribution[2] = 0x8000000000000000000000000000000000000000000000000000000000000000;
        bitDistribution[3] = 1;
        bitDistribution[4] = 0x8000000000000000000000000000000000000000000000000000000000000000;
        bitDistribution[5] = 1;
        VerificationBuilder.__setBitDistributions(builder, bitDistribution);
        uint256[3] memory chi = [uint256(1), 1, 0];
        uint256[3] memory column = [uint256(1), MODULUS_MINUS_ONE, 0];

        for (uint8 i = 0; i < 3; ++i) {
            uint256 chiEval = chi[i];
            if (i == 0) {
                builder.singletonChiEvaluation = 1; // singleton_chi_eval for first test
            } else {
                builder.singletonChiEvaluation = 0;
            }
            Monotonic.__monotonicVerify({
                __builder: builder,
                __alpha: alpha,
                __beta: beta,
                __columnEval: column[i],
                __chiEval: chiEval,
                __strict: 1, // strict
                __asc: 0 // descending
            });
        }

        assert(builder.aggregateEvaluation == 0);
    }

    function testNonStrictlyDecreasingColumn() public pure {
        VerificationBuilder.Builder memory builder;
        builder.maxDegree = 3;
        builder.rowMultipliersEvaluation = 1;
        uint256 alpha = 3;
        uint256 beta = 8;
        uint256[3] memory shiftedColumn = [uint256(0), 1, 1];
        // cStar = 1 / (1 + alpha * (column + beta * (rho + chi)))
        uint256[3] memory cStarEval = [
            uint256(11725844395628183154774860220673540226008052357365732684124037957094183122652),
            21467315124303904544895513327079250567614742008100341375550161798372427563009,
            1
        ];
        // dStar = 1 / (1 + alpha * (shiftedColumn + beta * rhoPlusOne))
        uint256[3] memory dStarEval = [
            uint256(1),
            11725844395628183154774860220673540226008052357365732684124037957094183122652,
            21467315124303904544895513327079250567614742008100341375550161798372427563009
        ];
        uint256[3] memory trail = [uint256(1), 0, 1]; //Trailing bit of bit mask of [-1, 0, 1];
        uint256[3] memory sign = [uint256(0), 1, 1]; //Sign of [-1, 0, 1];

        builder.firstRoundMLEs = new uint256[](3);
        builder.finalRoundMLEs = new uint256[](12);
        for (uint8 i = 0; i < 3; ++i) {
            builder.firstRoundMLEs[i] = shiftedColumn[i];
            builder.finalRoundMLEs[i * 4] = cStarEval[i];
            builder.finalRoundMLEs[i * 4 + 1] = dStarEval[i];
            builder.finalRoundMLEs[i * 4 + 2] = trail[i];
            builder.finalRoundMLEs[i * 4 + 3] = sign[i];
        }

        builder.constraintMultipliers = new uint256[](15);
        for (uint8 i = 0; i < 15; ++i) {
            builder.constraintMultipliers[i] = 1;
        }
        builder.rhoEvaluations = new uint256[](6);
        builder.rhoEvaluations[0] = 0;
        builder.rhoEvaluations[1] = 0;
        builder.rhoEvaluations[2] = 1;
        builder.rhoEvaluations[3] = 1;
        builder.rhoEvaluations[4] = 0;
        builder.rhoEvaluations[5] = 2;
        builder.chiEvaluations = new uint256[](3);
        builder.chiEvaluations = new uint256[](6);
        builder.chiEvaluations[0] = 3;
        builder.chiEvaluations[1] = 1; // shifted_chi_eval
        builder.chiEvaluations[2] = 3;
        builder.chiEvaluations[3] = 1;
        builder.chiEvaluations[4] = 3;
        builder.chiEvaluations[5] = 1;

        uint256[] memory bitDistribution = new uint256[](6);
        bitDistribution[0] = 0x8000000000000000000000000000000000000000000000000000000000000001;
        bitDistribution[1] = 0;
        bitDistribution[2] = 0x8000000000000000000000000000000000000000000000000000000000000001;
        bitDistribution[3] = 0;
        bitDistribution[4] = 0x8000000000000000000000000000000000000000000000000000000000000001;
        bitDistribution[5] = 0;
        VerificationBuilder.__setBitDistributions(builder, bitDistribution);
        uint256[3] memory chi = [uint256(1), 1, 0];
        uint256[3] memory column = [uint256(1), 1, 0];

        for (uint8 i = 0; i < 3; ++i) {
            uint256 chiEval = chi[i];
            if (i == 0) {
                builder.singletonChiEvaluation = 1; // singleton_chi_eval for first test
            } else {
                builder.singletonChiEvaluation = 0;
            }
            Monotonic.__monotonicVerify({
                __builder: builder,
                __alpha: alpha,
                __beta: beta,
                __columnEval: column[i],
                __chiEval: chiEval,
                __strict: 0, // non-strict
                __asc: 0 // descending
            });
        }

        assert(builder.aggregateEvaluation == 0);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testInvalidMonotonicColumn() public {
        VerificationBuilder.Builder memory builder;
        builder.maxDegree = 3;
        builder.rowMultipliersEvaluation = 1;
        uint256 alpha = 3;
        uint256 beta = 8;
        uint256[3] memory shiftedColumn = [uint256(0), MODULUS_MINUS_ONE, 1];
        // cStar = 1 / (1 + alpha * (column + beta * (rho + chi)))
        uint256[3] memory cStarEval = [
            uint256(14923801958072233106077094826311778469464793909374568870703321036301687610648),
            21467315124303904544895513327079250567614742008100341375550161798372427563009,
            1
        ];
        // dStar = 1 / (1 + alpha * (shiftedColumn + beta * rhoPlusOne))
        uint256[3] memory dStarEval = [
            uint256(1),
            14923801958072233106077094826311778469464793909374568870703321036301687610648,
            21467315124303904544895513327079250567614742008100341375550161798372427563009
        ];
        uint256[3] memory sign = [uint256(1), 0, 1]; //Sign of [1, -2, 1];

        builder.firstRoundMLEs = new uint256[](3);
        builder.finalRoundMLEs = new uint256[](9);
        for (uint8 i = 0; i < 3; ++i) {
            builder.firstRoundMLEs[i] = shiftedColumn[i];
            builder.finalRoundMLEs[i * 3] = cStarEval[i];
            builder.finalRoundMLEs[i * 3 + 1] = dStarEval[i];
            builder.finalRoundMLEs[i * 3 + 2] = sign[i];
        }

        builder.constraintMultipliers = new uint256[](12);
        for (uint8 i = 0; i < 12; ++i) {
            builder.constraintMultipliers[i] = 1;
        }
        builder.rhoEvaluations = new uint256[](6);
        builder.rhoEvaluations[0] = 0;
        builder.rhoEvaluations[1] = 0;
        builder.rhoEvaluations[2] = 1;
        builder.rhoEvaluations[3] = 1;
        builder.rhoEvaluations[4] = 0;
        builder.rhoEvaluations[5] = 2;
        builder.chiEvaluations = new uint256[](3);
        builder.chiEvaluations = new uint256[](6);
        builder.chiEvaluations[0] = 3;
        builder.chiEvaluations[1] = 1; // shifted_chi_eval
        builder.chiEvaluations[2] = 3;
        builder.chiEvaluations[3] = 1;
        builder.chiEvaluations[4] = 3;
        builder.chiEvaluations[5] = 1;

        uint256[] memory bitDistribution = new uint256[](6);
        bitDistribution[0] = 0x8000000000000000000000000000000000000000000000000000000000000000;
        bitDistribution[1] = 1;
        bitDistribution[2] = 0x8000000000000000000000000000000000000000000000000000000000000000;
        bitDistribution[3] = 1;
        bitDistribution[4] = 0x8000000000000000000000000000000000000000000000000000000000000000;
        bitDistribution[5] = 1;
        VerificationBuilder.__setBitDistributions(builder, bitDistribution);
        uint256[3] memory chi = [uint256(1), 1, 0];
        uint256[3] memory column = [MODULUS_MINUS_ONE, uint256(1), 0];

        vm.expectRevert(Errors.MonotonyCheckFailed.selector);
        for (uint8 i = 0; i < 3; ++i) {
            uint256 chiEval = chi[i];
            if (i == 0) {
                builder.singletonChiEvaluation = 1; // singleton_chi_eval for first test
            } else {
                builder.singletonChiEvaluation = 0;
            }
            Monotonic.__monotonicVerify({
                __builder: builder,
                __alpha: alpha,
                __beta: beta,
                __columnEval: column[i],
                __chiEval: chiEval,
                __strict: 0, // non-strict
                __asc: 0 // descending
            });
        }
    }
}
