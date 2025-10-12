// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {VerificationBuilder} from "../../src/builder/VerificationBuilder.pre.sol";
import {CastExpr} from "../../src/proof_exprs/CastExpr.pre.sol";
//import {ProofExpr} from "../../src/proof_exprs/ProofExpr.pre.sol";
import {F} from "../base/FieldUtil.sol";

contract CastExprTest is Test {
    function testSimpleCastExpr() public pure {
        VerificationBuilder.Builder memory builder;
        bytes memory expr = abi.encodePacked(
            DATA_TYPE_BIGINT_VARIANT, LITERAL_EXPR_VARIANT, DATA_TYPE_INT_VARIANT, int32(7), hex"abcdef"
        );
        bytes memory expectedExprOut = hex"abcdef";

        uint256 eval;
        (expr, builder, eval) = CastExpr.__castExprEvaluate(expr, builder, 10);

        assert(eval == 70); // 7 * 10
        assert(expr.length == expectedExprOut.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == expectedExprOut[i]);
        }
    }

    function testDecimalCastExpr() public pure {
        VerificationBuilder.Builder memory builder;
        bytes memory expr = abi.encodePacked(
            DATA_TYPE_DECIMAL75_VARIANT,
            uint8(20),
            int8(0),
            LITERAL_EXPR_VARIANT,
            DATA_TYPE_INT_VARIANT,
            int32(7),
            hex"abcdef"
        );
        bytes memory expectedExprOut = hex"abcdef";

        uint256 eval;
        (expr, builder, eval) = CastExpr.__castExprEvaluate(expr, builder, 10);

        assert(eval == 70); // 7 * 10
        assert(expr.length == expectedExprOut.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == expectedExprOut[i]);
        }
    }

    function testTimestampCastExpr() public pure {
        VerificationBuilder.Builder memory builder;
        bytes memory expr = abi.encodePacked(
            DATA_TYPE_BIGINT_VARIANT,
            LITERAL_EXPR_VARIANT,
            DATA_TYPE_TIMESTAMP_VARIANT,
            uint32(3),
            int32(0),
            int64(7),
            hex"abcdef"
        );
        bytes memory expectedExprOut = hex"abcdef";

        uint256 eval;
        (expr, builder, eval) = CastExpr.__castExprEvaluate(expr, builder, 10);

        assert(eval == 70); // 7 * 10
        assert(expr.length == expectedExprOut.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == expectedExprOut[i]);
        }
    }

    function testFuzzCastExpr(
        VerificationBuilder.Builder memory builder,
        uint256 chiEvaluation,
        int32 inputValue,
        bytes memory trailingExpr
    ) public pure {
        bytes memory expr = abi.encodePacked(
            DATA_TYPE_BIGINT_VARIANT, LITERAL_EXPR_VARIANT, DATA_TYPE_INT_VARIANT, inputValue, trailingExpr
        );

        uint256 eval;
        (expr, builder, eval) = CastExpr.__castExprEvaluate(expr, builder, chiEvaluation);

        assert(eval == (F.from(inputValue) * F.from(chiEvaluation)).into());
        assert(expr.length == trailingExpr.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == trailingExpr[i]);
        }
    }

    function testFuzzDecimalCastExpr(
        VerificationBuilder.Builder memory builder,
        uint256 chiEvaluation,
        int32 inputValue,
        uint8 decimalPrecision,
        bytes memory trailingExpr
    ) public pure {
        vm.assume(decimalPrecision > 9);

        bytes memory expr = abi.encodePacked(
            DATA_TYPE_DECIMAL75_VARIANT,
            decimalPrecision,
            int8(0),
            LITERAL_EXPR_VARIANT,
            DATA_TYPE_INT_VARIANT,
            inputValue,
            trailingExpr
        );

        uint256 eval;
        (expr, builder, eval) = CastExpr.__castExprEvaluate(expr, builder, chiEvaluation);

        assert(eval == (F.from(inputValue) * F.from(chiEvaluation)).into());
        assert(expr.length == trailingExpr.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == trailingExpr[i]);
        }
    }

    function testFuzzTimestampCastExpr(
        VerificationBuilder.Builder memory builder,
        uint256 chiEvaluation,
        uint32 timestampUnit,
        int32 timestampOffset,
        int64 timestampValue,
        bytes memory trailingExpr
    ) public pure {
        bytes memory expr = abi.encodePacked(
            DATA_TYPE_BIGINT_VARIANT,
            LITERAL_EXPR_VARIANT,
            DATA_TYPE_TIMESTAMP_VARIANT,
            timestampUnit,
            timestampOffset,
            timestampValue,
            trailingExpr
        );

        uint256 eval;
        (expr, builder, eval) = CastExpr.__castExprEvaluate(expr, builder, chiEvaluation);

        assert(eval == (F.from(timestampValue) * F.from(chiEvaluation)).into());
        assert(expr.length == trailingExpr.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == trailingExpr[i]);
        }
    }
}
