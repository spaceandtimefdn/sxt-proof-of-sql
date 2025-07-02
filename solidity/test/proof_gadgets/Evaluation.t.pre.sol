// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {VerificationBuilder} from "../../src/builder/VerificationBuilder.pre.sol";
import {EvaluationUtil} from "../../src/proof_gadgets/EvaluationUtil.pre.sol";

contract EvaluationUtilTest is Test {
    function testEvaluateProofExprs() public pure {
        VerificationBuilder.Builder memory builder;

        // Create plan with two literal expressions: 2 and 3
        bytes memory plan = abi.encodePacked(
            LITERAL_EXPR_VARIANT,
            DATA_TYPE_BIGINT_VARIANT,
            int64(2),
            LITERAL_EXPR_VARIANT,
            DATA_TYPE_BIGINT_VARIANT,
            int64(3),
            hex"abcdef"
        );
        bytes memory expectedPlanOut = hex"abcdef";
        uint256 inputChiEval = 5;
        uint256 columnCount = 2;
        uint256[] memory evaluations;

        (plan, builder, evaluations) = EvaluationUtil.__evaluateProofExprs(plan, builder, inputChiEval, columnCount);

        assert(evaluations.length == 2);
        assert(evaluations[0] == 10);
        assert(evaluations[1] == 15);
        assert(plan.length == expectedPlanOut.length);
        uint256 planOutLength = plan.length;
        for (uint256 i = 0; i < planOutLength; ++i) {
            assert(plan[i] == expectedPlanOut[i]);
        }
    }
}
