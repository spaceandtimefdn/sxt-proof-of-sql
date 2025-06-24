// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";

/// @dev Invalid variant constant in the repo
uint32 constant INVALID_VARIANT = 0xFFFFFFFF;

contract ConstantsTest is Test {
    function testModulusMaskIsCorrect() public pure {
        assert(MODULUS > MODULUS_MASK);
        assert(MODULUS < (MODULUS_MASK << 1));

        // Check that the bits of MODULUS_MASK are a few 0s followed by all 1s.
        uint256 mask = MODULUS_MASK;
        while (mask & 1 == 1) {
            mask >>= 1;
        }
        assert(mask == 0);
    }

    function testModulusMaskAndModulusInvalidVaryMaskAgree() public pure {
        assert(MODULUS_MASK & MODULUS_INVALID_VARY_MASK == 0);
        assert(MODULUS_MASK | MODULUS_INVALID_VARY_MASK == (1 << 255) - 1);
    }

    function testModulusPlusAndMinusOneAreCorrect() public pure {
        assert(MODULUS_PLUS_ONE == MODULUS + 1);
        assert(MODULUS_MINUS_ONE == MODULUS - 1);
    }

    function testWordSizesAreCorrect() public pure {
        assert(WORD_SIZE == 32);
        assert(WORDX2_SIZE == 2 * WORD_SIZE);
        assert(WORDX3_SIZE == 3 * WORD_SIZE);
        assert(WORDX4_SIZE == 4 * WORD_SIZE);
        assert(WORDX5_SIZE == 5 * WORD_SIZE);
        assert(WORDX6_SIZE == 6 * WORD_SIZE);
        assert(WORDX8_SIZE == 8 * WORD_SIZE);
        assert(WORDX9_SIZE == 9 * WORD_SIZE);
        assert(WORDX10_SIZE == 10 * WORD_SIZE);
        assert(WORDX11_SIZE == 11 * WORD_SIZE);
        assert(WORDX12_SIZE == 12 * WORD_SIZE);
    }

    function testBooleanSizesAreCorrect() public pure {
        assert(BOOLEAN_SIZE * 8 == 8);
        assert(BOOLEAN_PADDING_BITS == 256 - 8);
        assert(BOOLEAN_SIZE_MINUS_ONE == BOOLEAN_SIZE - 1);
    }

    function testInt8SizesAreCorrect() public pure {
        assert(INT8_SIZE * 8 == 8);
        assert(INT8_PADDING_BITS == 256 - 8);
        assert(INT8_SIZE_MINUS_ONE == INT8_SIZE - 1);
    }

    function testUint8SizesAreCorrect() public pure {
        assert(UINT8_SIZE * 8 == 8);
    }

    function testInt16SizesAreCorrect() public pure {
        assert(INT16_SIZE * 8 == 16);
        assert(INT16_PADDING_BITS == 256 - 16);
        assert(INT16_SIZE_MINUS_ONE == INT16_SIZE - 1);
    }

    function testInt32SizesAreCorrect() public pure {
        assert(INT32_SIZE * 8 == 32);
        assert(INT32_PADDING_BITS == 256 - 32);
        assert(INT32_SIZE_MINUS_ONE == INT32_SIZE - 1);
    }

    function testUint32SizesAreCorrect() public pure {
        assert(UINT32_SIZE * 8 == 32);
        assert(UINT32_PADDING_BITS == 256 - 32);
    }

    function testUint64SizesAreCorrect() public pure {
        assert(UINT64_SIZE * 8 == 64);
        assert(UINT64_PADDING_BITS == 256 - 64);
    }

    function testInt64SizesAreCorrect() public pure {
        assert(INT64_SIZE * 8 == 64);
        assert(INT64_PADDING_BITS == 256 - 64);
        assert(INT64_SIZE_MINUS_ONE == INT64_SIZE - 1);
    }

    function testVerificationBuilderOffsetsAreValid() public pure {
        uint256[16] memory offsets = [
            BUILDER_CHALLENGES_OFFSET,
            BUILDER_FIRST_ROUND_MLES_OFFSET,
            BUILDER_FINAL_ROUND_MLES_OFFSET,
            BUILDER_CHI_EVALUATIONS_OFFSET,
            BUILDER_RHO_EVALUATIONS_OFFSET,
            BUILDER_CONSTRAINT_MULTIPLIERS_OFFSET,
            BUILDER_MAX_DEGREE_OFFSET,
            BUILDER_AGGREGATE_EVALUATION_OFFSET,
            BUILDER_ROW_MULTIPLIERS_EVALUATION_OFFSET,
            BUILDER_COLUMN_EVALUATIONS_OFFSET,
            BUILDER_TABLE_CHI_EVALUATIONS_OFFSET,
            BUILDER_PLACEHOLDER_PARAMETERS_OFFSET,
            BUILDER_FIRST_ROUND_COMMITMENTS_OFFSET,
            BUILDER_FINAL_ROUND_COMMITMENTS_OFFSET,
            BUILDER_SINGLETON_CHI_EVALUATION_OFFSET,
            BUILDER_FINAL_ROUND_BIT_DISTRIBUTIONS_OFFSET
        ];
        uint256 offsetsLength = offsets.length;
        assert(VERIFICATION_BUILDER_SIZE == offsetsLength * WORD_SIZE);
        for (uint256 i = 0; i < offsetsLength; ++i) {
            assert(offsets[i] % WORD_SIZE == 0); // Offsets must be word-aligned
            assert(offsets[i] < VERIFICATION_BUILDER_SIZE); // Offsets must be within the builder
            for (uint256 j = i + 1; j < offsetsLength; ++j) {
                assert(offsets[i] != offsets[j]); // Offsets must be unique
            }
        }
    }

    bytes32 private constant _PUBLIC_PARAMETERS_HASH =
        hex"c65198b7006b08652900d3dc4d282e2ad0bc71a04afffdbafa8fba7d956e478f";
    bytes32 private constant _SETUP_HASH = hex"e8840d8a41ce9d4e14e7ba0e1b023224751361577378291fcd3f0f05f0f7e875";

    function testInitialTranscriptStateIsHashPublicSetup() public pure {
        assert(
            _SETUP_HASH
                == keccak256(
                    // solhint-disable-next-line func-named-parameters
                    bytes.concat(
                        _PUBLIC_PARAMETERS_HASH,
                        bytes32(VK_TAU_HX_REAL),
                        bytes32(VK_TAU_HX_IMAG),
                        bytes32(VK_TAU_HY_REAL),
                        bytes32(VK_TAU_HY_IMAG)
                    )
                )
        );

        assert(INITIAL_TRANSCRIPT_STATE == uint256(keccak256(bytes.concat(_SETUP_HASH))));
    }
}
