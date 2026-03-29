#[cfg(test)]
mod tests {
    use crate::base::commitment::{
        naive_commitment::NaiveCommitment,
        VecCommitmentExt,
    };
    use crate::base::scalar::Curve25519Scalar;

    // -----------------------------------------------------------------------
    // from_columns_with_offset
    // -----------------------------------------------------------------------

    #[test]
    fn test_from_columns_with_offset_empty_produces_empty() {
        let no_columns: &[&[i64]] = &[];
        let commitments =
            Vec::<NaiveCommitment>::from_columns_with_offset(no_columns.iter().copied(), 0, &());
        assert!(commitments.is_empty());
    }

    #[test]
    fn test_from_columns_with_offset_single_column() {
        let data: &[Curve25519Scalar] = &[
            Curve25519Scalar::from(1_i64),
            Curve25519Scalar::from(2_i64),
            Curve25519Scalar::from(3_i64),
        ];
        let commitments =
            Vec::<NaiveCommitment>::from_columns_with_offset([data].iter().copied(), 0, &());
        assert_eq!(commitments.len(), 1);
    }

    #[test]
    fn test_from_columns_with_offset_multiple_columns() {
        let col_a: &[Curve25519Scalar] = &[
            Curve25519Scalar::from(10_i64),
            Curve25519Scalar::from(20_i64),
        ];
        let col_b: &[Curve25519Scalar] = &[
            Curve25519Scalar::from(30_i64),
            Curve25519Scalar::from(40_i64),
        ];
        let commitments = Vec::<NaiveCommitment>::from_columns_with_offset(
            [col_a, col_b].iter().copied(),
            0,
            &(),
        );
        assert_eq!(commitments.len(), 2);
    }

    // -----------------------------------------------------------------------
    // extend_commitments
    // -----------------------------------------------------------------------

    #[test]
    fn test_extend_commitments_appends_correctly() {
        let col_a: &[Curve25519Scalar] = &[Curve25519Scalar::from(1_i64)];
        let col_b: &[Curve25519Scalar] = &[Curve25519Scalar::from(2_i64)];

        let mut commitments =
            Vec::<NaiveCommitment>::from_columns_with_offset([col_a].iter().copied(), 0, &());
        assert_eq!(commitments.len(), 1);

        commitments.extend_commitments([col_b].iter().copied(), 0, &());
        assert_eq!(commitments.len(), 2);
    }

    // -----------------------------------------------------------------------
    // Determinism: same data => same commitment
    // -----------------------------------------------------------------------

    #[test]
    fn test_commitment_is_deterministic() {
        let data: &[Curve25519Scalar] = &[
            Curve25519Scalar::from(5_i64),
            Curve25519Scalar::from(6_i64),
        ];
        let c1 =
            Vec::<NaiveCommitment>::from_columns_with_offset([data].iter().copied(), 0, &());
        let c2 =
            Vec::<NaiveCommitment>::from_columns_with_offset([data].iter().copied(), 0, &());
        assert_eq!(c1, c2);
    }
}
