// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {VerificationBuilder} from "../../src/builder/VerificationBuilder.pre.sol";
import {GroupByExec} from "../../src/proof_plans/GroupByExec.pre.sol";
import {FF, F} from "../base/FieldUtil.sol";
import {ColumnExpr} from "../../src/proof_exprs/ColumnExpr.pre.sol";

contract GroupByExecTest is Test {
    function testSimpleGroupByExec() public pure {
        bytes memory plan = abi.encodePacked(
            uint64(0), // table_number
            abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(101)), // where clause
            uint64(2), // group_by_count
            abi.encodePacked(COLUMN_EXPR_VARIANT, uint64(0)), // group_by_expr[0] - column 0
            abi.encodePacked(COLUMN_EXPR_VARIANT, uint64(1)), // group_by_expr[1] - column 1
            uint64(2), // sum_count
            abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(104)), // sum_expr[0]
            abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, int64(105)), // sum_expr[1]
            uint64(64), // count_alias_length (unused in verification)
            hex"abcdef"
        );
        VerificationBuilder.Builder memory builder;
        builder.maxDegree = 3;
        builder.finalRoundMLEs = new uint256[](7);
        builder.finalRoundMLEs[0] = 202; // g_out_evals[0]
        builder.finalRoundMLEs[1] = 203; // g_out_evals[1]
        builder.finalRoundMLEs[2] = 204; // sum_out_evals[0]
        builder.finalRoundMLEs[3] = 205; // sum_out_evals[1]
        builder.finalRoundMLEs[4] = 206; // count_out_eval
        builder.finalRoundMLEs[5] = 207; // g_in_star_eval
        builder.finalRoundMLEs[6] = 208; // g_out_star_eval
        builder.constraintMultipliers = new uint256[](3);
        builder.constraintMultipliers[0] = 401;
        builder.constraintMultipliers[1] = 402;
        builder.constraintMultipliers[2] = 403;
        builder.challenges = new uint256[](2);
        builder.challenges[0] = 501; // alpha
        builder.challenges[1] = 502; // beta
        builder.aggregateEvaluation = 0;
        builder.rowMultipliersEvaluation = 601;
        builder.chiEvaluations = new uint256[](1);
        builder.chiEvaluations[0] = 701; // output_chi_eval
        builder.tableChiEvaluations = new uint256[](1);
        builder.tableChiEvaluations[0] = 801; // input_chi_eval
        // Define column evaluations for ColumnExpr
        builder.columnEvaluations = new uint256[](2);
        builder.columnEvaluations[0] = 102;
        builder.columnEvaluations[1] = 103;

        uint256[] memory evals;
        (plan, builder, evals) = GroupByExec.__groupByExecEvaluate(plan, builder);

        // g_in_fold = alpha * (g_in_evals[0] + g_in_evals[1] * beta)
        FF g_in_fold = FF.wrap(501) * (FF.wrap(102 * 801) + FF.wrap(103 * 801) * FF.wrap(502));

        // g_out_fold = alpha * (g_out_evals[0] + g_out_evals[1] * beta)
        FF g_out_fold = FF.wrap(501) * (FF.wrap(202) + FF.wrap(203) * FF.wrap(502));

        // sum_in_fold = input_chi_eval + beta * (sum_in_evals[0] + sum_in_evals[1] * beta)
        FF sum_in_fold = FF.wrap(801) + FF.wrap(502) * (FF.wrap(104 * 801) + FF.wrap(105 * 801) * FF.wrap(502));

        // sum_out_fold = count_out_eval + beta * (sum_out_evals[0] + sum_out_evals[1] * beta)
        FF sum_out_fold = FF.wrap(206) + FF.wrap(502) * (FF.wrap(204) + FF.wrap(205) * FF.wrap(502));

        // First constraint: g_in_star * sel_in * sum_in_fold - g_out_star * sum_out_fold = 0
        FF zeroSumConstraint = FF.wrap(207) * FF.wrap(101 * 801) * sum_in_fold - FF.wrap(208) * sum_out_fold;

        // Second constraint: g_in_star + g_in_star * g_in_fold - input_chi_eval = 0
        FF identityConstraint1 = FF.wrap(207) + FF.wrap(207) * g_in_fold - FF.wrap(801);

        // Third constraint: g_out_star + g_out_star * g_out_fold - output_chi_eval = 0
        FF identityConstraint2 = FF.wrap(208) + FF.wrap(208) * g_out_fold - FF.wrap(701);

        FF expectedAggregateEvaluation = zeroSumConstraint * FF.wrap(401) + identityConstraint1 * FF.wrap(402 * 601)
            + identityConstraint2 * FF.wrap(403 * 601);

        // Verify evaluations
        assert(evals.length == 5);
        assert(evals[0] == 202); // group_by result column 1
        assert(evals[1] == 203); // group_by result column 2
        assert(evals[2] == 204); // sum result column 1
        assert(evals[3] == 205); // sum result column 2
        assert(evals[4] == 206); // count result column

        // Verify aggregate evaluation
        assert(builder.aggregateEvaluation == expectedAggregateEvaluation.into());

        // Verify MLEs are consumed
        assert(builder.finalRoundMLEs.length == 0);

        // Verify constraint multipliers are consumed
        assert(builder.constraintMultipliers.length == 0);

        // Verify plan is advanced correctly
        bytes memory expectedExprOut = hex"abcdef";
        assert(plan.length == expectedExprOut.length);
        uint256 exprOutLength = plan.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(plan[i] == expectedExprOut[i]);
        }
    }

    function _computeGroupByExprZeroSumConstraint(
        VerificationBuilder.Builder memory builder,
        FF whereEvaluation,
        FF[] memory g_in_evals,
        FF[] memory sum_in_evals
    ) internal pure returns (FF zeroSumConstraint) {
        // g_in_fold = alpha * fold_vals(beta, g_in_evals)
        FF alpha = F.from(builder.challenges[0]);
        FF beta = F.from(builder.challenges[1]);
        FF g_in_fold = F.ZERO;
        for (uint256 i = 0; i < g_in_evals.length; i++) {
            g_in_fold = g_in_fold * beta + g_in_evals[i];
        }
        g_in_fold = alpha * g_in_fold;

        // g_out_fold = alpha * fold_vals(beta, g_out_evals)
        FF g_out_fold = F.ZERO;
        for (uint256 i = 0; i < g_in_evals.length; i++) {
            g_out_fold = g_out_fold * beta + F.from(builder.finalRoundMLEs[i]);
        }
        g_out_fold = alpha * g_out_fold;

        // sum_in_fold = input_chi_eval + beta * fold_vals(beta, sum_in_evals)
        FF input_chi_eval = F.from(builder.tableChiEvaluations[0]);
        FF sum_in_fold = F.ZERO;
        for (uint256 i = 0; i < sum_in_evals.length; i++) {
            sum_in_fold = sum_in_fold * beta + sum_in_evals[i];
        }
        sum_in_fold = input_chi_eval + beta * sum_in_fold;

        // sum_out_fold = count_out_eval + beta * fold_vals(beta, sum_out_evals)
        FF count_out_eval = F.from(builder.finalRoundMLEs[g_in_evals.length + sum_in_evals.length]);
        FF sum_out_fold = F.ZERO;
        for (uint256 i = 0; i < sum_in_evals.length; i++) {
            sum_out_fold = sum_out_fold * beta + F.from(builder.finalRoundMLEs[i + g_in_evals.length]);
        }
        sum_out_fold = count_out_eval + beta * sum_out_fold;

        FF g_in_star_eval = F.from(builder.finalRoundMLEs[g_in_evals.length + sum_in_evals.length + 1]);
        FF g_out_star_eval = F.from(builder.finalRoundMLEs[g_in_evals.length + sum_in_evals.length + 2]);

        // g_in_star * sel_in * sum_in_fold - g_out_star * sum_out_fold = 0
        zeroSumConstraint = g_in_star_eval * whereEvaluation * sum_in_fold - g_out_star_eval * sum_out_fold;
    }

    function testFuzzGroupByExec(
        VerificationBuilder.Builder memory builder,
        int64 where,
        int64[] memory g_in_vals,
        int64[] memory sum_in_vals,
        uint64 tableNumber
    ) public pure {
        vm.assume(g_in_vals.length > 0 && g_in_vals.length < 5);
        vm.assume(sum_in_vals.length > 0 && sum_in_vals.length < 5);

        // Construct plan
        bytes memory plan = abi.encodePacked(tableNumber);
        plan = abi.encodePacked(plan, abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, where));

        // Add group_by expressions as column expressions
        plan = abi.encodePacked(plan, uint64(g_in_vals.length));
        for (uint256 i = 0; i < g_in_vals.length; i++) {
            plan = abi.encodePacked(plan, abi.encodePacked(COLUMN_EXPR_VARIANT, uint64(i)));
        }

        // Add sum expressions
        plan = abi.encodePacked(plan, uint64(sum_in_vals.length));
        for (uint256 i = 0; i < sum_in_vals.length; i++) {
            plan =
                abi.encodePacked(plan, abi.encodePacked(LITERAL_EXPR_VARIANT, DATA_TYPE_BIGINT_VARIANT, sum_in_vals[i]));
        }

        // Add count alias (we don't care about the value for verification)
        plan = abi.encodePacked(plan, uint64(64));
        plan = abi.encodePacked(plan, hex"abcdef");

        // Setup builder
        vm.assume(builder.maxDegree > 2);
        vm.assume(builder.finalRoundMLEs.length > g_in_vals.length + sum_in_vals.length + 2);
        vm.assume(builder.constraintMultipliers.length > 2);
        vm.assume(builder.challenges.length > 1);
        vm.assume(builder.chiEvaluations.length > 0);
        vm.assume(builder.tableChiEvaluations.length > tableNumber);
        
        // Setup column evaluations for ColumnExpr
        builder.columnEvaluations = new uint256[](g_in_vals.length);
        for (uint256 i = 0; i < g_in_vals.length; i++) {
            builder.columnEvaluations[i] = uint256(g_in_vals[i]);
        }

        // Prepare expected evaluations
        uint256[] memory expectedResultEvaluations = new uint256[](g_in_vals.length + sum_in_vals.length + 1);
        for (uint256 i = 0; i < g_in_vals.length + sum_in_vals.length + 1; i++) {
            expectedResultEvaluations[i] = builder.finalRoundMLEs[i];
        }

        // Prepare input evaluations
        FF[] memory g_in_evals = new FF[](g_in_vals.length);
        for (uint256 i = 0; i < g_in_vals.length; i++) {
            g_in_evals[i] = F.from(g_in_vals[i]) * F.from(builder.tableChiEvaluations[tableNumber]);
        }

        FF[] memory sum_in_evals = new FF[](sum_in_vals.length);
        for (uint256 i = 0; i < sum_in_vals.length; i++) {
            sum_in_evals[i] = F.from(sum_in_vals[i]) * F.from(builder.tableChiEvaluations[tableNumber]);
        }

        //FF whereEvaluation = F.from(where) * F.from(builder.tableChiEvaluations[tableNumber]);

        // Execute group by
        uint256[] memory evals;
        (plan, builder, evals) = GroupByExec.__groupByExecEvaluate(plan, builder);

        // Verify evaluations
        uint256 evalsLength = evals.length;
        assert(evalsLength == expectedResultEvaluations.length);
        for (uint256 i = 0; i < evalsLength; i++) {
            assert(evals[i] == expectedResultEvaluations[i]);
        }

        // Verify plan is advanced correctly
        bytes memory expectedExprOut = hex"abcdef";
        assert(plan.length == expectedExprOut.length);
        uint256 exprOutLength = plan.length;
        for (uint256 i = 0; i < exprOutLength; i++) {
            assert(plan[i] == expectedExprOut[i]);
        }
    }
}
