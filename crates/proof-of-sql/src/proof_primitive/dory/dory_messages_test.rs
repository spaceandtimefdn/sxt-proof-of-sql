use super::{test_rng, DoryMessages, G1Affine, G2Affine, F, GT};
use ark_std::UniformRand;
use merlin::Transcript;
use std::sync::OnceLock;

#[derive(Clone)]
struct ProverMessages {
    f1: F,
    g1_1: G1Affine,
    g2_1: G2Affine,
    gt: GT,
    f2: F,
    g1_2: G1Affine,
    g2_2: G2Affine,
    g1_3: G1Affine,
    f3: F,
}

#[derive(Clone)]
struct MessageFixture {
    messages: DoryMessages,
    prover: ProverMessages,
    v1: (F, F),
    v2: (F, F),
    v3: (F, F),
}

fn prover_messages() -> ProverMessages {
    static MESSAGES: OnceLock<ProverMessages> = OnceLock::new();
    MESSAGES
        .get_or_init(|| {
            // test_rng is deterministic; cache these immutable randomized inputs because
            // G1/G2/GT generation dominates this small message-order test module.
            let mut rng = test_rng();
            ProverMessages {
                f1: F::rand(&mut rng),
                g1_1: G1Affine::rand(&mut rng),
                g2_1: G2Affine::rand(&mut rng),
                gt: GT::rand(&mut rng),
                f2: F::rand(&mut rng),
                g1_2: G1Affine::rand(&mut rng),
                g2_2: G2Affine::rand(&mut rng),
                g1_3: G1Affine::rand(&mut rng),
                f3: F::rand(&mut rng),
            }
        })
        .clone()
}

fn message_fixture() -> MessageFixture {
    let prover = prover_messages();
    let mut messages = DoryMessages::default();
    let mut transcript = Transcript::new(b"test");
    messages.prover_send_F_message(&mut transcript, prover.f1.clone());
    let v1 = messages.verifier_F_message(&mut transcript);
    messages.prover_send_G1_message(&mut transcript, prover.g1_1.clone());
    messages.prover_send_G2_message(&mut transcript, prover.g2_1.clone());
    messages.prover_send_GT_message(&mut transcript, prover.gt.clone());
    let v2 = messages.verifier_F_message(&mut transcript);
    messages.prover_send_F_message(&mut transcript, prover.f2.clone());
    messages.prover_send_G1_message(&mut transcript, prover.g1_2.clone());
    messages.prover_send_G2_message(&mut transcript, prover.g2_2.clone());
    messages.prover_send_G1_message(&mut transcript, prover.g1_3.clone());
    let v3 = messages.verifier_F_message(&mut transcript);
    messages.prover_send_F_message(&mut transcript, prover.f3.clone());

    MessageFixture {
        messages,
        prover,
        v1,
        v2,
        v3,
    }
}

#[test]
fn we_can_send_and_receive_the_correct_messages_in_the_same_order() {
    let MessageFixture {
        mut messages,
        prover,
        v1,
        v2,
        v3,
    } = message_fixture();

    // Verifier side
    let mut transcript = Transcript::new(b"test");
    assert_eq!(
        messages.prover_receive_F_message(&mut transcript),
        prover.f1
    );
    assert_eq!(messages.verifier_F_message(&mut transcript), v1);
    assert_eq!(
        messages.prover_receive_G1_message(&mut transcript),
        prover.g1_1
    );
    assert_eq!(
        messages.prover_receive_G2_message(&mut transcript),
        prover.g2_1
    );
    assert_eq!(
        messages.prover_receive_GT_message(&mut transcript),
        prover.gt
    );
    assert_eq!(messages.verifier_F_message(&mut transcript), v2);
    assert_eq!(
        messages.prover_receive_F_message(&mut transcript),
        prover.f2
    );
    assert_eq!(
        messages.prover_receive_G1_message(&mut transcript),
        prover.g1_2
    );
    assert_eq!(
        messages.prover_receive_G2_message(&mut transcript),
        prover.g2_2
    );
    assert_eq!(
        messages.prover_receive_G1_message(&mut transcript),
        prover.g1_3
    );
    assert_eq!(messages.verifier_F_message(&mut transcript), v3);
    assert_eq!(
        messages.prover_receive_F_message(&mut transcript),
        prover.f3
    );
}

#[test]
fn verifier_messages_fail_when_the_transcript_is_wrong() {
    let MessageFixture {
        mut messages,
        prover,
        v1,
        v2,
        v3,
    } = message_fixture();

    // Verifier side
    let mut transcript = Transcript::new(b"test_wrong");
    assert_eq!(
        messages.prover_receive_F_message(&mut transcript),
        prover.f1
    );
    assert_ne!(messages.verifier_F_message(&mut transcript), v1);
    assert_eq!(
        messages.prover_receive_G1_message(&mut transcript),
        prover.g1_1
    );
    assert_eq!(
        messages.prover_receive_G2_message(&mut transcript),
        prover.g2_1
    );
    assert_eq!(
        messages.prover_receive_GT_message(&mut transcript),
        prover.gt
    );
    assert_ne!(messages.verifier_F_message(&mut transcript), v2);
    assert_eq!(
        messages.prover_receive_F_message(&mut transcript),
        prover.f2
    );
    assert_eq!(
        messages.prover_receive_G1_message(&mut transcript),
        prover.g1_2
    );
    assert_eq!(
        messages.prover_receive_G2_message(&mut transcript),
        prover.g2_2
    );
    assert_eq!(
        messages.prover_receive_G1_message(&mut transcript),
        prover.g1_3
    );
    assert_ne!(messages.verifier_F_message(&mut transcript), v3);
    assert_eq!(
        messages.prover_receive_F_message(&mut transcript),
        prover.f3
    );
}

#[test]
fn verifier_messages_fail_when_a_verifier_message_is_in_the_wrong_order() {
    let MessageFixture {
        mut messages,
        prover,
        v1,
        v2,
        v3,
    } = message_fixture();

    // Verifier side
    let mut transcript = Transcript::new(b"test");
    assert_eq!(
        messages.prover_receive_F_message(&mut transcript),
        prover.f1
    );
    assert_eq!(messages.verifier_F_message(&mut transcript), v1);
    assert_eq!(
        messages.prover_receive_G1_message(&mut transcript),
        prover.g1_1
    );
    assert_eq!(
        messages.prover_receive_G2_message(&mut transcript),
        prover.g2_1
    );
    assert_eq!(
        messages.prover_receive_GT_message(&mut transcript),
        prover.gt
    );
    assert_eq!(
        messages.prover_receive_F_message(&mut transcript),
        prover.f2
    );
    assert_ne!(messages.verifier_F_message(&mut transcript), v2);
    assert_eq!(
        messages.prover_receive_G1_message(&mut transcript),
        prover.g1_2
    );
    assert_eq!(
        messages.prover_receive_G2_message(&mut transcript),
        prover.g2_2
    );
    assert_eq!(
        messages.prover_receive_G1_message(&mut transcript),
        prover.g1_3
    );
    assert_ne!(messages.verifier_F_message(&mut transcript), v3);
    assert_eq!(
        messages.prover_receive_F_message(&mut transcript),
        prover.f3
    );
}

#[test]
fn verifier_messages_fail_when_prover_messages_are_out_of_order() {
    let MessageFixture {
        mut messages,
        prover,
        v1,
        v2,
        v3,
    } = message_fixture();

    // Verifier side
    let mut transcript = Transcript::new(b"test");
    assert_eq!(
        messages.prover_receive_F_message(&mut transcript),
        prover.f1
    );
    assert_eq!(messages.verifier_F_message(&mut transcript), v1);
    assert_eq!(
        messages.prover_receive_G1_message(&mut transcript),
        prover.g1_1
    );
    assert_eq!(
        messages.prover_receive_GT_message(&mut transcript),
        prover.gt
    );
    assert_eq!(
        messages.prover_receive_G2_message(&mut transcript),
        prover.g2_1
    );
    assert_ne!(messages.verifier_F_message(&mut transcript), v2);
    assert_eq!(
        messages.prover_receive_F_message(&mut transcript),
        prover.f2
    );
    assert_eq!(
        messages.prover_receive_G1_message(&mut transcript),
        prover.g1_2
    );
    assert_eq!(
        messages.prover_receive_G2_message(&mut transcript),
        prover.g2_2
    );
    assert_eq!(
        messages.prover_receive_G1_message(&mut transcript),
        prover.g1_3
    );
    assert_ne!(messages.verifier_F_message(&mut transcript), v3);
    assert_eq!(
        messages.prover_receive_F_message(&mut transcript),
        prover.f3
    );
}
