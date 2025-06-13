// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {Errors} from "../../src/base/Errors.sol";
import {VerificationBuilder} from "../../src/builder/VerificationBuilder.pre.sol";
import {ProofPlan} from "../../src/proof_plans/ProofPlan.pre.sol";
import {FF, F} from "../base/FieldUtil.sol";

contract ProofPlanTest is Test {
    function testFilterExecVariant() public pure {
        bytes memory plan = abi.encodePacked(
            FILTER_EXEC_VARIANT,
            uint64(0), // table_number
            abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(101)), // where clause
            abi.encodePacked( // select clause
                uint64(3),
                abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(102)),
                abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(103)),
                abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(104))
            ),
            hex"abcdef"
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
        builder.chiEvaluations = new uint256[](1);
        builder.chiEvaluations[0] = 701;
        builder.tableChiEvaluations = new uint256[](1);
        builder.tableChiEvaluations[0] = 801;

        uint256[] memory evals;
        uint256 outputChiEval;
        (plan, builder, evals, outputChiEval) = ProofPlan.__proofPlanEvaluate(plan, builder);

        FF cFold = FF.wrap(502 * 502) * FF.wrap(102 * 801) + FF.wrap(502) * FF.wrap(103 * 801) + FF.wrap(104 * 801);
        FF dFold = FF.wrap(502 * 502) * FF.wrap(202) + FF.wrap(502) * FF.wrap(203) + FF.wrap(204);

        FF zeroSumConstraint0 = FF.wrap(301) * FF.wrap(101 * 801) - FF.wrap(302);
        FF identityConstraint1 = (F.ONE + FF.wrap(501) * cFold) * FF.wrap(301) - FF.wrap(801);
        FF identityConstraint2 = (F.ONE + FF.wrap(501) * dFold) * FF.wrap(302) - FF.wrap(701);
        FF identityConstraint3 = FF.wrap(501) * dFold * (FF.wrap(701) - F.ONE);

        FF expectedAggregateEvaluation = zeroSumConstraint0 * FF.wrap(401) + identityConstraint1 * FF.wrap(402 * 601)
            + identityConstraint2 * FF.wrap(403 * 601) + identityConstraint3 * FF.wrap(404 * 601);

        assert(evals.length == 3);
        assert(evals[0] == 202);
        assert(evals[1] == 203);
        assert(evals[2] == 204);
        assert(builder.aggregateEvaluation == expectedAggregateEvaluation.into());
        assert(builder.finalRoundMLEs.length == 0);
        assert(builder.constraintMultipliers.length == 0);

        bytes memory expectedExprOut = hex"abcdef";
        assert(plan.length == expectedExprOut.length);
        uint256 exprOutLength = plan.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(plan[i] == expectedExprOut[i]);
        }
    }

    /// forge-config: default.allow_internal_expect_revert = true
    // function testGroupByExecVariant() public pure {
    //     bytes memory plan = abi.encodePacked(
    //         GROUP_BY_EXEC_VARIANT,
    //         uint64(0), // table_number
    //         abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(101)), // where clause
    //         uint64(2), // group_by_count
    //         abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(102)), // group_by_expr[0]
    //         abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(103)), // group_by_expr[1]
    //         uint64(2), // sum_count
    //         abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(104)), // sum_expr[0]
    //         abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(105)), // sum_expr[1]
    //         uint64(64), // count_alias_length (unused in verification)
    //         hex"abcdef"
    //     );
    //     VerificationBuilder.Builder memory builder;
    //     builder.maxDegree = 3;
    //     builder.finalRoundMLEs = new uint256[](7);
    //     builder.finalRoundMLEs[0] = 202; // g_out_evals[0]
    //     builder.finalRoundMLEs[1] = 203; // g_out_evals[1]
    //     builder.finalRoundMLEs[2] = 204; // sum_out_evals[0]
    //     builder.finalRoundMLEs[3] = 205; // sum_out_evals[1]
    //     builder.finalRoundMLEs[4] = 206; // count_out_eval
    //     builder.finalRoundMLEs[5] = 207; // g_in_star_eval
    //     builder.finalRoundMLEs[6] = 208; // g_out_star_eval
    //     builder.constraintMultipliers = new uint256[](3);
    //     builder.constraintMultipliers[0] = 401;
    //     builder.constraintMultipliers[1] = 402;
    //     builder.constraintMultipliers[2] = 403;
    //     builder.challenges = new uint256[](2);
    //     builder.challenges[0] = 501; // alpha
    //     builder.challenges[1] = 502; // beta
    //     builder.aggregateEvaluation = 0;
    //     builder.rowMultipliersEvaluation = 601;
    //     builder.chiEvaluations = new uint256[](1);
    //     builder.chiEvaluations[0] = 701; // output_chi_eval
    //     builder.tableChiEvaluations = new uint256[](1);
    //     builder.tableChiEvaluations[0] = 801; // input_chi_eval

    //     uint256[] memory evals;
    //     (plan, builder, evals) = ProofPlan.__proofPlanEvaluate(plan, builder);

    //     // g_in_fold = alpha * (g_in_evals[0] + g_in_evals[1] * beta)
    //     FF g_in_fold = FF.wrap(501) * (FF.wrap(102 * 801) + FF.wrap(103 * 801) * FF.wrap(502));

    //     // g_out_fold = alpha * (g_out_evals[0] + g_out_evals[1] * beta)
    //     FF g_out_fold = FF.wrap(501) * (FF.wrap(202) + FF.wrap(203) * FF.wrap(502));

    //     // sum_in_fold = input_chi_eval + beta * (sum_in_evals[0] + sum_in_evals[1] * beta)
    //     FF sum_in_fold = FF.wrap(801) + FF.wrap(502) * (FF.wrap(104 * 801) + FF.wrap(105 * 801) * FF.wrap(502));

    //     // sum_out_fold = count_out_eval + beta * (sum_out_evals[0] + sum_out_evals[1] * beta)
    //     FF sum_out_fold = FF.wrap(206) + FF.wrap(502) * (FF.wrap(204) + FF.wrap(205) * FF.wrap(502));

    //     // First constraint: g_in_star * sel_in * sum_in_fold - g_out_star * sum_out_fold = 0
    //     FF zeroSumConstraint = FF.wrap(207) * FF.wrap(101 * 801) * sum_in_fold - FF.wrap(208) * sum_out_fold;

    //     // Second constraint: g_in_star + g_in_star * g_in_fold - input_chi_eval = 0
    //     FF identityConstraint1 = FF.wrap(207) + FF.wrap(207) * g_in_fold - FF.wrap(801);

    //     // Third constraint: g_out_star + g_out_star * g_out_fold - output_chi_eval = 0
    //     FF identityConstraint2 = FF.wrap(208) + FF.wrap(208) * g_out_fold - FF.wrap(701);

    //     FF expectedAggregateEvaluation = zeroSumConstraint * FF.wrap(401) + identityConstraint1 * FF.wrap(402 * 601)
    //         + identityConstraint2 * FF.wrap(403 * 601);

    //     // Verify evaluations
    //     assert(evals.length == 5);
    //     assert(evals[0] == 202); // group_by result column 1
    //     assert(evals[1] == 203); // group_by result column 2
    //     assert(evals[2] == 204); // sum result column 1
    //     assert(evals[3] == 205); // sum result column 2
    //     assert(evals[4] == 206); // count result column

    //     // Verify aggregate evaluation
    //     assert(builder.aggregateEvaluation == expectedAggregateEvaluation.into());

    //     // Verify MLEs are consumed
    //     assert(builder.finalRoundMLEs.length == 0);

    //     // Verify constraint multipliers are consumed
    //     assert(builder.constraintMultipliers.length == 0);

    //     // Verify plan is advanced correctly
    //     bytes memory expectedExprOut = hex"abcdef";
    //     assert(plan.length == expectedExprOut.length);
    //     uint256 exprOutLength = plan.length;
    //     for (uint256 i = 0; i < exprOutLength; ++i) {
    //         assert(plan[i] == expectedExprOut[i]);
    //     }
    // }

    function testUnsupportedVariant() public {
        VerificationBuilder.Builder memory builder;
        bytes memory plan = abi.encodePacked(uint32(100), hex"abcdef");
        vm.expectRevert(Errors.UnsupportedProofPlanVariant.selector);
        ProofPlan.__proofPlanEvaluate(plan, builder);
    }

    function testEmptyExecVariant() public pure {
        VerificationBuilder.Builder memory builder;
        bytes memory plan = abi.encodePacked(EMPTY_EXEC_VARIANT, hex"abcdef");
        uint256[] memory evals;
        uint256 chiEval;
        (plan, builder, evals, chiEval) = ProofPlan.__proofPlanEvaluate(plan, builder);
        assert(evals.length == 0);
        bytes memory expectedExprOut = hex"abcdef";
        uint256 exprOutLength = plan.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(plan[i] == expectedExprOut[i]);
        }
    }

    function testTableExecVariant() public pure {
        bytes memory plan = abi.encodePacked(
            TABLE_EXEC_VARIANT,
            uint64(0), // table_ref
            uint64(3), // column_count
            uint64(0), // column1_index
            uint64(1), // column2_index
            uint64(2), // column3_index
            hex"abcdef"
        );

        VerificationBuilder.Builder memory builder;
        builder.tableChiEvaluations = new uint256[](1);
        builder.tableChiEvaluations[0] = 801;
        builder.columnEvaluations = new uint256[](3);
        builder.columnEvaluations[0] = 101;
        builder.columnEvaluations[1] = 102;
        builder.columnEvaluations[2] = 103;

        uint256[] memory evals;
        uint256 chiEval;
        (plan, builder, evals, chiEval) = ProofPlan.__proofPlanEvaluate(plan, builder);

        assert(evals.length == 3);
        assert(evals[0] == 101);
        assert(evals[1] == 102);
        assert(evals[2] == 103);

        bytes memory expectedExprOut = hex"abcdef";
        assert(plan.length == expectedExprOut.length);
        uint256 exprOutLength = plan.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(plan[i] == expectedExprOut[i]);
        }
    }

    function testVariantsMatchEnum() public pure {
        assert(uint32(ProofPlan.PlanVariant.Filter) == FILTER_EXEC_VARIANT);
        assert(uint32(ProofPlan.PlanVariant.Empty) == EMPTY_EXEC_VARIANT);
        assert(uint32(ProofPlan.PlanVariant.Table) == TABLE_EXEC_VARIANT);
        assert(uint32(ProofPlan.PlanVariant.GroupBy) == GROUP_BY_EXEC_VARIANT);
    }
}
