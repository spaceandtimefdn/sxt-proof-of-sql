// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import {VerificationBuilder} from "../../src/builder/VerificationBuilder.pre.sol";
import {SortMergeJoinExec} from "../../src/proof_plans/SortMergeJoinExec.pre.sol";

contract SortMergeJoinExecTest is Test {
    function testSortMergeJoinExec() public pure {
        VerificationBuilder.Builder memory builder;
        bytes memory plan = abi.encodePacked(uint64(0));
        uint256[] memory evaluations;
        uint256 chi_length;
        uint256 chi_eval;
        (plan, builder, evaluations, chi_length, chi_eval) = SortMergeJoinExec.__sortMergeJoinEvaluate(plan, builder);
    }
}
