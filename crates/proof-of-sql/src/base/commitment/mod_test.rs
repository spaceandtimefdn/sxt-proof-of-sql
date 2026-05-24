#[cfg(test)]
mod commitment_mod_test {
    #[test]
    fn test_module_re_exports() {
        use crate::base::commitment::{
            Bounds, ColumnBounds, ColumnCommitmentMetadata, ColumnCommitments,
            CommittableColumn, Commitment, CommitmentEvaluationProof, HyperKZGCommitment,
            NegativeBounds, NumColumnsMismatch, QueryCommitments, TableCommitment,
            VecCommitmentExt,
        };
        // Just verify the types exist
        fn _check<T: std::fmt::Debug>() {}
        _check::<Bounds>();
        _check::<ColumnBounds>();
        _check::<NegativeBounds>();
    }
}
