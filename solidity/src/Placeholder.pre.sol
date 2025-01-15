// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

library TestScript {
    function placeholderFunction() public pure {
        assembly {
            function return_one() -> result {
                result := 1
            }
        }
    }

    function testWeCanImportYul() public pure {
        uint256 a = 0;
        assembly {
            // IMPORT-YUL Placeholder.pre.sol
            function return_one() -> result {
                revert(0, 0)
            }
            a := return_one()
        }
        assert(a == 1);
    }
}
