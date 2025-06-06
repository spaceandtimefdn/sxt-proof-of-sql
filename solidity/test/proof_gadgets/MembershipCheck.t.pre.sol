// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
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
        builder.finalRoundMLEs = new uint256[](24);
        {
            uint256 inv901 = 6340545382408491490573043173709377134418560608777563777697260036288885701838;
            uint256 inv4 = 16416182153879456416684804308942956316411273300312025757773653139931856371713;
            uint256 inv7 = 3126891838834182174606629392179610726935480628630862049099743455225115499374;
            uint256[8] memory multiplicity = [uint256(2), 1, 1, 0, 0, 0, 1, 0];
            uint256[8] memory cStar = [inv901, inv4, inv4, inv4, inv4, inv4, 1, inv7];
            uint256[8] memory dStar = [inv901, inv901, 1, inv4, inv4, 0, 0, 0];
            for (uint8 i = 0; i < 8; ++i) {
                builder.finalRoundMLEs[i * 3] = multiplicity[i];
                builder.finalRoundMLEs[i * 3 + 1] = cStar[i];
                builder.finalRoundMLEs[i * 3 + 2] = dStar[i];
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

    /// forge-config: default.allow_internal_expect_revert = true
    function testMismatchingMembership() public {
        VerificationBuilder.Builder memory builder;

        uint256[] memory columnEval = new uint256[](2);
        columnEval[0] = 0;
        columnEval[1] = 0;
        uint256[] memory candidateEval = new uint256[](1);
        candidateEval[0] = 0;
        vm.expectRevert(Errors.MismatchingColumnAndCandidateLengths.selector);
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
