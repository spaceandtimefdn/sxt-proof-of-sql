// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";

/// @title Math Utilities Library
/// @notice Provides functions to perform various math operations
library MathUtil {
    /// @notice Computes `max(1,ceil(log_2(value)))`
    /// @dev The smallest integer greater than or equal to the base 2 logarithm of a number.
    /// If the number is less than 2, the result is 1.
    /// @param __value The input value for which to compute the logarithm
    /// @return __exponent The computed logarithm value
    function __log2Up(uint256 __value) internal pure returns (uint256 __exponent) {
        assembly {
            function log2_up(value) -> exponent {
                if value { value := sub(value, 1) }
                exponent := 1
                if gt(value, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF) {
                    exponent := add(exponent, 128)
                    value := shr(128, value)
                }
                if gt(value, 0xFFFFFFFFFFFFFFFF) {
                    exponent := add(exponent, 64)
                    value := shr(64, value)
                }
                if gt(value, 0xFFFFFFFF) {
                    exponent := add(exponent, 32)
                    value := shr(32, value)
                }
                if gt(value, 0xFFFF) {
                    exponent := add(exponent, 16)
                    value := shr(16, value)
                }
                if gt(value, 0xFF) {
                    exponent := add(exponent, 8)
                    value := shr(8, value)
                }
                if gt(value, 0xF) {
                    exponent := add(exponent, 4)
                    value := shr(4, value)
                }
                if gt(value, 0x3) {
                    exponent := add(exponent, 2)
                    value := shr(2, value)
                }
                if gt(value, 0x1) {
                    exponent := add(exponent, 1)
                    value := shr(1, value)
                }
            }
            __exponent := log2_up(__value)
        }
    }

    /// @notice Computes `addmod(lhs, rhs, MODULUS)`
    /// @dev The sum of two uint256 values mod the order of bn254.
    /// @param __lhs The left hand side
    /// @param __rhs The right hand side
    /// @return __sum The sum of the two sides with the appropriate modulus
    function __addModBN254(uint256 __lhs, uint256 __rhs) internal pure returns (uint256 __sum) {
        assembly {
            function addmod_bn254(lhs, rhs) -> sum {
                sum := addmod(lhs, rhs, MODULUS)
            }
            __sum := addmod_bn254(__lhs, __rhs)
        }
    }

    /// @notice Computes `addmod(lhs, mulmod(rhs, MODULUS_MINUS_ONE, MODULUS), MODULUS)`
    /// @dev The difference of two uint256 values mod the order of bn254.
    /// @param __lhs The left hand side
    /// @param __rhs The right hand side
    /// @return __difference The difference of the two sides with the appropriate modulus
    function __subModBN254(uint256 __lhs, uint256 __rhs) internal pure returns (uint256 __difference) {
        assembly {
            function submod_bn254(lhs, rhs) -> difference {
                difference := addmod(lhs, mulmod(rhs, MODULUS_MINUS_ONE, MODULUS), MODULUS)
            }
            __difference := submod_bn254(__lhs, __rhs)
        }
    }

    /// @notice Computes `mulmod(lhs, rhs, MODULUS)`
    /// @dev The product of two uint256 values mod the order of bn254.
    /// @param __lhs The left hand side
    /// @param __rhs The right hand side
    /// @return __product The product of the two sides with the appropriate modulus
    function __mulModBN254(uint256 __lhs, uint256 __rhs) internal pure returns (uint256 __product) {
        assembly {
            function mulmod_bn254(lhs, rhs) -> product {
                product := mulmod(lhs, rhs, MODULUS)
            }
            __product := mulmod_bn254(__lhs, __rhs)
        }
    }
}
