// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "./../../src/base/Constants.sol";
import {Errors} from "./../../src/base/Errors.sol";
import {Verifier} from "./../../src/verifier/Verifier.pre.sol";

contract VerifierTest is Test {
    function verify(
        bytes calldata result,
        bytes calldata plan,
        uint256[] calldata placeholderParameters,
        bytes calldata proof,
        uint256[] memory tableLengths,
        uint256[] memory commitments
    ) public view {
        Verifier.__internalVerify({
            __result: result,
            __plan: plan,
            __placeholderParameters: placeholderParameters,
            __proof: proof,
            __tableLengths: tableLengths,
            __commitments: commitments
        });
    }

    function testSimpleFull() public view {
        uint256[] memory tableLengths = new uint256[](1);
        tableLengths[0] = 6;
        uint256[] memory commitments = new uint256[](4);
        commitments[0] = 10330259091102222580680469239495014349163118537637164875615751892418606990939;
        commitments[1] = 11920476268597659993223945595713248355766624440856258163660132291159882440306;
        commitments[2] = 11994260765248910561540716452031502208136256599213361935879450273410509855704;
        commitments[3] = 13760418268827664034425846360177624835399216842325030254024970295997593510876;
        uint256[] memory placeholderParameters = new uint256[](0);
        Verifier.__internalVerify({
            __result: hex"00000000000000010000000000000001620000000005000000000000000200000000000000000000000000000003",
            __plan: hex"0000000000000001000000000000000f6e616d6573706163652e7461626c650000000000000002000000000000000000000000000000016200000005000000000000000000000000000000016100000005000000000000000100000000000000016200000000000000000000000000000002000000000000000000000001000000010000000500000000000000050000000000000001000000000000000000000000",
            __placeholderParameters: placeholderParameters,
            __proof: hex"000000000000000600000000000000020000000000000001000000000000000200000000000000000000000000000001236cd1dd5ae57dd08a8b09bd6bd3397fe4b58aa1638c554b8054dcb2d4d0297e12a7e3d2a6d11cf4e66979ea5458d24f66ba7866c43b933fd2135c2a761990140000000000000006000000000000000427a607595971e4f8531109d6bcf114838faac36652aa81d86d51f087cd08799c1095fa761ceaceb796068f4b8174492ede1e88c2f4aafee493331c267284be330078dfd5227feba93faa9a0c9c38432d6c169909d2546f8cef0b7a2108d44db71d518c19207b5ee92cca5cfe09ec5aca67af638b4205a7b650ccb9b9ecd6dd812bbc76361bfaf1e75d70512e9911c71b134404cecbce4053e29ec7c0067717941b11c59c7ba544571e02ba430f9256a63b51e7c8927ffd0d9763fc59bcdcad9521defadf9d440192285b6a0f4e7dc657225821e39bca44e618f7331859e431a62921dd8f70e56f4b52a667f4a851eef537584fb196d14cde47a607f2f6015f3c0000000000000000000000000000000c08a4646bbe5bcbd84f70d57e4199feea96727b0891cd7fc2b9c4548c79134fdc3056f9d01ed8ac370d2d91499df82105cc5842626884827bbd5da23bde9d9f2f27cd3ea9e52ec844140224a5237090c9ed9d1325f920dee410a1f45f884f10f700000000000000000000000000000000000000000000000000000000000000000cb3c1f4236d067d2121b1970bba959f6eb34a5dc55780360adf721cb7706d841e3a39b44aa419741fae45991ff853985acb44d6da117591f5c62bdcd34efa5e0cad3ebe8e7c85e28b999e7e98891b7f5de8dfeb641816693ff4c88ccd378bb10f76ac6934ab963d8cf2de36d5082457f1d4f37b399bab85a7a3e0748b98f4481f5fbcd1225d6fc2179d67784f8cc33f96af63fd2bc3bc0a0b24a2aef032f4e003ede74206e0c0224fbe9c670c28a6a8523bbb853df530b7e8d210d41fa1eeb20567b8ab1dd1ac6b76c90f55380a0e9f6ae397f9f2b29cdd9ed166243e6637591ac21cb2b3d1ed5ae89915fb2dfff88f8d86565b98cf106ed13c21abd2a0d86800000000000000012ab9aab80d43f7435784460a9a510611d5ea4cddb88ea32ccc3c7ca747b95c9d000000000000000216939642c7e9031cbb6e8dc903683c563fe29ffee7344b7c1705ecfb08c627b22f0a59c8d49c3b5d84f2e6e73cf44f2c05005ccf6f4b644e26399fa59e3eab2000000000000000041e837176dd46439a98b5e1aad2c2be57a9cec6c0a6e949e325681657fc59201824588a6ca09cd690acfac8a388cd7c2c2b2105cef96856365a2809a051aaaf231b7e9025b83e0435c1d7f5a8f42da0c4d0809a2b89df0792574778003419af3e2b59a06fd0744f42d07b57f7d8cda7a2032b86fc9eeaf80e4f858faa0e1da9ee00000000000000021cc3fcf38084a75d2d2e8f65edd2e7f1c98e27936ed4ec9c2f83d7d7f953d48d2276ff007d3bda222501c2f5b5ef10b7bf69d177de3ae102dc6e95507b7db4620d6089448b955000aec99031b580405143cd992004d92d0bb840748ad17cce2808947359cf839bc2792b71bb653862169475ccf7d354f0e05ff44b9600fb97500000000000000003120eef7fd4b3ee1684781b3828b14e5ce92a348993e090ef7aefed628428f04111c60d0bb82ab6223350ef46a99b6aaf72d9d10a090dc84c1b4dd1719f6d047829729324a31c3bcf393b4fa25704e2104cfd60e68e43f68892f8b69292efea7202e33bf988181994edcd6d7663b5590339d28e0509d8c0fc0d93334298e1295d12401b7ec5e8363c4ff0d04d7a20ff9584798bf571d5f5ea6a18ac08511742d229d4fa347951ed2d4812b378ac25dc7f278efea58239686edd309c65d1254b20238a63c8139c66b11f73e4490c47bd54517e9a3d84e7e1728bd2545c50070b8f2a5d10f0f8dc22249e979d95b4d5cbede923f576993884fc468d5a675bd7e4f02ae68ad8a170536483d1c64ad7763ee693cb2f963577bd6e530a275337a3480e2eefe01b65c73902b1bd918c0320fa626052596cdd8a43c1be1904df048ca02e16a1c960daf465f7578155147ee1ccf23aff18f620a299a539b3341a5bde26f80ef8f2d1a2febc7c56bf170a18a3215cdc2d2861002a582b7ff03c5c2ceceaf52f21e226bb2d43a1cfc1daee7dad66c1be678f2bb55ec1fc312916b58a240a980687fda634e438a343b885bc944199728f2161d6a244e06ac84a4cdbafdcb6d60e62f98d94384fa7bc869e213dce126be5d98e3b72241f7a615fa4f301d6af09",
            __tableLengths: tableLengths,
            __commitments: commitments
        });
    }

    function testCommitmentArrayOddLength() public {
        uint256[] memory tableLengths = new uint256[](1);
        tableLengths[0] = 6;

        // Create an array with an odd length (5 elements)
        uint256[] memory commitments = new uint256[](5);
        commitments[0] = 10330259091102222580680469239495014349163118537637164875615751892418606990939;
        commitments[1] = 11920476268597659993223945595713248355766624440856258163660132291159882440306;
        commitments[2] = 11994260765248910561540716452031502208136256599213361935879450273410509855704;
        commitments[3] = 13760418268827664034425846360177624835399216842325030254024970295997593510876;
        commitments[4] = 12345678901234567890123456789012345678901234567890123456789012345678901234567; // Add extra element

        uint256[] memory placeholderParameters = new uint256[](0);

        // Expect the error for odd-length commitment array
        vm.expectRevert(Errors.CommitmentArrayOddLength.selector);

        Verifier.__internalVerify({
            __result: hex"00",
            __plan: hex"00",
            __placeholderParameters: placeholderParameters,
            __proof: hex"00",
            __tableLengths: tableLengths,
            __commitments: commitments
        });
    }

    function testVerify() public view {
        bytes[] memory tableCommitments = new bytes[](1);
        tableCommitments[0] =
            hex"00000000000000000000000000000006000000000000000216D6B82D96CFEC48A1C026598F99BBF3C07CA111E6EB2A9C4E2E9187827D065B1a5ac01ef230e94760b6ad505b9a00be520384bc89cb0e72260135c637679e721a8482d208bad3eeadc836daec1e6bba51ccbc58387b4457d2175934a8fa5bd81e6c1ee8c9a4d39fe271abfb51d3662d199fe6856ded86f727174dbe21ab63dc00000000000000020000000000000001620000000005000000050000000180000000000000007fffffffffffffff0000000000000001610000000005000000050000000180000000000000007fffffffffffffff";
        bytes memory queryPlan =
            hex"0000000000000001000000000000000f6e616d6573706163652e7461626c650000000000000002000000000000000000000000000000016200000005000000000000000000000000000000016100000005000000000000000100000000000000016200000000000000000000000000000002000000000000000000000001000000010000000500000000000000050000000000000001000000000000000000000000";

        Verifier.verify({
            __result: hex"00000000000000010000000000000001620000000005000000000000000200000000000000000000000000000003",
            __plan: queryPlan,
            __placeholderParameters: new uint256[](0),
            __proof: hex"000000000000000600000000000000020000000000000001000000000000000200000000000000000000000000000001236cd1dd5ae57dd08a8b09bd6bd3397fe4b58aa1638c554b8054dcb2d4d0297e12a7e3d2a6d11cf4e66979ea5458d24f66ba7866c43b933fd2135c2a761990140000000000000006000000000000000427a607595971e4f8531109d6bcf114838faac36652aa81d86d51f087cd08799c1095fa761ceaceb796068f4b8174492ede1e88c2f4aafee493331c267284be330078dfd5227feba93faa9a0c9c38432d6c169909d2546f8cef0b7a2108d44db71d518c19207b5ee92cca5cfe09ec5aca67af638b4205a7b650ccb9b9ecd6dd812bbc76361bfaf1e75d70512e9911c71b134404cecbce4053e29ec7c0067717941b11c59c7ba544571e02ba430f9256a63b51e7c8927ffd0d9763fc59bcdcad9521defadf9d440192285b6a0f4e7dc657225821e39bca44e618f7331859e431a62921dd8f70e56f4b52a667f4a851eef537584fb196d14cde47a607f2f6015f3c0000000000000000000000000000000c08a4646bbe5bcbd84f70d57e4199feea96727b0891cd7fc2b9c4548c79134fdc3056f9d01ed8ac370d2d91499df82105cc5842626884827bbd5da23bde9d9f2f27cd3ea9e52ec844140224a5237090c9ed9d1325f920dee410a1f45f884f10f700000000000000000000000000000000000000000000000000000000000000000cb3c1f4236d067d2121b1970bba959f6eb34a5dc55780360adf721cb7706d841e3a39b44aa419741fae45991ff853985acb44d6da117591f5c62bdcd34efa5e0cad3ebe8e7c85e28b999e7e98891b7f5de8dfeb641816693ff4c88ccd378bb10f76ac6934ab963d8cf2de36d5082457f1d4f37b399bab85a7a3e0748b98f4481f5fbcd1225d6fc2179d67784f8cc33f96af63fd2bc3bc0a0b24a2aef032f4e003ede74206e0c0224fbe9c670c28a6a8523bbb853df530b7e8d210d41fa1eeb20567b8ab1dd1ac6b76c90f55380a0e9f6ae397f9f2b29cdd9ed166243e6637591ac21cb2b3d1ed5ae89915fb2dfff88f8d86565b98cf106ed13c21abd2a0d86800000000000000012ab9aab80d43f7435784460a9a510611d5ea4cddb88ea32ccc3c7ca747b95c9d000000000000000216939642c7e9031cbb6e8dc903683c563fe29ffee7344b7c1705ecfb08c627b22f0a59c8d49c3b5d84f2e6e73cf44f2c05005ccf6f4b644e26399fa59e3eab2000000000000000041e837176dd46439a98b5e1aad2c2be57a9cec6c0a6e949e325681657fc59201824588a6ca09cd690acfac8a388cd7c2c2b2105cef96856365a2809a051aaaf231b7e9025b83e0435c1d7f5a8f42da0c4d0809a2b89df0792574778003419af3e2b59a06fd0744f42d07b57f7d8cda7a2032b86fc9eeaf80e4f858faa0e1da9ee00000000000000021cc3fcf38084a75d2d2e8f65edd2e7f1c98e27936ed4ec9c2f83d7d7f953d48d2276ff007d3bda222501c2f5b5ef10b7bf69d177de3ae102dc6e95507b7db4620d6089448b955000aec99031b580405143cd992004d92d0bb840748ad17cce2808947359cf839bc2792b71bb653862169475ccf7d354f0e05ff44b9600fb97500000000000000003120eef7fd4b3ee1684781b3828b14e5ce92a348993e090ef7aefed628428f04111c60d0bb82ab6223350ef46a99b6aaf72d9d10a090dc84c1b4dd1719f6d047829729324a31c3bcf393b4fa25704e2104cfd60e68e43f68892f8b69292efea7202e33bf988181994edcd6d7663b5590339d28e0509d8c0fc0d93334298e1295d12401b7ec5e8363c4ff0d04d7a20ff9584798bf571d5f5ea6a18ac08511742d229d4fa347951ed2d4812b378ac25dc7f278efea58239686edd309c65d1254b20238a63c8139c66b11f73e4490c47bd54517e9a3d84e7e1728bd2545c50070b8f2a5d10f0f8dc22249e979d95b4d5cbede923f576993884fc468d5a675bd7e4f02ae68ad8a170536483d1c64ad7763ee693cb2f963577bd6e530a275337a3480e2eefe01b65c73902b1bd918c0320fa626052596cdd8a43c1be1904df048ca02e16a1c960daf465f7578155147ee1ccf23aff18f620a299a539b3341a5bde26f80ef8f2d1a2febc7c56bf170a18a3215cdc2d2861002a582b7ff03c5c2ceceaf52f21e226bb2d43a1cfc1daee7dad66c1be678f2bb55ec1fc312916b58a240a980687fda634e438a343b885bc944199728f2161d6a244e06ac84a4cdbafdcb6d60e62f98d94384fa7bc869e213dce126be5d98e3b72241f7a615fa4f301d6af09",
            __tableCommitments: tableCommitments
        });
    }

    function testDeserializeTableCommitment() public pure {
        bytes[] memory tableCommitments = new bytes[](1);
        tableCommitments[0] =
            hex"00000000000000000000000000000003000000000000000a28b9628e4dd40477d8b9c22553ab1cef7de92be01aa635272a5153fdcc667a3c283fc7f7405e8f20c65e94abcfaddc18fb6d6d5fd6f24126a55fd428dc700b9028be5523f6dbeba579ba197a7c1f4f03f46da1dae8ea2e49527194195818f6680238e6a4be9f9fc58f0fe9a6d2ac32e2f357de9f33867506bcc8867695fcd8522699c20d6b46ccff9aea7842e64fccb91f59bd19b41ae4e4d29dee4804838ae0128ee7a84902f933505672b38340d2cdabeeca5c0c76e5548398ed95cdc8384c2fc17fec090794eae15adf182c461cad4867eab5807d4f38c86952472960e1d60ad5cb46df983be9f745caa334c85de105e9718c88205188939729b1794ea8de257aad035de6dc285d1be01072947fb3bbff4b71457613d266dbde54710e52dd0e5bab33d51316dd8f77fcd732e259fb1599aaefd7d55c59c63fe81bdf030ca2257aad035de6dc285d1be01072947fb3bbff4b71457613d266dbde54710e52dd0e5bab33d51316dd8f77fcd732e259fb1599aaefd7d55c59c63fe81bdf030ca2257aad035de6dc285d1be01072947fb3bbff4b71457613d266dbde54710e52dd0e5bab33d51316dd8f77fcd732e259fb1599aaefd7d55c59c63fe81bdf030ca216650968b1ec0b8cd847a81234cb55453682b378aba2b9ee1124232fab5f2e082ad3e1cb2fc5c73829abfe1aca3cbc9787c9bdccc3f698d4acd55b7f6d70ed07257aad035de6dc285d1be01072947fb3bbff4b71457613d266dbde54710e52dd0e5bab33d51316dd8f77fcd732e259fb1599aaefd7d55c59c63fe81bdf030ca2257aad035de6dc285d1be01072947fb3bbff4b71457613d266dbde54710e52dd0e5bab33d51316dd8f77fcd732e259fb1599aaefd7d55c59c63fe81bdf030ca2000000000000000a00000000000000016100000000000000000000000000000000016200000000020000000200000002fe0200000000000000016300000000030000000300000002fc18000e00000000000000016400000000040000000400000002fffffffe000f42400000000000000001650000000005000000050000000200000000000000010000000000000003000000000000000166000000000700000000000000000000000167000000000808010000000000000000000000016800000000090000000100000000000000070000000200000000000000010000000000000005000000000000000169000000000a0000000000000000000000016a000000000b00000000";
        Verifier.TableCommitment memory deserializedTableCommitment =
            VerifierTestWrapper.deserializeTableCommitments(tableCommitments)[0];
        assert(deserializedTableCommitment.tableLength == 3);
        bytes32[] memory expectedColumnHashes = new bytes32[](10);
        expectedColumnHashes[0] = keccak256(bytes("a")); // Boolean
        expectedColumnHashes[1] = keccak256(bytes("b")); // TinyInt
        expectedColumnHashes[2] = keccak256(bytes("c")); // SmallInt
        expectedColumnHashes[3] = keccak256(bytes("d")); // Int
        expectedColumnHashes[4] = keccak256(bytes("e")); // BigInt
        expectedColumnHashes[5] = keccak256(bytes("f")); // VarChar
        expectedColumnHashes[6] = keccak256(bytes("g")); // Decimal75
        expectedColumnHashes[7] = keccak256(bytes("h")); // TimestampTZ
        expectedColumnHashes[8] = keccak256(bytes("i")); // Scalar
        expectedColumnHashes[9] = keccak256(bytes("j")); // VarBinary
        assert(deserializedTableCommitment.columnNameHashes[0] == expectedColumnHashes[0]);
        assert(deserializedTableCommitment.columnNameHashes[1] == expectedColumnHashes[1]);
        assert(deserializedTableCommitment.columnNameHashes[2] == expectedColumnHashes[2]);
        assert(deserializedTableCommitment.columnNameHashes[3] == expectedColumnHashes[3]);
        assert(deserializedTableCommitment.columnNameHashes[4] == expectedColumnHashes[4]);
        assert(deserializedTableCommitment.columnNameHashes[5] == expectedColumnHashes[5]);
        assert(deserializedTableCommitment.columnNameHashes[6] == expectedColumnHashes[6]);
        assert(deserializedTableCommitment.columnNameHashes[7] == expectedColumnHashes[7]);
        assert(deserializedTableCommitment.columnNameHashes[8] == expectedColumnHashes[8]);
        assert(deserializedTableCommitment.columnNameHashes[9] == expectedColumnHashes[9]);
    }

    function testDeserializeTableCommitmentWithUnsupportedType() public {
        vm.expectRevert(Errors.TableCommitmentUnsupported.selector);
        VerifierTestWrapper.deserializeTableCommitment(
            hex"0000000000000000000000000000000300000000000000011a21982f135adddb079a91b739a8d358e47f38478eeaf8844c2f670f6595895e276a4c8714fba9d532e03556294ffdb2c6fb376a11ec2dda177a056e9513a28b0000000000000001000000000000000162000000000700000008"
        );

        vm.expectRevert(Errors.TableCommitmentUnsupported.selector);
        VerifierTestWrapper.deserializeTableCommitment(hex"0000000000000001");

        vm.expectRevert(Errors.TableCommitmentUnsupported.selector);
        VerifierTestWrapper.deserializeTableCommitment(
            hex"0000000000000000000000000000000000000000000000000000000000000001"
        );

        vm.expectRevert(Errors.TableCommitmentUnsupported.selector);
        VerifierTestWrapper.deserializeTableCommitment(
            hex"00000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000010001"
        );
    }

    function testPlan() public pure {
        bytes memory plan =
            hex"0000000000000001000000000000000f6e616d6573706163652e7461626c6500000000000000030000000000000000000000000000000162000000080a000000000000000000000000000000000163000000090000000100000000000000000000000000000000000000016100000005000000000000000200000000000000016200000000000000016300000000000000000000000000000002000000000000000000000002000000010000000500000000000000050000000000000002000000000000000000000000000000000000000000000001";
        VerifierTestWrapper.deserializeProofPlanPrefix(plan);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testNotEnoughTableCommitments() public {
        uint64[] memory indices = new uint64[](1);
        indices[0] = 0;
        bytes32[] memory hashes = new bytes32[](1);
        hashes[0] = 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef;
        vm.expectRevert(Errors.CommitmentsNotFound.selector);
        Verifier.getRelevantCommitments(indices, hashes, new Verifier.TableCommitment[](0));
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testColumnHashesNotMatching() public {
        uint64[] memory indices = new uint64[](1);
        indices[0] = 0;
        bytes32[] memory hashes = new bytes32[](1);
        hashes[0] = 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef;
        Verifier.TableCommitment[] memory commitments = new Verifier.TableCommitment[](1);
        Verifier.TableCommitment memory first;
        bytes32[] memory firstHashes = new bytes32[](1);
        firstHashes[0] = 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdee;
        first.columnNameHashes = firstHashes;
        first.tableLength = 1;
        first.commitmentsPtr = 1;
        commitments[0] = first;
        vm.expectRevert(Errors.CommitmentsNotFound.selector);
        Verifier.getRelevantCommitments(indices, hashes, commitments);
    }
}

library VerifierTestWrapper {
    function deserializeProofPlanPrefix(bytes calldata plan)
        external
        pure
        returns (
            // tableNameHashes[tableId] = tableNameHash
            bytes32[] memory tableNameHashes,
            // columnTableIndexes[columnId] = tableId
            uint64[] memory columnTableIndexes,
            // columnNameHashes[columnId] = columnNameHash
            bytes32[] memory columnNameHashes
        )
    {
        (tableNameHashes, columnTableIndexes, columnNameHashes) = Verifier.deserializeProofPlanPrefix(plan);
    }

    function deserializeTableCommitment(bytes calldata tableCommitment)
        external
        pure
        returns (Verifier.TableCommitment memory result)
    {
        result = Verifier.deserializeTableCommitment(tableCommitment);
    }

    function deserializeTableCommitments(bytes[] calldata tableCommitments)
        external
        pure
        returns (
            // tableCommitments[tableId] = TableCommitment
            Verifier.TableCommitment[] memory result
        )
    {
        result = Verifier.deserializeTableCommitments(tableCommitments);
    }
}
