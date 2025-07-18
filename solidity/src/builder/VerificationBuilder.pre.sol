// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";

/// @title VerificationBuilder
/// @dev Library providing memory management and state tracking for the verification process.
/// Maintains queues of challenges and various MLE evaluations.
library VerificationBuilder {
    struct Builder {
        uint256[] challenges;
        uint256[] firstRoundMLEs;
        uint256[] finalRoundMLEs;
        uint256[] chiEvaluations;
        uint256[] rhoEvaluations;
        uint256[] constraintMultipliers;
        uint256 maxDegree;
        uint256 aggregateEvaluation;
        uint256 rowMultipliersEvaluation;
        uint256[] columnEvaluations;
        uint256[] tableChiEvaluations;
        uint256[] placeholderParameters;
        uint256 firstRoundCommitmentsPtr;
        uint256 finalRoundCommitmentsPtr;
        uint256 singletonChiEvaluation;
        uint256[] bitDistributions;
    }

    /// @notice Allocates and reserves a block of memory for a verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_new() -> builder_ptr
    /// ```
    /// ##### Return Values
    /// * `builder_ptr` - memory pointer to the newly allocated builder region
    /// @dev Allocates memory for the builder structure and updates the free memory pointer
    /// @return __builder The builder struct
    function __builderNew() internal pure returns (Builder memory __builder) {
        assembly {
            function builder_new() -> builder_ptr {
                builder_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(builder_ptr, VERIFICATION_BUILDER_SIZE))
            }
            __builder := builder_new()
        }
    }

    /// @notice Sets the challenges in the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_set_challenges(builder_ptr, challenges_ptr)
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// * `challenges_ptr` - pointer to the array in memory. In Solidity memory layout,
    ///   this points to where the array length is stored, followed by the array elements
    /// @dev Stores the challenges array pointer in the builder structure.
    /// WARNING: The challenges array will be mutated during verification and should not
    /// be used after passing to this function.
    /// @param __builder The builder struct
    /// @param __challenges The challenges array
    function __setChallenges(Builder memory __builder, uint256[] memory __challenges) internal pure {
        assembly {
            function builder_set_challenges(builder_ptr, challenges_ptr) {
                mstore(add(builder_ptr, BUILDER_CHALLENGES_OFFSET), challenges_ptr)
            }
            builder_set_challenges(__builder, __challenges)
        }
    }

    /// @notice Consumes a challenge from the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_consume_challenge(builder_ptr) -> challenge
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - pointer to the verification builder
    /// ##### Return Values
    /// * `challenge` - the consumed challenge value
    /// @dev Dequeues and returns a challenge. Reverts with Errors.EmptyQueue if no challenges remain
    /// @param __builder The pointer to the verification builder
    /// @return __challenge The consumed challenge
    function __consumeChallenge(Builder memory __builder) internal pure returns (uint256 __challenge) {
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Queue.pre.sol
            function dequeue(queue_ptr) -> value {
                revert(0, 0)
            }
            function builder_consume_challenge(builder_ptr) -> challenge {
                challenge := dequeue(add(builder_ptr, BUILDER_CHALLENGES_OFFSET))
            }
            __challenge := builder_consume_challenge(__builder)
        }
    }

    /// @notice Sets the first round MLE evaluations in the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_set_first_round_mles(builder_ptr, values_ptr)
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// * `values_ptr` - pointer to the array in memory. In Solidity memory layout,
    ///   this points to where the array length is stored, followed by the array elements
    /// @dev Stores the first round MLE array pointer in the builder structure.
    /// WARNING: The values array will be mutated during verification and should not
    /// be used after passing to this function.
    /// @param __builder The builder struct
    /// @param __values The first round MLE values array
    function __setFirstRoundMLEs(Builder memory __builder, uint256[] memory __values) internal pure {
        assembly {
            function builder_set_first_round_mles(builder_ptr, values_ptr) {
                mstore(add(builder_ptr, BUILDER_FIRST_ROUND_MLES_OFFSET), values_ptr)
            }
            builder_set_first_round_mles(__builder, __values)
        }
    }

    /// @notice Gets the first round MLE values from the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_get_first_round_mles(builder_ptr) -> values_ptr
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// ##### Return Values
    /// * `values_ptr` - pointer to the array in memory
    /// @param __builder The builder struct
    /// @return __values The first round MLE values array
    function __getFirstRoundMLEs(Builder memory __builder) internal pure returns (uint256[] memory __values) {
        assembly {
            function builder_get_first_round_mles(builder_ptr) -> values_ptr {
                values_ptr := mload(add(builder_ptr, BUILDER_FIRST_ROUND_MLES_OFFSET))
            }
            __values := builder_get_first_round_mles(__builder)
        }
    }

    /// @notice Consumes a first round MLE evaluation from the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_consume_first_round_mle(builder_ptr) -> value
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// ##### Return Values
    /// * `value` - the consumed first round MLE value
    /// @dev Dequeues and returns a first round MLE value. Reverts with Errors.EmptyQueue if no values remain
    /// @param __builder The builder struct
    /// @return __value The consumed first round MLE value
    function __consumeFirstRoundMLE(Builder memory __builder) internal pure returns (uint256 __value) {
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Queue.pre.sol
            function dequeue(queue_ptr) -> value {
                revert(0, 0)
            }
            function builder_consume_first_round_mle(builder_ptr) -> value {
                value := dequeue(add(builder_ptr, BUILDER_FIRST_ROUND_MLES_OFFSET))
            }
            __value := builder_consume_first_round_mle(__builder)
        }
    }

    /// @notice Sets the final round MLE evaluations in the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_set_final_round_mles(builder_ptr, values_ptr)
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// * `values_ptr` - pointer to the array in memory. In Solidity memory layout,
    ///   this points to where the array length is stored, followed by the array elements
    /// @dev Stores the final round MLE array pointer in the builder structure.
    /// WARNING: The values array will be mutated during verification and should not
    /// be used after passing to this function.
    /// @param __builder The builder struct
    /// @param __values The final round MLE values array
    function __setFinalRoundMLEs(Builder memory __builder, uint256[] memory __values) internal pure {
        assembly {
            function builder_set_final_round_mles(builder_ptr, values_ptr) {
                mstore(add(builder_ptr, BUILDER_FINAL_ROUND_MLES_OFFSET), values_ptr)
            }
            builder_set_final_round_mles(__builder, __values)
        }
    }

    /// @notice Gets the final round MLE values from the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_get_final_round_mles(builder_ptr) -> values_ptr
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// ##### Return Values
    /// * `values_ptr` - pointer to the array in memory
    /// @param __builder The builder struct
    /// @return __values The final round MLE values array
    function __getFinalRoundMLEs(Builder memory __builder) internal pure returns (uint256[] memory __values) {
        assembly {
            function builder_get_final_round_mles(builder_ptr) -> values_ptr {
                values_ptr := mload(add(builder_ptr, BUILDER_FINAL_ROUND_MLES_OFFSET))
            }
            __values := builder_get_final_round_mles(__builder)
        }
    }

    /// @notice Consumes a final round MLE evaluation from the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_consume_final_round_mle(builder_ptr) -> value
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// ##### Return Values
    /// * `value` - the consumed final round MLE value
    /// @dev Dequeues and returns a final round MLE value. Reverts with Errors.EmptyQueue if no values remain
    /// @param __builder The builder struct
    /// @return __value The consumed final round MLE value
    function __consumeFinalRoundMLE(Builder memory __builder) internal pure returns (uint256 __value) {
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Queue.pre.sol
            function dequeue(queue_ptr) -> value {
                revert(0, 0)
            }
            function builder_consume_final_round_mle(builder_ptr) -> value {
                value := dequeue(add(builder_ptr, BUILDER_FINAL_ROUND_MLES_OFFSET))
            }
            __value := builder_consume_final_round_mle(__builder)
        }
    }

    /// @notice Sets the chi column evaluations in the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_set_chi_evaluations(builder_ptr, values_ptr)
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// * `values_ptr` - pointer to the array in memory. In Solidity memory layout,
    ///   this points to where the array length is stored, followed by the array elements
    /// @dev Stores the chi column evaluations array pointer in the builder structure.
    /// WARNING: The values array will be mutated during verification and should not
    /// be used after passing to this function.
    /// @param __builder The builder struct
    /// @param __values The chi column evaluation values array
    function __setChiEvaluations(Builder memory __builder, uint256[] memory __values) internal pure {
        assembly {
            function builder_set_chi_evaluations(builder_ptr, values_ptr) {
                mstore(add(builder_ptr, BUILDER_CHI_EVALUATIONS_OFFSET), values_ptr)
            }
            builder_set_chi_evaluations(__builder, __values)
        }
    }

    /// @notice Consumes a chi column evaluation from the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_consume_chi_evaluation(builder_ptr) -> value
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// ##### Return Values
    /// * `value` - the consumed chi evaluation value
    /// @dev Dequeues and returns a chi column evaluation value. Reverts with Errors.EmptyQueue if no values remain
    /// @param __builder The builder struct
    /// @return __value The consumed chi column evaluation value
    function __consumeChiEvaluation(Builder memory __builder) internal pure returns (uint256 __value) {
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Queue.pre.sol
            function dequeue_uint512(queue_ptr) -> upper, lower {
                revert(0, 0)
            }
            function builder_consume_chi_evaluation(builder_ptr) -> value {
                let length
                length, value := dequeue_uint512(add(builder_ptr, BUILDER_CHI_EVALUATIONS_OFFSET))
            }
            __value := builder_consume_chi_evaluation(__builder)
        }
    }

    /// @notice Consumes a chi column length and evaluation from the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// function builder_consume_chi_evaluation_with_length(builder_ptr) -> length, chi_eval
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// ##### Return Values
    /// * `length` - the length of the column of ones
    /// * `chi_eval` - the consumed chi evaluation value
    /// @dev Dequeues and returns a chi column length and evaluation value. Reverts with Errors.EmptyQueue if no values remain
    /// @param __builder The builder struct
    /// @return __length The length of the column of ones
    /// @return __chiEval The consumed chi column evaluation value
    function __consumeChiEvaluationWithLength(Builder memory __builder)
        internal
        pure
        returns (uint256 __length, uint256 __chiEval)
    {
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Queue.pre.sol
            function dequeue_uint512(queue_ptr) -> upper, lower {
                revert(0, 0)
            }
            function builder_consume_chi_evaluation_with_length(builder_ptr) -> length, chi_eval {
                length, chi_eval := dequeue_uint512(add(builder_ptr, BUILDER_CHI_EVALUATIONS_OFFSET))
            }
            __length, __chiEval := builder_consume_chi_evaluation_with_length(__builder)
        }
    }

    /// @notice Sets the rho column evaluations in the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_set_rho_evaluations(builder_ptr, values_ptr)
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// * `values_ptr` - pointer to the array in memory. In Solidity memory layout,
    ///   this points to where the array length is stored, followed by the array elements
    /// @dev Stores the rho column evaluations array pointer in the builder structure.
    /// WARNING: The values array will be mutated during verification and should not
    /// be used after passing to this function.
    /// @param __builder The builder struct
    /// @param __values The rho column evaluation values array
    function __setRhoEvaluations(Builder memory __builder, uint256[] memory __values) internal pure {
        assembly {
            function builder_set_rho_evaluations(builder_ptr, values_ptr) {
                mstore(add(builder_ptr, BUILDER_RHO_EVALUATIONS_OFFSET), values_ptr)
            }
            builder_set_rho_evaluations(__builder, __values)
        }
    }

    /// @notice Consumes a rho column evaluation from the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_consume_rho_evaluation(builder_ptr) -> value
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// ##### Return Values
    /// * `value` - the consumed rho evaluation value
    /// @dev Dequeues and returns a rho column evaluation value. Reverts with Errors.EmptyQueue if no values remain
    /// @param __builder The builder struct
    /// @return __value The consumed rho column evaluation value
    function __consumeRhoEvaluation(Builder memory __builder) internal pure returns (uint256 __value) {
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Queue.pre.sol
            function dequeue(queue_ptr) -> value {
                revert(0, 0)
            }
            function builder_consume_rho_evaluation(builder_ptr) -> value {
                value := dequeue(add(builder_ptr, BUILDER_RHO_EVALUATIONS_OFFSET))
            }
            __value := builder_consume_rho_evaluation(__builder)
        }
    }

    /// @notice Sets the constraint multipliers in the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_set_constraint_multipliers(builder_ptr, values_ptr)
    /// ```
    /// @param __builder The builder struct
    /// @param __values The constraint multipliers array
    function __setConstraintMultipliers(Builder memory __builder, uint256[] memory __values) internal pure {
        assembly {
            function builder_set_constraint_multipliers(builder_ptr, values_ptr) {
                mstore(add(builder_ptr, BUILDER_CONSTRAINT_MULTIPLIERS_OFFSET), values_ptr)
            }
            builder_set_constraint_multipliers(__builder, __values)
        }
    }

    /// @notice Sets the max degree in the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_set_max_degree(builder_ptr, value)
    /// ```
    /// @param __builder The builder struct
    /// @param __value The max degree value
    function __setMaxDegree(Builder memory __builder, uint256 __value) internal pure {
        assembly {
            function builder_set_max_degree(builder_ptr, value) {
                mstore(add(builder_ptr, BUILDER_MAX_DEGREE_OFFSET), value)
            }
            builder_set_max_degree(__builder, __value)
        }
    }

    /// @notice Sets the aggregate evaluation in the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_set_aggregate_evaluation(builder_ptr, value)
    /// ```
    /// @param __builder The builder struct
    /// @param __value The aggregate evaluation value
    function __setAggregateEvaluation(Builder memory __builder, uint256 __value) internal pure {
        assembly {
            function builder_set_aggregate_evaluation(builder_ptr, value) {
                mstore(add(builder_ptr, BUILDER_AGGREGATE_EVALUATION_OFFSET), value)
            }
            builder_set_aggregate_evaluation(__builder, __value)
        }
    }

    /// @notice Gets the aggregate evaluation from the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_get_aggregate_evaluation(builder_ptr) -> value
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// ##### Return Values
    /// * `value` - the aggregate evaluation value
    /// @param __builder The builder struct
    /// @return __value The aggregate evaluation value
    function __getAggregateEvaluation(Builder memory __builder) internal pure returns (uint256 __value) {
        assembly {
            function builder_get_aggregate_evaluation(builder_ptr) -> value {
                value := mload(add(builder_ptr, BUILDER_AGGREGATE_EVALUATION_OFFSET))
            }
            __value := builder_get_aggregate_evaluation(__builder)
        }
    }

    /// @notice Sets the row multipliers evaluation in the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_set_row_multipliers_evaluation(builder_ptr, value)
    /// ```
    /// @param __builder The builder struct
    /// @param __value The row multipliers evaluation value
    function __setRowMultipliersEvaluation(Builder memory __builder, uint256 __value) internal pure {
        assembly {
            function builder_set_row_multipliers_evaluation(builder_ptr, value) {
                mstore(add(builder_ptr, BUILDER_ROW_MULTIPLIERS_EVALUATION_OFFSET), value)
            }
            builder_set_row_multipliers_evaluation(__builder, __value)
        }
    }

    function __produceZerosumConstraint(Builder memory __builder, uint256 __evaluation, uint256 __degree)
        internal
        pure
    {
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Queue.pre.sol
            function dequeue(queue_ptr) -> value {
                revert(0, 0)
            }
            function builder_produce_zerosum_constraint(builder_ptr, evaluation, degree) {
                if gt(degree, mload(add(builder_ptr, BUILDER_MAX_DEGREE_OFFSET))) {
                    err(ERR_CONSTRAINT_DEGREE_TOO_HIGH)
                }
                // builder.aggregateEvaluation += evaluation * dequeue(builder.constraintMultipliers)
                mstore(
                    add(builder_ptr, BUILDER_AGGREGATE_EVALUATION_OFFSET),
                    addmod(
                        mload(add(builder_ptr, BUILDER_AGGREGATE_EVALUATION_OFFSET)),
                        mulmod(evaluation, dequeue(add(builder_ptr, BUILDER_CONSTRAINT_MULTIPLIERS_OFFSET)), MODULUS),
                        MODULUS
                    )
                )
            }
            builder_produce_zerosum_constraint(__builder, __evaluation, __degree)
        }
    }

    function __produceIdentityConstraint(Builder memory __builder, uint256 __evaluation, uint256 __degree)
        internal
        pure
    {
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Queue.pre.sol
            function dequeue(queue_ptr) -> value {
                revert(0, 0)
            }
            function builder_produce_identity_constraint(builder_ptr, evaluation, degree) {
                if gt(add(degree, 1), mload(add(builder_ptr, BUILDER_MAX_DEGREE_OFFSET))) {
                    err(ERR_CONSTRAINT_DEGREE_TOO_HIGH)
                }
                // builder.aggregateEvaluation +=
                //     evaluation * dequeue(builder.constraintMultipliers) * builder.rowMultipliersEvaluation;
                mstore(
                    add(builder_ptr, BUILDER_AGGREGATE_EVALUATION_OFFSET),
                    addmod(
                        mload(add(builder_ptr, BUILDER_AGGREGATE_EVALUATION_OFFSET)),
                        mulmod(
                            evaluation,
                            mulmod(
                                dequeue(add(builder_ptr, BUILDER_CONSTRAINT_MULTIPLIERS_OFFSET)),
                                mload(add(builder_ptr, BUILDER_ROW_MULTIPLIERS_EVALUATION_OFFSET)),
                                MODULUS
                            ),
                            MODULUS
                        ),
                        MODULUS
                    )
                )
            }
            builder_produce_identity_constraint(__builder, __evaluation, __degree)
        }
    }

    /// @notice Sets the column evaluations in the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_set_column_evaluations(builder_ptr, values_ptr)
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// * `values_ptr` - pointer to the array in memory
    /// @param __builder The builder struct
    /// @param __values The column evaluation values array
    function __setColumnEvaluations(Builder memory __builder, uint256[] memory __values) internal pure {
        assembly {
            function builder_set_column_evaluations(builder_ptr, values_ptr) {
                mstore(add(builder_ptr, BUILDER_COLUMN_EVALUATIONS_OFFSET), values_ptr)
            }
            builder_set_column_evaluations(__builder, __values)
        }
    }

    /// @notice Gets the column evaluations array from the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_get_column_evaluations(builder_ptr) -> values_ptr
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// ##### Return Values
    /// * `values_ptr` - pointer to the array in memory
    /// @param __builder The builder struct
    /// @return __values The column evaluation values array
    function __getColumnEvaluations(Builder memory __builder) internal pure returns (uint256[] memory __values) {
        assembly {
            function builder_get_column_evaluations(builder_ptr) -> values_ptr {
                values_ptr := mload(add(builder_ptr, BUILDER_COLUMN_EVALUATIONS_OFFSET))
            }
            __values := builder_get_column_evaluations(__builder)
        }
    }

    /// @notice Gets a column evaluation by column number
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_get_column_evaluation(builder_ptr, column_num) -> value
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// * `column_num` - the column number to get evaluation for
    /// ##### Return Values
    /// * `value` - the column evaluation
    /// @param __builder The builder struct
    /// @param __columnNum The column number
    /// @return __value The column evaluation value
    function __getColumnEvaluation(Builder memory __builder, uint256 __columnNum)
        internal
        pure
        returns (uint256 __value)
    {
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Array.pre.sol
            function get_array_element(arr_ptr, index) -> value {
                revert(0, 0)
            }
            function builder_get_column_evaluation(builder_ptr, column_num) -> value {
                value := get_array_element(add(builder_ptr, BUILDER_COLUMN_EVALUATIONS_OFFSET), column_num)
            }
            __value := builder_get_column_evaluation(__builder, __columnNum)
        }
    }

    /// @notice Sets the table chi evaluations in the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_set_table_chi_evaluations(builder_ptr, values_ptr)
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// * `values_ptr` - pointer to the array in memory
    /// @param __builder The builder struct
    /// @param __values The table chi evaluation values array
    function __setTableChiEvaluations(Builder memory __builder, uint256[] memory __values) internal pure {
        assembly {
            function builder_set_table_chi_evaluations(builder_ptr, values_ptr) {
                mstore(add(builder_ptr, BUILDER_TABLE_CHI_EVALUATIONS_OFFSET), values_ptr)
            }
            builder_set_table_chi_evaluations(__builder, __values)
        }
    }

    /// @notice Gets a table chi evaluation by table number
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_get_table_chi_evaluation(builder_ptr, table_num) -> value
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// * `table_num` - the table number to get evaluation for
    /// ##### Return Values
    /// * `value` - the table chi evaluation
    /// @param __builder The builder struct
    /// @param __tableNum The table number
    /// @return __value The table chi evaluation value
    function __getTableChiEvaluation(Builder memory __builder, uint256 __tableNum)
        internal
        pure
        returns (uint256 __value)
    {
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Array.pre.sol
            function get_uint512_array_element(arr_ptr, index) -> upper, lower {
                revert(0, 0)
            }
            function builder_get_table_chi_evaluation(builder_ptr, table_num) -> value {
                let length
                length, value :=
                    get_uint512_array_element(add(builder_ptr, BUILDER_TABLE_CHI_EVALUATIONS_OFFSET), table_num)
            }
            __value := builder_get_table_chi_evaluation(__builder, __tableNum)
        }
    }

    /// @notice Gets a table chi length and evaluation by table number
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_get_table_chi_evaluation_with_length(builder_ptr, table_num) -> length, chi_eval
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// * `table_num` - the table number to get evaluation for
    /// ##### Return Values
    /// * `length` - the length of the column of ones
    /// * `chi_eval` - the table chi evaluation
    /// @param __builder The builder struct
    /// @param __tableNum The table number
    /// @return __length The length of the column of ones
    /// @return __chiEval The consumed chi column evaluation value
    function __getTableChiEvaluationWithLength(Builder memory __builder, uint256 __tableNum)
        internal
        pure
        returns (uint256 __length, uint256 __chiEval)
    {
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Array.pre.sol
            function get_uint512_array_element(arr_ptr, index) -> upper, lower {
                revert(0, 0)
            }
            function builder_get_table_chi_evaluation_with_length(builder_ptr, table_num) -> length, chi_eval {
                length, chi_eval :=
                    get_uint512_array_element(add(builder_ptr, BUILDER_TABLE_CHI_EVALUATIONS_OFFSET), table_num)
            }
            __length, __chiEval := builder_get_table_chi_evaluation_with_length(__builder, __tableNum)
        }
    }

    /// @notice Sets the first round commitments in the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_set_first_round_commitments(builder_ptr, values_ptr)
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// * `values_ptr` - memory pointer to the first round commitments array
    /// @param __builder The builder struct
    /// @param __valuesPtr Memory pointer to first round commitments array
    function __setFirstRoundCommitments(Builder memory __builder, uint256 __valuesPtr) internal pure {
        assembly {
            function builder_set_first_round_commitments(builder_ptr, values_ptr) {
                mstore(add(builder_ptr, BUILDER_FIRST_ROUND_COMMITMENTS_OFFSET), values_ptr)
            }
            builder_set_first_round_commitments(__builder, __valuesPtr)
        }
    }

    /// @notice Gets the first round commitments pointer from the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_get_first_round_commitments(builder_ptr) -> values_ptr
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// ##### Return Values
    /// * `values_ptr` - memory pointer to the first round commitments array
    /// @param __builder The builder struct
    /// @return __valuesPtr Memory pointer to first round commitments array
    function __getFirstRoundCommitments(Builder memory __builder) internal pure returns (uint256 __valuesPtr) {
        assembly {
            function builder_get_first_round_commitments(builder_ptr) -> values_ptr {
                values_ptr := mload(add(builder_ptr, BUILDER_FIRST_ROUND_COMMITMENTS_OFFSET))
            }
            __valuesPtr := builder_get_first_round_commitments(__builder)
        }
    }

    /// @notice Sets the final round commitments in the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_set_final_round_commitments(builder_ptr, values_ptr)
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// * `values_ptr` - memory pointer to the final round commitments array
    /// @param __builder The builder struct
    /// @param __valuesPtr Memory pointer to final round commitments array
    function __setFinalRoundCommitments(Builder memory __builder, uint256 __valuesPtr) internal pure {
        assembly {
            function builder_set_final_round_commitments(builder_ptr, values_ptr) {
                mstore(add(builder_ptr, BUILDER_FINAL_ROUND_COMMITMENTS_OFFSET), values_ptr)
            }
            builder_set_final_round_commitments(__builder, __valuesPtr)
        }
    }

    /// @notice Gets the final round commitments pointer from the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_get_final_round_commitments(builder_ptr) -> values_ptr
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// ##### Return Values
    /// * `values_ptr` - memory pointer to the final round commitments array
    /// @param __builder The builder struct
    /// @return __valuesPtr Memory pointer to final round commitments array
    function __getFinalRoundCommitments(Builder memory __builder) internal pure returns (uint256 __valuesPtr) {
        assembly {
            function builder_get_final_round_commitments(builder_ptr) -> values_ptr {
                values_ptr := mload(add(builder_ptr, BUILDER_FINAL_ROUND_COMMITMENTS_OFFSET))
            }
            __valuesPtr := builder_get_final_round_commitments(__builder)
        }
    }

    /// @notice Sets the bit distributions in the verification builder. Errors if the length is non-zero.
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_set_bit_distributions(builder_ptr, values_ptr)
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// * `values_ptr` - pointer to the array in memory
    /// @dev Always reverts with Errors.UnsupportedProof if length is non-zero
    /// @param __builder The builder struct
    /// @param __values The bit distributions array
    function __setBitDistributions(Builder memory __builder, uint256[] memory __values) internal pure {
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            function builder_set_bit_distributions(builder_ptr, values_ptr) {
                mstore(add(builder_ptr, BUILDER_FINAL_ROUND_BIT_DISTRIBUTIONS_OFFSET), values_ptr)
            }
            builder_set_bit_distributions(__builder, __values)
        }
    }

    /// @notice Consumes a final round bit distribution from the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_consume_bit_distribution(builder_ptr) -> vary_mask, leading_bit_mask
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// ##### Return Values
    /// * `vary_mask` - the vary mask of the bit distribution
    /// * `leading_bit_mask` - the leading bit mask of the bit distribution
    /// @dev Dequeues and returns a final round bit distribution. Reverts with Errors.EmptyQueue if no values remain
    /// @param __builder The builder struct
    /// @return __varyMask the vary mask of the consumed bit distribution
    /// @return __leadingBitMask the leading bit mask of the consumed bit distribution
    function __consumeBitDistribution(Builder memory __builder)
        internal
        pure
        returns (uint256 __varyMask, uint256 __leadingBitMask)
    {
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Queue.pre.sol
            function dequeue_uint512(queue_ptr) -> upper, lower {
                revert(0, 0)
            }
            function builder_consume_bit_distribution(builder_ptr) -> vary_mask, leading_bit_mask {
                let values_ptr := add(builder_ptr, BUILDER_FINAL_ROUND_BIT_DISTRIBUTIONS_OFFSET)
                vary_mask, leading_bit_mask := dequeue_uint512(values_ptr)
            }
            __varyMask, __leadingBitMask := builder_consume_bit_distribution(__builder)
        }
    }

    /// @notice Gets the chi column evaluations array from the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_get_chi_evaluations(builder_ptr) -> values_ptr
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// ##### Return Values
    /// * `values_ptr` - pointer to the array in memory
    /// @param __builder The builder struct
    /// @return __values The chi evaluations array
    function __getChiEvaluations(Builder memory __builder) internal pure returns (uint256[] memory __values) {
        assembly {
            function builder_get_chi_evaluations(builder_ptr) -> values_ptr {
                values_ptr := mload(add(builder_ptr, BUILDER_CHI_EVALUATIONS_OFFSET))
            }
            __values := builder_get_chi_evaluations(__builder)
        }
    }

    /// @notice Gets the table chi evaluations array from the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_get_table_chi_evaluations(builder_ptr) -> values_ptr
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// ##### Return Values
    /// * `values_ptr` - pointer to the array in memory
    /// @param __builder The builder struct
    /// @return __values The table chi evaluations array
    function __getTableChiEvaluations(Builder memory __builder) internal pure returns (uint256[] memory __values) {
        assembly {
            function builder_get_table_chi_evaluations(builder_ptr) -> values_ptr {
                values_ptr := mload(add(builder_ptr, BUILDER_TABLE_CHI_EVALUATIONS_OFFSET))
            }
            __values := builder_get_table_chi_evaluations(__builder)
        }
    }

    /// @notice Gets the rho column evaluations array from the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_get_rho_evaluations(builder_ptr) -> values_ptr
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// ##### Return Values
    /// * `values_ptr` - pointer to the array in memory
    /// @param __builder The builder struct
    /// @return __values The rho evaluations array
    function __getRhoEvaluations(Builder memory __builder) internal pure returns (uint256[] memory __values) {
        assembly {
            function builder_get_rho_evaluations(builder_ptr) -> values_ptr {
                values_ptr := mload(add(builder_ptr, BUILDER_RHO_EVALUATIONS_OFFSET))
            }
            __values := builder_get_rho_evaluations(__builder)
        }
    }

    /// @notice Checks if the aggregate evaluation is non-zero and triggers an error if so
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_check_aggregate_evaluation(builder_ptr)
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// @param __builder The builder struct
    function __checkAggregateEvaluation(Builder memory __builder) internal pure {
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            function builder_check_aggregate_evaluation(builder_ptr) {
                if mload(add(builder_ptr, BUILDER_AGGREGATE_EVALUATION_OFFSET)) {
                    err(ERR_AGGREGATE_EVALUATION_MISMATCH)
                }
            }
            builder_check_aggregate_evaluation(__builder)
        }
    }

    /// @notice Sets the singleton chi evaluation in the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_set_singleton_chi_evaluation(builder_ptr, value)
    /// ```
    /// @param __builder The builder struct
    /// @param __value The singleton chi evaluation value
    function __setSingletonChiEvaluation(Builder memory __builder, uint256 __value) internal pure {
        assembly {
            function builder_set_singleton_chi_evaluation(builder_ptr, value) {
                mstore(add(builder_ptr, BUILDER_SINGLETON_CHI_EVALUATION_OFFSET), value)
            }
            builder_set_singleton_chi_evaluation(__builder, __value)
        }
    }

    /// @notice Gets the singleton chi evaluation from the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_get_singleton_chi_evaluation(builder_ptr) -> value
    /// ```
    /// @param __builder The builder struct
    /// @return __value The singleton chi evaluation value
    function __getSingletonChiEvaluation(Builder memory __builder) internal pure returns (uint256 __value) {
        assembly {
            function builder_get_singleton_chi_evaluation(builder_ptr) -> value {
                value := mload(add(builder_ptr, BUILDER_SINGLETON_CHI_EVALUATION_OFFSET))
            }
            __value := builder_get_singleton_chi_evaluation(__builder)
        }
    }

    /// @notice Sets the placeholder parameters in the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_set_placeholder_parameters(builder_ptr, values_ptr)
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// * `values_ptr` - pointer to the array in memory. In Solidity memory layout,
    ///   this points to where the array length is stored, followed by the array elements
    /// @dev Stores the placeholder parameters array pointer in the builder structure.
    /// @param __builder The builder struct
    /// @param __placeholderParameters The placeholder parameters array (already converted to scalars)
    function __setPlaceholderParameters(Builder memory __builder, uint256[] memory __placeholderParameters)
        internal
        pure
    {
        assembly {
            function builder_set_placeholder_parameters(builder_ptr, values_ptr) {
                mstore(add(builder_ptr, BUILDER_PLACEHOLDER_PARAMETERS_OFFSET), values_ptr)
            }
            builder_set_placeholder_parameters(__builder, __placeholderParameters)
        }
    }

    /// @notice Gets a placeholder parameter value from the verification builder
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// builder_get_placeholder_parameter(builder_ptr, index) -> value
    /// ```
    /// ##### Parameters
    /// * `builder_ptr` - memory pointer to the builder struct region
    /// * `index` - the index of the placeholder parameter to retrieve
    /// ##### Return Values
    /// * `value` - the placeholder parameter value at the given index
    /// @dev Gets a placeholder parameter by index. Reverts if index is out of bounds
    /// @param __builder The builder struct
    /// @param __index The index of the placeholder parameter
    /// @return __value The placeholder parameter value
    function __getPlaceholderParameter(Builder memory __builder, uint256 __index)
        internal
        pure
        returns (uint256 __value)
    {
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Array.pre.sol
            function get_array_element(arr_ptr, index) -> value {
                revert(0, 0)
            }
            function builder_get_placeholder_parameter(builder_ptr, index) -> value {
                value := get_array_element(add(builder_ptr, BUILDER_PLACEHOLDER_PARAMETERS_OFFSET), index)
            }
            __value := builder_get_placeholder_parameter(__builder, __index)
        }
    }
}
