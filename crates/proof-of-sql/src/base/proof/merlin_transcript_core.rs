impl super::transcript_core::TranscriptCore for merlin::Transcript {
    fn new() -> Self {
        merlin::Transcript::new(b"TranscriptCore::new")
    }
    fn raw_append(&mut self, message: &[u8]) {
        self.append_message(b"TranscriptCore::raw_append", message);
    }
    fn raw_challenge(&mut self) -> [u8; 32] {
        let mut result = [0u8; 32];
        self.challenge_bytes(b"TranscriptCore::raw_challenge", &mut result);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        transcript_core::{test_util::*, TranscriptCore},
        Transcript,
    };
    #[test]
    fn we_get_equivalent_challenges_with_equivalent_merlin_transcripts() {
        we_get_equivalent_challenges_with_equivalent_transcripts::<merlin::Transcript>();
    }
    #[test]
    fn we_get_different_challenges_with_different_keccak256_transcripts() {
        we_get_different_challenges_with_different_transcripts::<merlin::Transcript>();
    }
    #[test]
    fn we_get_different_nontrivial_consecutive_challenges_from_keccak256_transcript() {
        we_get_different_nontrivial_consecutive_challenges_from_transcript::<merlin::Transcript>();
    }
    #[test]
    fn we_can_add_values_to_merlin_transcript_in_big_endian_form() {
        let mut transcript1 = <merlin::Transcript as Transcript>::new();
        transcript1.extend_as_be([1_u16, 1000, 2]);

        let mut transcript2: merlin::Transcript = TranscriptCore::new();
        transcript2.raw_append(&[0, 1]);
        transcript2.raw_append(&[3, 232]);
        transcript2.raw_append(&[0, 2]);

        assert_eq!(transcript1.raw_challenge(), transcript2.raw_challenge());
    }
    #[test]
    fn we_can_add_refs_to_merlin_transcript_in_little_endian_form() {
        let values = [1_u16, 1000, 2];
        let mut transcript1 = <merlin::Transcript as Transcript>::new();
        transcript1.extend_as_le_from_refs(&values);

        let mut transcript2: merlin::Transcript = TranscriptCore::new();
        transcript2.raw_append(&[1, 0]);
        transcript2.raw_append(&[232, 3]);
        transcript2.raw_append(&[2, 0]);

        assert_eq!(transcript1.raw_challenge(), transcript2.raw_challenge());
    }
}
