// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {VerificationBuilder} from "../../src/builder/VerificationBuilder.pre.sol";
import {ProjectionExec} from "../../src/proof_plans/ProjectionExec.pre.sol";

contract ProjectionExecTest is Test {
    function testSimpleProjectionExec() public pure {
        bytes memory inputPlan = abi.encodePacked(
            FILTER_EXEC_VARIANT,
            uint64(0), // table_number
            abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(101)), // where clause
            abi.encodePacked( // select clause
                uint64(3),
                abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(102)),
                abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(103)),
                abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(104))
            )
        );
        VerificationBuilder.Builder memory builder;
        builder.maxDegree = 3;
        builder.finalRoundMLEs = new uint256[](5);
        builder.finalRoundMLEs[0] = 202;
        builder.finalRoundMLEs[1] = 203;
        builder.finalRoundMLEs[2] = 204;
        builder.finalRoundMLEs[3] = 301;
        builder.finalRoundMLEs[4] = 302;
        builder.constraintMultipliers = new uint256[](4);
        builder.constraintMultipliers[0] = 401;
        builder.constraintMultipliers[1] = 402;
        builder.constraintMultipliers[2] = 403;
        builder.constraintMultipliers[3] = 404;
        builder.challenges = new uint256[](2);
        builder.challenges[0] = 501;
        builder.challenges[1] = 502;
        builder.aggregateEvaluation = 0;
        builder.rowMultipliersEvaluation = 601;
        builder.chiEvaluations = new uint256[](2);
        builder.chiEvaluations[0] = 1;
        builder.chiEvaluations[1] = 701;
        builder.tableChiEvaluations = new uint256[](1);
        builder.tableChiEvaluations[0] = 801;

        bytes memory plan = abi.encodePacked(
            inputPlan, // inputPlan
            abi.encodePacked( // select clause
                uint64(3), // It doesn't really make sense to select literals from a subquery, but this is just what's easiest for testing right now
                abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(102)),
                abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(103)),
                abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(104))
            ),
            hex"abcdef"
        );

        uint256[] memory evals;
        uint256 outputChiEval;
        (plan, builder, evals,, outputChiEval) = ProjectionExec.__projectionExecEvaluate(plan, builder);

        assert(evals.length == 3);
        assert(evals[0] == 71502);
        assert(evals[1] == 72203);
        assert(evals[2] == 72904);
        assert(outputChiEval == 701);

        bytes memory expectedExprOut = hex"abcdef";
        assert(plan.length == expectedExprOut.length);
        uint256 exprOutLength = plan.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(plan[i] == expectedExprOut[i]);
        }
    }
}
