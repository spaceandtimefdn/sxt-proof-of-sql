// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "./Constants.sol";
import "./Errors.sol";

/// @title Queue
/// @dev Library providing queue operations for memory-based queues.
library Queue {
    /// @notice Dequeues a value from the front of the queue
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// dequeue(queue_ptr) -> value
    /// ```
    /// ##### Parameters
    /// * `queue_ptr` - pointer to the array in memory. In Solidity memory layout,
    ///   this points to where the array length is stored, followed by the array elements
    /// ##### Return Values
    /// * `value` - the dequeued value from the front of the queue
    /// @dev Removes and returns the first element from the queue.
    /// Reverts with Errors.EmptyQueue if the queue is empty.
    /// @param __queue Single-element array containing the queue array
    /// @return __value The dequeued value
    function __dequeue(uint256[][1] memory __queue) internal pure returns (uint256 __value) {
        assembly {
            // IMPORT-YUL Errors.sol
            function err(code) {
                revert(0, 0)
            }
            function dequeue(queue_ptr) -> value {
                let queue := mload(queue_ptr)
                let length := mload(queue)
                if iszero(length) { err(ERR_EMPTY_QUEUE) }
                queue := add(queue, WORD_SIZE)
                value := mload(queue)
                mstore(queue, sub(length, 1))
                mstore(queue_ptr, queue)
            }
            __value := dequeue(__queue)
        }
    }

    /// @notice Dequeues two uint256 values from the front of the queue
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// function dequeue_uint512(queue_ptr) -> upper, lower
    /// ```
    /// ##### Parameters
    /// * `queue_ptr` - pointer to the array in memory. In Solidity memory layout,
    ///   this points to where the array length is stored, followed by the array elements
    /// ##### Return Values
    /// * `upper` - the first word from the dequeued value from the front of the queue
    /// * `lower` - the second word from the dequeued value from the front of the queue
    /// @dev Removes and returns the first element from the queue.
    /// Reverts with Errors.EmptyQueue if the queue is empty.
    /// @param __queuePtr Single-element array containing the queue array
    /// @return __upper The first word from the dequeued value from the front of the queue
    /// @return __lower The second word from the dequeued value from the front of the queue
    function __dequeueUint512(uint256[][1] memory __queuePtr)
        internal
        pure
        returns (uint256 __upper, uint256 __lower)
    {
        assembly {
            // IMPORT-YUL Errors.sol
            function err(code) {
                revert(0, 0)
            }
            function dequeue_uint512(queue_ptr) -> upper, lower {
                let queue := mload(queue_ptr)
                let length := mload(queue)
                if iszero(length) { err(ERR_EMPTY_QUEUE) }
                queue := add(queue, WORD_SIZE)
                upper := mload(queue)
                queue := add(queue, WORD_SIZE)
                lower := mload(queue)
                mstore(queue, sub(length, 1))
                mstore(queue_ptr, queue)
            }
            __upper, __lower := dequeue_uint512(__queuePtr)
        }
    }
}
