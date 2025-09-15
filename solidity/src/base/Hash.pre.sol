// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "./Constants.sol";

/// @title Hash
/// @dev Library for hashing
library Hash {
    /// @notice Hashes a string using keccak256
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// function hash_string(ptr, free_ptr) -> ptr_out, hashed_string, free_ptr_out
    /// ```
    /// ##### Parameters
    /// * `ptr` - the calldata pointer to the string
    /// * `free_ptr` - the location to load the hashed string
    /// @dev Returns the entry value and updated result pointer
    /// @param __ptr The pointer to the string
    /// @return __ptrOut The updated pointer
    /// @return __hashedString The hashed string
    function __hashString(bytes calldata __ptr)
        external
        pure
        returns (bytes calldata __ptrOut, bytes32 __hashedString)
    {
        assembly {
            function hash_string(ptr, free_ptr) -> ptr_out, free_ptr_out {
                let name_len := shr(UINT64_PADDING_BITS, calldataload(ptr))
                ptr := add(ptr, UINT64_SIZE)

                // TODO: This line should probably be using the FREE_PTR directly, instead of having it passed in the function.
                // This is a little dangerous as it is.
                calldatacopy(free_ptr, ptr, name_len)
                mstore(free_ptr, keccak256(free_ptr, name_len))
                ptr_out := add(ptr, name_len)
                free_ptr_out := add(free_ptr, WORD_SIZE)
            }

            let free_ptr := mload(FREE_PTR)
            let __ptrOutOffset
            __ptrOutOffset, free_ptr := hash_string(__ptr.offset, free_ptr)
            __hashedString := mload(sub(free_ptr, WORD_SIZE))
            __ptrOut.offset := __ptrOutOffset
            // slither-disable-next-line write-after-write
            __ptrOut.length := sub(__ptr.length, sub(__ptrOutOffset, __ptr.offset))
        }
    }
}
