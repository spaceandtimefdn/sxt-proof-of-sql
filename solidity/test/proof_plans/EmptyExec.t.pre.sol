// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import {EmptyExec} from "../../src/proof_plans/EmptyExec.pre.sol";

contract EmptyExecTest is Test {
    function testEmptyExec() public pure {
        uint256[] memory evaluations = EmptyExec.__emptyExecEvaluate();
        assert(evaluations.length == 0);
    }
}
