use super::{test_rng, DoryMessages, G1Affine, G2Affine, F, GT};
use alloc::vec::Vec;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, Compress, Validate};
use ark_std::UniformRand;
use merlin::Transcript;

#[test]
fn we_can_send_and_receive_the_correct_messages_in_the_same_order() {
    let mut rng = test_rng();
    let mut messages = DoryMessages::default();

    // Prover side
    let mut transcript = Transcript::new(b"test");
    let Pmessage1 = F::rand(&mut rng);
    let Pmessage2 = G1Affine::rand(&mut rng);
    let Pmessage3 = G2Affine::rand(&mut rng);
    let Pmessage4 = GT::rand(&mut rng);
    let Pmessage5 = F::rand(&mut rng);
    let Pmessage6 = G1Affine::rand(&mut rng);
    let Pmessage7 = G2Affine::rand(&mut rng);
    let Pmessage8 = G1Affine::rand(&mut rng);
    let Pmessage9 = F::rand(&mut rng);
    messages.prover_send_F_message(&mut transcript, Pmessage1);
    let Vmessage1 = messages.verifier_F_message(&mut transcript);
    messages.prover_send_G1_message(&mut transcript, Pmessage2);
    messages.prover_send_G2_message(&mut transcript, Pmessage3);
    messages.prover_send_GT_message(&mut transcript, Pmessage4);
    let Vmessage2 = messages.verifier_F_message(&mut transcript);
    messages.prover_send_F_message(&mut transcript, Pmessage5);
    messages.prover_send_G1_message(&mut transcript, Pmessage6);
    messages.prover_send_G2_message(&mut transcript, Pmessage7);
    messages.prover_send_G1_message(&mut transcript, Pmessage8);
    let Vmessage3 = messages.verifier_F_message(&mut transcript);
    messages.prover_send_F_message(&mut transcript, Pmessage9);

    // Verifier side
    let mut transcript = Transcript::new(b"test");
    assert_eq!(
        messages.prover_receive_F_message(&mut transcript),
        Pmessage1
    );
    assert_eq!(messages.verifier_F_message(&mut transcript), Vmessage1);
    assert_eq!(
        messages.prover_receive_G1_message(&mut transcript),
        Pmessage2
    );
    assert_eq!(
        messages.prover_receive_G2_message(&mut transcript),
        Pmessage3
    );
    assert_eq!(
        messages.prover_receive_GT_message(&mut transcript),
        Pmessage4
    );
    assert_eq!(messages.verifier_F_message(&mut transcript), Vmessage2);
    assert_eq!(
        messages.prover_receive_F_message(&mut transcript),
        Pmessage5
    );
    assert_eq!(
        messages.prover_receive_G1_message(&mut transcript),
        Pmessage6
    );
    assert_eq!(
        messages.prover_receive_G2_message(&mut transcript),
        Pmessage7
    );
    assert_eq!(
        messages.prover_receive_G1_message(&mut transcript),
        Pmessage8
    );
    assert_eq!(messages.verifier_F_message(&mut transcript), Vmessage3);
    assert_eq!(
        messages.prover_receive_F_message(&mut transcript),
        Pmessage9
    );
}

#[test]
fn verifier_messages_fail_when_the_transcript_is_wrong() {
    let mut rng = test_rng();
    let mut messages = DoryMessages::default();

    // Prover side
    let mut transcript = Transcript::new(b"test");
    let Pmessage1 = F::rand(&mut rng);
    let Pmessage2 = G1Affine::rand(&mut rng);
    let Pmessage3 = G2Affine::rand(&mut rng);
    let Pmessage4 = GT::rand(&mut rng);
    let Pmessage5 = F::rand(&mut rng);
    let Pmessage6 = G1Affine::rand(&mut rng);
    let Pmessage7 = G2Affine::rand(&mut rng);
    let Pmessage8 = G1Affine::rand(&mut rng);
    let Pmessage9 = F::rand(&mut rng);
    messages.prover_send_F_message(&mut transcript, Pmessage1);
    let Vmessage1 = messages.verifier_F_message(&mut transcript);
    messages.prover_send_G1_message(&mut transcript, Pmessage2);
    messages.prover_send_G2_message(&mut transcript, Pmessage3);
    messages.prover_send_GT_message(&mut transcript, Pmessage4);
    let Vmessage2 = messages.verifier_F_message(&mut transcript);
    messages.prover_send_F_message(&mut transcript, Pmessage5);
    messages.prover_send_G1_message(&mut transcript, Pmessage6);
    messages.prover_send_G2_message(&mut transcript, Pmessage7);
    messages.prover_send_G1_message(&mut transcript, Pmessage8);
    let Vmessage3 = messages.verifier_F_message(&mut transcript);
    messages.prover_send_F_message(&mut transcript, Pmessage9);

    // Verifier side
    let mut transcript = Transcript::new(b"test_wrong");
    assert_eq!(
        messages.prover_receive_F_message(&mut transcript),
        Pmessage1
    );
    assert_ne!(messages.verifier_F_message(&mut transcript), Vmessage1);
    assert_eq!(
        messages.prover_receive_G1_message(&mut transcript),
        Pmessage2
    );
    assert_eq!(
        messages.prover_receive_G2_message(&mut transcript),
        Pmessage3
    );
    assert_eq!(
        messages.prover_receive_GT_message(&mut transcript),
        Pmessage4
    );
    assert_ne!(messages.verifier_F_message(&mut transcript), Vmessage2);
    assert_eq!(
        messages.prover_receive_F_message(&mut transcript),
        Pmessage5
    );
    assert_eq!(
        messages.prover_receive_G1_message(&mut transcript),
        Pmessage6
    );
    assert_eq!(
        messages.prover_receive_G2_message(&mut transcript),
        Pmessage7
    );
    assert_eq!(
        messages.prover_receive_G1_message(&mut transcript),
        Pmessage8
    );
    assert_ne!(messages.verifier_F_message(&mut transcript), Vmessage3);
    assert_eq!(
        messages.prover_receive_F_message(&mut transcript),
        Pmessage9
    );
}

#[test]
fn verifier_messages_fail_when_a_verifier_message_is_in_the_wrong_order() {
    let mut rng = test_rng();
    let mut messages = DoryMessages::default();

    // Prover side
    let mut transcript = Transcript::new(b"test");
    let Pmessage1 = F::rand(&mut rng);
    let Pmessage2 = G1Affine::rand(&mut rng);
    let Pmessage3 = G2Affine::rand(&mut rng);
    let Pmessage4 = GT::rand(&mut rng);
    let Pmessage5 = F::rand(&mut rng);
    let Pmessage6 = G1Affine::rand(&mut rng);
    let Pmessage7 = G2Affine::rand(&mut rng);
    let Pmessage8 = G1Affine::rand(&mut rng);
    let Pmessage9 = F::rand(&mut rng);
    messages.prover_send_F_message(&mut transcript, Pmessage1);
    let Vmessage1 = messages.verifier_F_message(&mut transcript);
    messages.prover_send_G1_message(&mut transcript, Pmessage2);
    messages.prover_send_G2_message(&mut transcript, Pmessage3);
    messages.prover_send_GT_message(&mut transcript, Pmessage4);
    let Vmessage2 = messages.verifier_F_message(&mut transcript);
    messages.prover_send_F_message(&mut transcript, Pmessage5);
    messages.prover_send_G1_message(&mut transcript, Pmessage6);
    messages.prover_send_G2_message(&mut transcript, Pmessage7);
    messages.prover_send_G1_message(&mut transcript, Pmessage8);
    let Vmessage3 = messages.verifier_F_message(&mut transcript);
    messages.prover_send_F_message(&mut transcript, Pmessage9);

    // Verifier side
    let mut transcript = Transcript::new(b"test");
    assert_eq!(
        messages.prover_receive_F_message(&mut transcript),
        Pmessage1
    );
    assert_eq!(messages.verifier_F_message(&mut transcript), Vmessage1);
    assert_eq!(
        messages.prover_receive_G1_message(&mut transcript),
        Pmessage2
    );
    assert_eq!(
        messages.prover_receive_G2_message(&mut transcript),
        Pmessage3
    );
    assert_eq!(
        messages.prover_receive_GT_message(&mut transcript),
        Pmessage4
    );
    assert_eq!(
        messages.prover_receive_F_message(&mut transcript),
        Pmessage5
    );
    assert_ne!(messages.verifier_F_message(&mut transcript), Vmessage2);
    assert_eq!(
        messages.prover_receive_G1_message(&mut transcript),
        Pmessage6
    );
    assert_eq!(
        messages.prover_receive_G2_message(&mut transcript),
        Pmessage7
    );
    assert_eq!(
        messages.prover_receive_G1_message(&mut transcript),
        Pmessage8
    );
    assert_ne!(messages.verifier_F_message(&mut transcript), Vmessage3);
    assert_eq!(
        messages.prover_receive_F_message(&mut transcript),
        Pmessage9
    );
}

#[test]
fn verifier_messages_fail_when_prover_messages_are_out_of_order() {
    let mut rng = test_rng();
    let mut messages = DoryMessages::default();

    // Prover side
    let mut transcript = Transcript::new(b"test");
    let Pmessage1 = F::rand(&mut rng);
    let Pmessage2 = G1Affine::rand(&mut rng);
    let Pmessage3 = G2Affine::rand(&mut rng);
    let Pmessage4 = GT::rand(&mut rng);
    let Pmessage5 = F::rand(&mut rng);
    let Pmessage6 = G1Affine::rand(&mut rng);
    let Pmessage7 = G2Affine::rand(&mut rng);
    let Pmessage8 = G1Affine::rand(&mut rng);
    let Pmessage9 = F::rand(&mut rng);
    messages.prover_send_F_message(&mut transcript, Pmessage1);
    let Vmessage1 = messages.verifier_F_message(&mut transcript);
    messages.prover_send_G1_message(&mut transcript, Pmessage2);
    messages.prover_send_G2_message(&mut transcript, Pmessage3);
    messages.prover_send_GT_message(&mut transcript, Pmessage4);
    let Vmessage2 = messages.verifier_F_message(&mut transcript);
    messages.prover_send_F_message(&mut transcript, Pmessage5);
    messages.prover_send_G1_message(&mut transcript, Pmessage6);
    messages.prover_send_G2_message(&mut transcript, Pmessage7);
    messages.prover_send_G1_message(&mut transcript, Pmessage8);
    let Vmessage3 = messages.verifier_F_message(&mut transcript);
    messages.prover_send_F_message(&mut transcript, Pmessage9);

    // Verifier side
    let mut transcript = Transcript::new(b"test");
    assert_eq!(
        messages.prover_receive_F_message(&mut transcript),
        Pmessage1
    );
    assert_eq!(messages.verifier_F_message(&mut transcript), Vmessage1);
    assert_eq!(
        messages.prover_receive_G1_message(&mut transcript),
        Pmessage2
    );
    assert_eq!(
        messages.prover_receive_GT_message(&mut transcript),
        Pmessage4
    );
    assert_eq!(
        messages.prover_receive_G2_message(&mut transcript),
        Pmessage3
    );
    assert_ne!(messages.verifier_F_message(&mut transcript), Vmessage2);
    assert_eq!(
        messages.prover_receive_F_message(&mut transcript),
        Pmessage5
    );
    assert_eq!(
        messages.prover_receive_G1_message(&mut transcript),
        Pmessage6
    );
    assert_eq!(
        messages.prover_receive_G2_message(&mut transcript),
        Pmessage7
    );
    assert_eq!(
        messages.prover_receive_G1_message(&mut transcript),
        Pmessage8
    );
    assert_ne!(messages.verifier_F_message(&mut transcript), Vmessage3);
    assert_eq!(
        messages.prover_receive_F_message(&mut transcript),
        Pmessage9
    );
}

#[test]
fn dory_messages_round_trip_through_canonical_serialization() {
    let mut rng = test_rng();
    let mut messages = DoryMessages::default();
    let mut transcript = Transcript::new(b"serialization");

    messages.prover_send_F_message(&mut transcript, F::rand(&mut rng));
    messages.prover_send_G1_message(&mut transcript, G1Affine::rand(&mut rng));
    messages.prover_send_G2_message(&mut transcript, G2Affine::rand(&mut rng));
    messages.prover_send_GT_message(&mut transcript, GT::rand(&mut rng));

    let mut bytes = Vec::new();
    messages
        .serialize_with_mode(&mut bytes, Compress::No)
        .expect("serialize DoryMessages");
    let decoded = DoryMessages::deserialize_with_mode(&mut &bytes[..], Compress::No, Validate::Yes)
        .expect("deserialize DoryMessages");

    assert_eq!(decoded, messages);
}
