// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {Errors} from "../../src/base/Errors.sol";
import {VerificationBuilder} from "../../src/builder/VerificationBuilder.pre.sol";
import {SortMergeJoinExec} from "../../src/proof_plans/SortMergeJoinExec.pre.sol";

contract SortMergeJoinExecTest is Test {
    uint256 private constant _CHI_TOTAL = 4;
    uint256 private constant _RHO_TOTAL = 6;
    uint256 private constant _FIRST_ROUND_MLE_TOTAL = 6;
    uint256 private constant _FINAL_ROUND_MLE_TOTAL_PART_ONE = 14;
    uint256 private constant _FINAL_ROUND_MLE_TOTAL_PART_TWO = 11;
    uint256 private constant _BIT_DITRIBUTION_TOTAL = 2;
    uint256 private constant _ROW_TOTAL = 4;
    uint256 private constant _VARYING_BIT_TOTAL = 63;
    uint256 private constant _FINAL_ROUND_MLE_TOTAL =
        _FINAL_ROUND_MLE_TOTAL_PART_ONE + _FINAL_ROUND_MLE_TOTAL_PART_TWO + _VARYING_BIT_TOTAL;

    /* solhint-disable code-complexity */
    function testSortMergeJoinExec() public pure {
        VerificationBuilder.Builder memory builder;
        uint256 chiLength;
        uint256 chiEval;
        bytes memory plan;

        {
            bytes memory leftPlan = abi.encodePacked(
                TABLE_EXEC_VARIANT,
                uint64(0), // table_ref
                uint64(3), // column_count
                uint64(0), // column1_index
                uint64(1), // column2_index
                uint64(2) // column3_index
            );

            bytes memory leftJoinIndices = abi.encodePacked(
                uint64(1), // join index count
                uint64(1)
            );

            bytes memory rightPlan = abi.encodePacked(
                TABLE_EXEC_VARIANT,
                uint64(1), // table_ref
                uint64(3), // column_count
                uint64(3), // column1_index
                uint64(4), // column2_index
                uint64(5) // column3_index
            );

            bytes memory rightJoinIndices = abi.encodePacked(
                uint64(1), // join index count
                uint64(0)
            );
            plan = abi.encodePacked(leftPlan, rightPlan, leftJoinIndices, rightJoinIndices, hex"abcdef");
        }

        // challenges
        {
            builder.challenges = new uint256[](2 * _ROW_TOTAL);
            for (uint8 i = 0; i < _ROW_TOTAL; ++i) {
                builder.challenges[2 * i] = 3; // alpha
                builder.challenges[2 * i + 1] = 8; // beta
            }
        }

        // chi evals
        builder.chiEvaluations = new uint256[](8 * _CHI_TOTAL);
        {
            uint256[_CHI_TOTAL] memory lengths = [
                uint256(3), // output
                4, // i shifted
                2, // u
                3 // u shifted
            ];
            for (uint8 i = 0; i < _ROW_TOTAL; ++i) {
                for (uint8 j = 0; j < _CHI_TOTAL; ++j) {
                    uint256 length = lengths[j];
                    builder.chiEvaluations[2 * (i * _CHI_TOTAL + j)] = length;
                    uint256 eval = 0;
                    if (length > i) {
                        eval = 1;
                    }
                    builder.chiEvaluations[2 * (i * _CHI_TOTAL + j) + 1] = eval;
                }
            }
        }

        builder.tableChiEvaluations = new uint256[](4);
        builder.tableChiEvaluations[0] = 3;
        builder.tableChiEvaluations[1] = 1;
        builder.tableChiEvaluations[2] = 2;
        builder.tableChiEvaluations[3] = 1;
        builder.singletonChiEvaluation = 1;

        // rho evals
        builder.rhoEvaluations = new uint256[](_ROW_TOTAL * _RHO_TOTAL);
        {
            uint256[_RHO_TOTAL] memory lengths = [
                uint256(3), // left
                2, // right
                3, // i unshifted
                4, // i shifted
                2, // u unshifted
                3 // u shifted
            ];
            for (uint8 i = 0; i < _ROW_TOTAL; ++i) {
                for (uint8 j = 0; j < _RHO_TOTAL; ++j) {
                    uint256 eval = 0;
                    if (lengths[j] > i) {
                        eval = i;
                    }
                    builder.rhoEvaluations[i * _RHO_TOTAL + j] = eval;
                }
            }
        }

        // mles
        builder.firstRoundMLEs = new uint256[](_ROW_TOTAL * _FIRST_ROUND_MLE_TOTAL);
        {
            uint256[_FIRST_ROUND_MLE_TOTAL][_ROW_TOTAL] memory firstRoundMLEs = [
                [
                    1, // left hat multiplicity
                    1, // right hat multiplicity
                    0, // shifted i eval
                    uint256(0), // shifted u eval
                    2, // left join multiplicity
                    1 // right join multiplicity
                ],
                [
                    1, // left hat multiplicity
                    2, // right hat multiplicity
                    1, // shifted i eval
                    MODULUS_MINUS_ONE, // shifted u eval
                    1, // left join multiplicity
                    1 // right join multiplicity
                ],
                [
                    1, // left hat multiplicity
                    0, // right hat multiplicity
                    18446744073709551616, // shifted i eval
                    uint256(1), // shifted u eval
                    0, // left join multiplicity
                    0 // right join multiplicity
                ],
                [
                    0, // left hat multiplicity
                    0, // right hat multiplicity
                    36893488147419103233, // shifted i eval
                    uint256(0), // shifted u eval
                    0, // left join multiplicity
                    0 // right join multiplicity
                ]
            ];

            for (uint8 i = 0; i < _ROW_TOTAL; ++i) {
                for (uint8 j = 0; j < _FIRST_ROUND_MLE_TOTAL; ++j) {
                    builder.firstRoundMLEs[i * _FIRST_ROUND_MLE_TOTAL + j] = firstRoundMLEs[i][j];
                }
            }
        }

        builder.finalRoundMLEs = new uint256[](_ROW_TOTAL * (_FINAL_ROUND_MLE_TOTAL));
        {
            uint256[_FINAL_ROUND_MLE_TOTAL_PART_ONE + _FINAL_ROUND_MLE_TOTAL_PART_TWO][_ROW_TOTAL] memory finalRoundMLEs =
            [
                [
                    MODULUS_MINUS_ONE, // join eval
                    1, // left output column eval 0
                    3, // left output column eval 2
                    0, // left rho eval
                    10074446955173859956738117514536196637923519413252069308468488945040793052664, // left hat c_star
                    10074446955173859956738117514536196637923519413252069308468488945040793052664, // left hat d_star
                    5, // right output column eval 1
                    6, // right output column eval 2
                    1, // right rho eval
                    8039225893859938267920868448655644390720148984628380656337469920855181461851, // right hat c_star
                    460266789361106254673405728288120270553587101877907264236644480558837094534, // right hat d_star
                    11725844395628183154774860220673540226008052357365732684124037957094183122652, // c_star for i shift
                    1, // d_star for i shift gadget
                    1, // i bit eval for sign gadget
                    1, // i bit eval for sign gadget
                    1, // i bit eval for sign gadget
                    0, // i bit eval for sign gadget
                    MODULUS_MINUS_ONE, // u eval
                    14923801958072233106077094826311778469464793909374568870703321036301687610648, // c_star for shift gadget
                    1, // d_star for shift gadget
                    1, // bit eval for sign gadget for u shift
                    10944121435919637611123202872628637544274182200208017171849102093287904247808, // left join c_star
                    10944121435919637611123202872628637544274182200208017171849102093287904247808, // left join d_star
                    10944121435919637611123202872628637544274182200208017171849102093287904247808, // right join c_star
                    16416182153879456416684804308942956316411273300312025757773653139931856371713 // right join d_star
                ],
                [
                    uint256(1), // join eval
                    1, // left column eval 0
                    3, // left column eval 2
                    1, // left rho eval
                    2851295496054451040591965271693713772621322413579694052532748328073899665449, // left hat c_star
                    2851295496054451040591965271693713772621322413579694052532748328073899665449, // left hat d_star
                    5, // right output column eval 1
                    6, // right output column eval 2
                    0, // right rho eval
                    460266789361106254673405728288120270553587101877907264236644480558837094534, // right hat c_star
                    8039225893859938267920868448655644390720148984628380656337469920855181461851, // right hat d_star
                    6496471390482283403974624657428010136580938079855111550940942529249946921050, // c_star for i shift
                    11725844395628183154774860220673540226008052357365732684124037957094183122652, // d_star for i shift gadget
                    1, // i bit eval for sign gadget
                    1, // i bit eval for sign gadget
                    1, // i bit eval for sign gadget
                    0, // i bit eval for sign gadget
                    1, // u eval
                    21467315124303904544895513327079250567614742008100341375550161798372427563009, // c_star for shift gadget
                    14923801958072233106077094826311778469464793909374568870703321036301687610648, // d_star for shift gadget
                    0, // bit eval for sign gadget
                    16416182153879456416684804308942956316411273300312025757773653139931856371713, // left join c_star
                    16416182153879456416684804308942956316411273300312025757773653139931856371713, // left join d_star
                    16416182153879456416684804308942956316411273300312025757773653139931856371713, // right join c_star
                    10944121435919637611123202872628637544274182200208017171849102093287904247808 // right join d_star
                ],
                [
                    MODULUS_MINUS_ONE, // join eval
                    1, // left column eval 0
                    3, // left column eval 2
                    2, // left rho eval
                    19344707929420007666775874800946745888535234308035673040517464253432216520237, // left hat c_star
                    19344707929420007666775874800946745888535234308035673040517464253432216520237, // left hat d_star
                    5, // right output column eval 1
                    6, // right output column eval 2
                    1, // right rho eval
                    0, // right hat c_star
                    460266789361106254673405728288120270553587101877907264236644480558837094534, // right hat d_star
                    18735783757041287116993268168181089483405786268842422841364722931849478463737, // c_star for i shift
                    6496471390482283403974624657428010136580938079855111550940942529249946921050, // d_star for i shift gadget
                    1, // i bit eval for sign gadget
                    0, // i bit eval for sign gadget
                    1, // i bit eval for sign gadget
                    0, // i bit eval for sign gadget
                    0, // u eval
                    1, // c_star for shift gadget
                    21467315124303904544895513327079250567614742008100341375550161798372427563009, // d_star for shift gadget
                    1, // bit eval for sign gadget
                    0, // left join c_star
                    10944121435919637611123202872628637544274182200208017171849102093287904247808, // left join d_star
                    0, // right join c_star
                    0 // right join d_star
                ],
                [
                    uint256(0), // join eval
                    0, // left column eval 0
                    0, // left column eval 2
                    0, // left rho eval
                    0, // left hat c_star
                    0, // left hat d_star
                    0, // right output column eval 1
                    0, // right output column eval 2
                    0, // right rho eval
                    0, // right hat c_star
                    0, // right hat d_star
                    1, // c_star for i shift
                    18735783757041287116993268168181089483405786268842422841364722931849478463737, // d_star for i shift gadget
                    1, // i bit eval for sign gadget
                    0, // i bit eval for sign gadget
                    1, // i bit eval for sign gadget
                    1, // i bit eval for sign gadget
                    0, // u eval
                    0, // c_star for shift gadget
                    0, // d_star for shift gadget
                    0, // bit eval for sign gadget
                    0, // left join c_star
                    0, // left join d_star
                    0, // right join c_star
                    0 // right join d_star
                ]
            ];
            uint256[_ROW_TOTAL] memory bitEvals = [uint256(1), 0, 1, 0];

            for (uint8 i = 0; i < _ROW_TOTAL; ++i) {
                for (uint8 j = 0; j < _FINAL_ROUND_MLE_TOTAL_PART_ONE; ++j) {
                    builder.finalRoundMLEs[i * (_FINAL_ROUND_MLE_TOTAL) + j] = finalRoundMLEs[i][j];
                }
                for (uint8 j = 0; j < _VARYING_BIT_TOTAL; ++j) {
                    builder.finalRoundMLEs[i * _FINAL_ROUND_MLE_TOTAL + _FINAL_ROUND_MLE_TOTAL_PART_ONE + j] =
                        bitEvals[i];
                }

                for (
                    uint256 j = _FINAL_ROUND_MLE_TOTAL_PART_ONE;
                    j < _FINAL_ROUND_MLE_TOTAL_PART_ONE + _FINAL_ROUND_MLE_TOTAL_PART_TWO;
                    ++j
                ) {
                    builder.finalRoundMLEs[i * _FINAL_ROUND_MLE_TOTAL + _VARYING_BIT_TOTAL + j] = finalRoundMLEs[i][j];
                }
            }
        }

        // bit distributions
        {
            uint256[2 * _BIT_DITRIBUTION_TOTAL] memory bitDistributions = [
                57896044618658097711785492504343953926634992332820282019802578980251403026431,
                0,
                0x8000000000000000000000000000000000000000000000000000000000000000,
                1
            ];
            builder.bitDistributions = new uint256[](2 * _BIT_DITRIBUTION_TOTAL * _ROW_TOTAL);
            for (uint8 i = 0; i < _ROW_TOTAL; ++i) {
                for (uint8 j = 0; j < 2 * _BIT_DITRIBUTION_TOTAL; ++j) {
                    builder.bitDistributions[2 * i * _BIT_DITRIBUTION_TOTAL + j] = bitDistributions[j];
                }
            }
        }

        builder.maxDegree = 3;
        builder.rowMultipliersEvaluation = 1;
        builder.aggregateEvaluation = 0;
        builder.constraintMultipliers = new uint256[](1000);
        for (uint16 i = 0; i < 1000; ++i) {
            builder.constraintMultipliers[i] = 1;
        }

        // column evaluations
        uint256[6][_ROW_TOTAL] memory columnEvaluations = [
            [uint256(1), MODULUS_MINUS_ONE, 3, 1, 5, 6],
            [uint256(1), 1, 3, MODULUS_MINUS_ONE, 5, 6],
            [uint256(1), MODULUS_MINUS_ONE, 3, 0, 0, 0],
            [uint256(0), 0, 0, 0, 0, 0]
        ];
        builder.columnEvaluations = new uint256[](6);

        for (uint8 i = 0; i < _ROW_TOTAL; ++i) {
            for (uint8 j = 0; j < 6; ++j) {
                builder.columnEvaluations[j] = columnEvaluations[i][j];
            }
            if (i == 2) {
                builder.tableChiEvaluations[3] = 0;
            }
            if (i == 3) {
                builder.tableChiEvaluations[1] = 0;
            }
            if (i == 1) {
                builder.singletonChiEvaluation = 0;
            }
            uint256[] memory evals;
            bytes memory planOut;
            (planOut, builder, evals, chiLength, chiEval) = SortMergeJoinExec.__sortMergeJoinEvaluate(plan, builder);
        }
        assert(builder.aggregateEvaluation == 0);
    }

    function testSortMergeJoinExecTooManyJoinColumns() public {
        VerificationBuilder.Builder memory builder;
        bytes memory plan;

        {
            bytes memory leftPlan = abi.encodePacked(
                TABLE_EXEC_VARIANT,
                uint64(0), // table_ref
                uint64(3), // column_count
                uint64(0), // column1_index
                uint64(1), // column2_index
                uint64(2) // column3_index
            );

            bytes memory leftJoinIndices = abi.encodePacked(
                uint64(2), // join index count
                uint64(1),
                uint64(2)
            );

            bytes memory rightPlan = abi.encodePacked(
                TABLE_EXEC_VARIANT,
                uint64(1), // table_ref
                uint64(3), // column_count
                uint64(3), // column1_index
                uint64(4), // column2_index
                uint64(5) // column3_index
            );

            bytes memory rightJoinIndices = abi.encodePacked(
                uint64(1), // join index count
                uint64(0)
            );
            plan = abi.encodePacked(leftPlan, rightPlan, leftJoinIndices, rightJoinIndices, hex"abcdef");
        }

        // challenges
        {
            builder.challenges = new uint256[](2 * _ROW_TOTAL);
            for (uint8 i = 0; i < _ROW_TOTAL; ++i) {
                builder.challenges[2 * i] = 3; // alpha
                builder.challenges[2 * i + 1] = 8; // beta
            }
        }

        // chi evals
        builder.chiEvaluations = new uint256[](8 * _CHI_TOTAL);
        {
            uint256[_CHI_TOTAL] memory lengths = [
                uint256(3), // output
                4, // i shifted
                2, // u
                3 // u shifted
            ];
            for (uint8 i = 0; i < _ROW_TOTAL; ++i) {
                for (uint8 j = 0; j < _CHI_TOTAL; ++j) {
                    uint256 length = lengths[j];
                    builder.chiEvaluations[2 * (i * _CHI_TOTAL + j)] = length;
                    uint256 eval = 0;
                    if (length > i) {
                        eval = 1;
                    }
                    builder.chiEvaluations[2 * (i * _CHI_TOTAL + j) + 1] = eval;
                }
            }
        }

        builder.tableChiEvaluations = new uint256[](4);
        builder.tableChiEvaluations[0] = 3;
        builder.tableChiEvaluations[1] = 1;
        builder.tableChiEvaluations[2] = 2;
        builder.tableChiEvaluations[3] = 1;
        builder.singletonChiEvaluation = 1;

        // rho evals
        builder.rhoEvaluations = new uint256[](_ROW_TOTAL * _RHO_TOTAL);
        {
            uint256[_RHO_TOTAL] memory lengths = [
                uint256(3), // left
                2, // right
                3, // i unshifted
                4, // i shifted
                2, // u unshifted
                3 // u shifted
            ];
            for (uint8 i = 0; i < _ROW_TOTAL; ++i) {
                for (uint8 j = 0; j < _RHO_TOTAL; ++j) {
                    uint256 eval = 0;
                    if (lengths[j] > i) {
                        eval = i;
                    }
                    builder.rhoEvaluations[i * _RHO_TOTAL + j] = eval;
                }
            }
        }

        // mles
        builder.firstRoundMLEs = new uint256[](_ROW_TOTAL * _FIRST_ROUND_MLE_TOTAL);
        {
            uint256[_FIRST_ROUND_MLE_TOTAL][_ROW_TOTAL] memory firstRoundMLEs = [
                [
                    1, // left hat multiplicity
                    1, // right hat multiplicity
                    0, // shifted i eval
                    uint256(0), // shifted u eval
                    2, // left join multiplicity
                    1 // right join multiplicity
                ],
                [
                    1, // left hat multiplicity
                    2, // right hat multiplicity
                    1, // shifted i eval
                    MODULUS_MINUS_ONE, // shifted u eval
                    1, // left join multiplicity
                    1 // right join multiplicity
                ],
                [
                    1, // left hat multiplicity
                    0, // right hat multiplicity
                    18446744073709551616, // shifted i eval
                    uint256(1), // shifted u eval
                    0, // left join multiplicity
                    0 // right join multiplicity
                ],
                [
                    0, // left hat multiplicity
                    0, // right hat multiplicity
                    36893488147419103233, // shifted i eval
                    uint256(0), // shifted u eval
                    0, // left join multiplicity
                    0 // right join multiplicity
                ]
            ];

            for (uint8 i = 0; i < _ROW_TOTAL; ++i) {
                for (uint8 j = 0; j < _FIRST_ROUND_MLE_TOTAL; ++j) {
                    builder.firstRoundMLEs[i * _FIRST_ROUND_MLE_TOTAL + j] = firstRoundMLEs[i][j];
                }
            }
        }

        builder.finalRoundMLEs = new uint256[](_ROW_TOTAL * (_FINAL_ROUND_MLE_TOTAL));
        {
            uint256[_FINAL_ROUND_MLE_TOTAL_PART_ONE + _FINAL_ROUND_MLE_TOTAL_PART_TWO][_ROW_TOTAL] memory finalRoundMLEs =
            [
                [
                    MODULUS_MINUS_ONE, // join eval
                    1, // left output column eval 0
                    3, // left output column eval 2
                    0, // left rho eval
                    10074446955173859956738117514536196637923519413252069308468488945040793052664, // left hat c_star
                    10074446955173859956738117514536196637923519413252069308468488945040793052664, // left hat d_star
                    5, // right output column eval 1
                    6, // right output column eval 2
                    1, // right rho eval
                    8039225893859938267920868448655644390720148984628380656337469920855181461851, // right hat c_star
                    460266789361106254673405728288120270553587101877907264236644480558837094534, // right hat d_star
                    11725844395628183154774860220673540226008052357365732684124037957094183122652, // c_star for i shift
                    1, // d_star for i shift gadget
                    1, // i bit eval for sign gadget
                    1, // i bit eval for sign gadget
                    1, // i bit eval for sign gadget
                    0, // i bit eval for sign gadget
                    MODULUS_MINUS_ONE, // u eval
                    14923801958072233106077094826311778469464793909374568870703321036301687610648, // c_star for shift gadget
                    1, // d_star for shift gadget
                    1, // bit eval for sign gadget for u shift
                    10944121435919637611123202872628637544274182200208017171849102093287904247808, // left join c_star
                    10944121435919637611123202872628637544274182200208017171849102093287904247808, // left join d_star
                    10944121435919637611123202872628637544274182200208017171849102093287904247808, // right join c_star
                    16416182153879456416684804308942956316411273300312025757773653139931856371713 // right join d_star
                ],
                [
                    uint256(1), // join eval
                    1, // left column eval 0
                    3, // left column eval 2
                    1, // left rho eval
                    2851295496054451040591965271693713772621322413579694052532748328073899665449, // left hat c_star
                    2851295496054451040591965271693713772621322413579694052532748328073899665449, // left hat d_star
                    5, // right output column eval 1
                    6, // right output column eval 2
                    0, // right rho eval
                    460266789361106254673405728288120270553587101877907264236644480558837094534, // right hat c_star
                    8039225893859938267920868448655644390720148984628380656337469920855181461851, // right hat d_star
                    6496471390482283403974624657428010136580938079855111550940942529249946921050, // c_star for i shift
                    11725844395628183154774860220673540226008052357365732684124037957094183122652, // d_star for i shift gadget
                    1, // i bit eval for sign gadget
                    1, // i bit eval for sign gadget
                    1, // i bit eval for sign gadget
                    0, // i bit eval for sign gadget
                    1, // u eval
                    21467315124303904544895513327079250567614742008100341375550161798372427563009, // c_star for shift gadget
                    14923801958072233106077094826311778469464793909374568870703321036301687610648, // d_star for shift gadget
                    0, // bit eval for sign gadget
                    16416182153879456416684804308942956316411273300312025757773653139931856371713, // left join c_star
                    16416182153879456416684804308942956316411273300312025757773653139931856371713, // left join d_star
                    16416182153879456416684804308942956316411273300312025757773653139931856371713, // right join c_star
                    10944121435919637611123202872628637544274182200208017171849102093287904247808 // right join d_star
                ],
                [
                    MODULUS_MINUS_ONE, // join eval
                    1, // left column eval 0
                    3, // left column eval 2
                    2, // left rho eval
                    19344707929420007666775874800946745888535234308035673040517464253432216520237, // left hat c_star
                    19344707929420007666775874800946745888535234308035673040517464253432216520237, // left hat d_star
                    5, // right output column eval 1
                    6, // right output column eval 2
                    1, // right rho eval
                    0, // right hat c_star
                    460266789361106254673405728288120270553587101877907264236644480558837094534, // right hat d_star
                    18735783757041287116993268168181089483405786268842422841364722931849478463737, // c_star for i shift
                    6496471390482283403974624657428010136580938079855111550940942529249946921050, // d_star for i shift gadget
                    1, // i bit eval for sign gadget
                    0, // i bit eval for sign gadget
                    1, // i bit eval for sign gadget
                    0, // i bit eval for sign gadget
                    0, // u eval
                    1, // c_star for shift gadget
                    21467315124303904544895513327079250567614742008100341375550161798372427563009, // d_star for shift gadget
                    1, // bit eval for sign gadget
                    0, // left join c_star
                    10944121435919637611123202872628637544274182200208017171849102093287904247808, // left join d_star
                    0, // right join c_star
                    0 // right join d_star
                ],
                [
                    uint256(0), // join eval
                    0, // left column eval 0
                    0, // left column eval 2
                    0, // left rho eval
                    0, // left hat c_star
                    0, // left hat d_star
                    0, // right output column eval 1
                    0, // right output column eval 2
                    0, // right rho eval
                    0, // right hat c_star
                    0, // right hat d_star
                    1, // c_star for i shift
                    18735783757041287116993268168181089483405786268842422841364722931849478463737, // d_star for i shift gadget
                    1, // i bit eval for sign gadget
                    0, // i bit eval for sign gadget
                    1, // i bit eval for sign gadget
                    1, // i bit eval for sign gadget
                    0, // u eval
                    0, // c_star for shift gadget
                    0, // d_star for shift gadget
                    0, // bit eval for sign gadget
                    0, // left join c_star
                    0, // left join d_star
                    0, // right join c_star
                    0 // right join d_star
                ]
            ];
            uint256[_ROW_TOTAL] memory bitEvals = [uint256(1), 0, 1, 0];

            for (uint8 i = 0; i < _ROW_TOTAL; ++i) {
                for (uint8 j = 0; j < _FINAL_ROUND_MLE_TOTAL_PART_ONE; ++j) {
                    builder.finalRoundMLEs[i * (_FINAL_ROUND_MLE_TOTAL) + j] = finalRoundMLEs[i][j];
                }
                for (uint8 j = 0; j < _VARYING_BIT_TOTAL; ++j) {
                    builder.finalRoundMLEs[i * _FINAL_ROUND_MLE_TOTAL + _FINAL_ROUND_MLE_TOTAL_PART_ONE + j] =
                        bitEvals[i];
                }

                for (
                    uint256 j = _FINAL_ROUND_MLE_TOTAL_PART_ONE;
                    j < _FINAL_ROUND_MLE_TOTAL_PART_ONE + _FINAL_ROUND_MLE_TOTAL_PART_TWO;
                    ++j
                ) {
                    builder.finalRoundMLEs[i * _FINAL_ROUND_MLE_TOTAL + _VARYING_BIT_TOTAL + j] = finalRoundMLEs[i][j];
                }
            }
        }

        // bit distributions
        {
            uint256[2 * _BIT_DITRIBUTION_TOTAL] memory bitDistributions = [
                57896044618658097711785492504343953926634992332820282019802578980251403026431,
                0,
                0x8000000000000000000000000000000000000000000000000000000000000000,
                1
            ];
            builder.bitDistributions = new uint256[](2 * _BIT_DITRIBUTION_TOTAL * _ROW_TOTAL);
            for (uint8 i = 0; i < _ROW_TOTAL; ++i) {
                for (uint8 j = 0; j < 2 * _BIT_DITRIBUTION_TOTAL; ++j) {
                    builder.bitDistributions[2 * i * _BIT_DITRIBUTION_TOTAL + j] = bitDistributions[j];
                }
            }
        }

        builder.maxDegree = 3;
        builder.rowMultipliersEvaluation = 1;
        builder.aggregateEvaluation = 0;
        builder.constraintMultipliers = new uint256[](1000);
        for (uint16 i = 0; i < 1000; ++i) {
            builder.constraintMultipliers[i] = 1;
        }
        builder.columnEvaluations = new uint256[](6);

        vm.expectRevert(Errors.NumberOfJoinColumnsNotOne.selector);
        SortMergeJoinExec.__sortMergeJoinEvaluate(plan, builder);
    }
    /* solhint-enable code-complexity */
}
