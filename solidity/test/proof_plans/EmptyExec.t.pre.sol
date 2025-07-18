// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import {VerificationBuilder} from "../../src/builder/VerificationBuilder.pre.sol";
import {EmptyExec} from "../../src/proof_plans/EmptyExec.pre.sol";

contract EmptyExecTest is Test {
    function testEmptyExec() public pure {
        VerificationBuilder.Builder memory builder;
        builder.singletonChiEvaluation = 1;
        uint256[] memory evaluations;
        uint256 one;
        uint256 singleton;
        (builder, evaluations, one, singleton) = EmptyExec.__emptyExecEvaluate(builder);
        assert(evaluations.length == 0);
        assert(one == 1);
        assert(singleton == 1);
    }
}
