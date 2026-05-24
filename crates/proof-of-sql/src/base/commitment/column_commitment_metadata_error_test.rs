use crate::base::commitment::{InvalidColumnCommitmentMetadata, ColumnCommitmentMetadataMismatch};
use crate::base::database::{ColumnType, ColumnBounds, Bounds};
use crate::base::commitment::Bounds::Bounded;

#[test]
fn invalid_column_commitment_metadata_display() {
    let err = InvalidColumnCommitmentMetadata::TypeBoundsMismatch {
        column_type: ColumnType::Boolean,
        column_bounds: ColumnBounds::Sharp(crate::base::commitment::BoundsInner {
            min: crate::base::scalar::Scalar::zero().into(),
            max: crate::base::scalar::Scalar::one().into(),
        }),
    };
    assert!(format!("{}").contains("cannot have bounds"));
}

#[test]
fn invalid_column_commitment_metadata_debug() {
    use crate::base::commitment::BoundsInner;
    use crate::base::scalar::Scalar;
    let err = InvalidColumnCommitmentMetadata::TypeBoundsMismatch {
        column_type: ColumnType::Int,
        column_bounds: ColumnBounds::Sharp(BoundsInner {
            min: Scalar::zero().into(),
            max: Scalar::one().into(),
        }),
    };
    assert!(format!("{:?}").contains("TypeBoundsMismatch"));
}

#[test]
fn column_commitment_metadata_mismatch_display() {
    let err = ColumnCommitmentMetadataMismatch {
        datatype_a: ColumnType::Int,
        datatype_b: ColumnType::Boolean,
    };
    assert_eq!(format!("{}"), "column with type Int cannot operate with column with type Boolean");
}

#[test]
fn column_commitment_metadata_mismatch_debug() {
    let err = ColumnCommitmentMetadataMismatch {
        datatype_a: ColumnType::BigInt,
        datatype_b: ColumnType::VarChar,
    };
    assert!(format!("{:?}").contains("ColumnCommitmentMetadataMismatch"));
}
