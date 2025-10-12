// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {VerificationBuilder} from "../../src/builder/VerificationBuilder.pre.sol";
import {Shift} from "../../src/proof_gadgets/Shift.pre.sol";
import {F} from "../base/FieldUtil.sol";

contract ShiftTest is Test {
    function testSimpleShift() public pure {
        VerificationBuilder.Builder memory builder;
        builder.maxDegree = 3;
        builder.rowMultipliersEvaluation = 1;
        uint256 alpha = 3;
        uint256 beta = 8;
        uint256[3] memory column = [uint256(700), F.from(-6).into(), 0];
        uint256[3] memory shiftedColumn = [uint256(0), 700, F.from(-6).into()];
        uint256[3] memory chi = [uint256(1), 1, 0];
        builder.constraintMultipliers = new uint256[](9);
        for (uint8 i = 0; i < 9; ++i) {
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
        builder.chiEvaluations[1] = 1;
        builder.chiEvaluations[2] = 3;
        builder.chiEvaluations[3] = 1;
        builder.chiEvaluations[4] = 3;
        builder.chiEvaluations[5] = 1;
        builder.firstRoundMLEs = new uint256[](3);
        builder.firstRoundMLEs[0] = shiftedColumn[0];
        builder.firstRoundMLEs[1] = shiftedColumn[1];
        builder.firstRoundMLEs[2] = shiftedColumn[2];
        builder.finalRoundMLEs = new uint256[](6);
        builder.finalRoundMLEs[0] = 17222184509042479139574583720503606563789583659997933845959245835272824378669;
        builder.finalRoundMLEs[1] = 1;
        builder.finalRoundMLEs[2] = 2824289402817970996418891063904164527554627664569810883057832798267846257499;
        builder.finalRoundMLEs[3] = 17222184509042479139574583720503606563789583659997933845959245835272824378669;
        builder.finalRoundMLEs[4] = 1;
        builder.finalRoundMLEs[5] = 2824289402817970996418891063904164527554627664569810883057832798267846257499;

        for (uint8 i = 0; i < 3; ++i) {
            uint256 chiEval = chi[i];
            Shift.__shiftEvaluate({
                __builder: builder,
                __alpha: alpha,
                __beta: beta,
                __exprEval: F.from(column[i]).into(),
                __chiEval: chiEval
            });
        }
        assert(builder.aggregateEvaluation == 0);
    }
}
