// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {Errors} from "../../src/base/Errors.sol";
import {VerificationBuilder} from "../../src/builder/VerificationBuilder.pre.sol";
import {MembershipCheck} from "../../src/proof_gadgets/MembershipCheck.pre.sol";

contract MembershipCheckTest is Test {
    function testSimpleMembership() public pure {
        VerificationBuilder.Builder memory builder;
        builder.maxDegree = 3;
        builder.rowMultipliersEvaluation = 1;
        uint256 alpha = 3;
        uint256 beta = 8;
        uint256[8] memory column = [uint256(300), 1, 1, 1, 1, 1, 0, 2];
        uint256[8] memory chiN = [uint256(1), 1, 1, 1, 1, 1, 1, 1];
        uint256[8] memory candidate = [uint256(300), 300, 0, 1, 1, 0, 0, 0];
        uint256[8] memory chiM = [uint256(1), 1, 1, 1, 1, 0, 0, 0];
        builder.firstRoundMLEs = new uint256[](8);
        builder.finalRoundMLEs = new uint256[](16);
        {
            uint256 inv901 = 6340545382408491490573043173709377134418560608777563777697260036288885701838;
            uint256 inv4 = 16416182153879456416684804308942956316411273300312025757773653139931856371713;
            uint256 inv7 = 3126891838834182174606629392179610726935480628630862049099743455225115499374;
            uint256[8] memory multiplicity = [uint256(2), 1, 1, 0, 0, 0, 1, 0];
            uint256[8] memory cStar = [inv901, inv4, inv4, inv4, inv4, inv4, 1, inv7];
            uint256[8] memory dStar = [inv901, inv901, 1, inv4, inv4, 0, 0, 0];
            for (uint8 i = 0; i < 8; ++i) {
                builder.firstRoundMLEs[i] = multiplicity[i];
                builder.finalRoundMLEs[i * 2] = cStar[i];
                builder.finalRoundMLEs[i * 2 + 1] = dStar[i];
            }
        }

        builder.constraintMultipliers = new uint256[](24);
        for (uint8 i = 0; i < 24; ++i) {
            builder.constraintMultipliers[i] = 1;
        }

        for (uint8 i = 0; i < 8; ++i) {
            uint256[] memory columnEval = new uint256[](1);
            columnEval[0] = column[i];
            uint256[] memory candidateEval = new uint256[](1);
            candidateEval[0] = candidate[i];
            MembershipCheck.__membershipCheckEvaluate({
                __builder: builder,
                __alpha: alpha,
                __beta: beta,
                __chiNEval: chiN[i],
                __chiMEval: chiM[i],
                __columnEvals: columnEval,
                __candidateEvals: candidateEval
            });
        }
        assert(builder.aggregateEvaluation == 0);
    }

    function testMultiColumnMembership() public pure {
        VerificationBuilder.Builder memory builder;
        builder.maxDegree = 3;
        builder.rowMultipliersEvaluation = 1;
        uint256 alpha = 3;
        uint256 beta = 8;
        uint256[2][8] memory column = [
            [uint256(300), 2],
            [uint256(1), 1],
            [uint256(MODULUS_MINUS_ONE), 3],
            [uint256(1), 1],
            [uint256(MODULUS_MINUS_ONE), 3],
            [uint256(1), 1],
            [uint256(5), 0],
            [uint256(2), 1]
        ];
        uint256[2][8] memory candidate = [
            [uint256(300), 2],
            [uint256(MODULUS_MINUS_ONE), 3],
            [uint256(5), 0],
            [uint256(1), 1],
            [uint256(1), 1],
            [uint256(0), 0],
            [uint256(0), 0],
            [uint256(0), 0]
        ];
        uint256[8] memory chiM = [uint256(1), 1, 1, 1, 1, 0, 0, 0];
        builder.firstRoundMLEs = new uint256[](8);
        builder.finalRoundMLEs = new uint256[](16);
        {
            uint256 inv7207 = 5767416846624501685451078744310193616366497016288337618798791418108430738612;
            uint256 inv28 = 11725844395628183154774860220673540226008052357365732684124037957094183122652;
            uint256 invNegative14 = 20324796952422184134943091049167469725080624086100603319148332458963250745930;
            uint256 inv121 = 18632140626441697090011403237698341604301500274734310226453843233200894835112;
            uint256 inv52 = 21467315124303904544895513327079250567614742008100341375550161798372427563009;
            uint256[8] memory multiplicity = [uint256(1), 0, 1, 2, 0, 0, 1, 0];
            uint256[8] memory cStar = [inv7207, inv28, invNegative14, inv28, invNegative14, inv28, inv121, inv52];
            uint256[8] memory dStar = [inv7207, invNegative14, inv121, inv28, inv28, 0, 0, 0];
            for (uint8 i = 0; i < 8; ++i) {
                builder.firstRoundMLEs[i] = multiplicity[i];
                builder.finalRoundMLEs[i * 2] = cStar[i];
                builder.finalRoundMLEs[i * 2 + 1] = dStar[i];
            }
        }

        builder.constraintMultipliers = new uint256[](24);
        for (uint8 i = 0; i < 24; ++i) {
            builder.constraintMultipliers[i] = 1;
        }

        for (uint8 i = 0; i < 1; ++i) {
            uint256[] memory columnEval = new uint256[](2);
            columnEval[0] = column[i][0];
            columnEval[1] = column[i][1];
            uint256[] memory candidateEval = new uint256[](2);
            candidateEval[0] = candidate[i][0];
            candidateEval[1] = candidate[i][1];
            MembershipCheck.__membershipCheckEvaluate({
                __builder: builder,
                __alpha: alpha,
                __beta: beta,
                __chiNEval: 1,
                __chiMEval: chiM[i],
                __columnEvals: columnEval,
                __candidateEvals: candidateEval
            });
        }
        assert(builder.aggregateEvaluation == 0);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testMismatchingColumnAndCandidateArrayLengths() public {
        VerificationBuilder.Builder memory builder;

        uint256[] memory columnEval = new uint256[](2);
        columnEval[0] = 0;
        columnEval[1] = 0;
        uint256[] memory candidateEval = new uint256[](1);
        candidateEval[0] = 0;
        // Check should fail because the lengths of the column evals and candidate evals differ
        vm.expectRevert(Errors.InternalError.selector);
        MembershipCheck.__membershipCheckEvaluate({
            __builder: builder,
            __alpha: 0,
            __beta: 0,
            __chiNEval: 0,
            __chiMEval: 0,
            __columnEvals: columnEval,
            __candidateEvals: candidateEval
        });
    }
}
