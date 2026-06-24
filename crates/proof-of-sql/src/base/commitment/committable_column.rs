use crate::base::{
    database::{Column, ColumnType, OwnedColumn},
    if_rayon,
    math::decimal::Precision,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    ref_into::RefInto,
    scalar::{Scalar, ScalarExt},
};
use alloc::vec::Vec;
#[cfg(feature = "blitzar")]
use blitzar::sequence::Sequence;
#[cfg(feature = "rayon")]
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

/// Column data in "committable form".
///
/// For some column types, transformations need to be applied before commitments are created.
/// These transformations require allocating new memory.
/// This is a problem since blitzar only borrows slices of data to commit to.
/// Normal column types don't store their data in "committable" form, so they cannot interface with
/// blitzar directly.
///
/// This type acts as an intermediate column type that *can* be used with blitzar directly.
/// For column types that need to be transformed, their "committable form" is owned here.
/// For column types that don't need to allocate new memory, their data is only borrowed here.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommittableColumn<'a> {
    /// Borrowed Bool column, mapped to `bool`.
    Boolean(&'a [bool]),
    /// Borrowed `Byte` column, mapped to `u8`.
    Uint8(&'a [u8]),
    /// Borrowed `TinyInt` column, mapped to `i8`.
    TinyInt(&'a [i8]),
    /// Borrowed `SmallInt` column, mapped to `i16`.
    SmallInt(&'a [i16]),
    /// Borrowed `Int` column, mapped to `i32`.
    Int(&'a [i32]),
    /// Borrowed `BigInt` column, mapped to `i64`.
    BigInt(&'a [i64]),
    /// Borrowed Int128 column, mapped to `i128`.
    Int128(&'a [i128]),
    /// Borrowed Decimal75(precision, scale, column), mapped to 'i256'
    Decimal75(Precision, i8, Vec<[u64; 4]>),
    /// Column of big ints for committing to, montgomery-reduced from a Scalar column.
    Scalar(Vec<[u64; 4]>),
    /// Column of limbs for committing to scalars, hashed from a `VarChar` column.
    VarChar(Vec<[u64; 4]>),
    /// Column of limbs for committing to scalars, hashed from a `Binary` column.
    VarBinary(Vec<[u64; 4]>),
    /// Borrowed Timestamp column with Timezone, mapped to `i64`.
    TimestampTZ(PoSQLTimeUnit, PoSQLTimeZone, &'a [i64]),
}

impl CommittableColumn<'_> {
    /// Returns the length of the column.
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            CommittableColumn::Uint8(col) => col.len(),
            CommittableColumn::TinyInt(col) => col.len(),
            CommittableColumn::SmallInt(col) => col.len(),
            CommittableColumn::Int(col) => col.len(),
            CommittableColumn::BigInt(col) | CommittableColumn::TimestampTZ(_, _, col) => col.len(),
            CommittableColumn::Int128(col) => col.len(),
            CommittableColumn::Decimal75(_, _, col)
            | CommittableColumn::Scalar(col)
            | CommittableColumn::VarChar(col)
            | CommittableColumn::VarBinary(col) => col.len(),
            CommittableColumn::Boolean(col) => col.len(),
        }
    }

    /// Returns true if the column is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the type of the column.
    #[must_use]
    pub fn column_type(&self) -> ColumnType {
        self.into()
    }
}

impl<'a> From<&CommittableColumn<'a>> for ColumnType {
    fn from(value: &CommittableColumn<'a>) -> Self {
        match value {
            CommittableColumn::Uint8(_) => ColumnType::Uint8,
            CommittableColumn::TinyInt(_) => ColumnType::TinyInt,
            CommittableColumn::SmallInt(_) => ColumnType::SmallInt,
            CommittableColumn::Int(_) => ColumnType::Int,
            CommittableColumn::BigInt(_) => ColumnType::BigInt,
            CommittableColumn::Int128(_) => ColumnType::Int128,
            CommittableColumn::Decimal75(precision, scale, _) => {
                ColumnType::Decimal75(*precision, *scale)
            }
            CommittableColumn::Scalar(_) => ColumnType::Scalar,
            CommittableColumn::VarChar(_) => ColumnType::VarChar,
            CommittableColumn::VarBinary(_) => ColumnType::VarBinary,
            CommittableColumn::Boolean(_) => ColumnType::Boolean,
            CommittableColumn::TimestampTZ(tu, tz, _) => ColumnType::TimestampTZ(*tu, *tz),
        }
    }
}

impl<'a, S: Scalar> From<&Column<'a, S>> for CommittableColumn<'a> {
    fn from(value: &Column<'a, S>) -> Self {
        match value {
            Column::Boolean(bools) => CommittableColumn::Boolean(bools),
            Column::Uint8(ints) => CommittableColumn::Uint8(ints),
            Column::TinyInt(ints) => CommittableColumn::TinyInt(ints),
            Column::SmallInt(ints) => CommittableColumn::SmallInt(ints),
            Column::Int(ints) => CommittableColumn::Int(ints),
            Column::BigInt(ints) => CommittableColumn::BigInt(ints),
            Column::Int128(ints) => CommittableColumn::Int128(ints),
            Column::Decimal75(precision, scale, decimals) => {
                let as_limbs: Vec<_> = decimals.iter().map(RefInto::<[u64; 4]>::ref_into).collect();
                CommittableColumn::Decimal75(*precision, *scale, as_limbs)
            }
            Column::Scalar(scalars) => (scalars as &[_]).into(),
            Column::VarChar((_, scalars)) => {
                let as_limbs: Vec<_> = scalars.iter().map(RefInto::<[u64; 4]>::ref_into).collect();
                CommittableColumn::VarChar(as_limbs)
            }
            Column::VarBinary((_, scalars)) => {
                let as_limbs: Vec<_> = scalars.iter().map(RefInto::<[u64; 4]>::ref_into).collect();
                CommittableColumn::VarBinary(as_limbs)
            }
            Column::TimestampTZ(tu, tz, times) => CommittableColumn::TimestampTZ(*tu, *tz, times),
        }
    }
}

impl<'a, S: Scalar> From<Column<'a, S>> for CommittableColumn<'a> {
    fn from(value: Column<'a, S>) -> Self {
        (&value).into()
    }
}

impl<'a, S: Scalar> From<&'a OwnedColumn<S>> for CommittableColumn<'a> {
    fn from(value: &'a OwnedColumn<S>) -> Self {
        match value {
            OwnedColumn::Boolean(bools) => CommittableColumn::Boolean(bools),
            OwnedColumn::Uint8(ints) => CommittableColumn::Uint8(ints),
            OwnedColumn::TinyInt(ints) => (ints as &[_]).into(),
            OwnedColumn::SmallInt(ints) => (ints as &[_]).into(),
            OwnedColumn::Int(ints) => (ints as &[_]).into(),
            OwnedColumn::BigInt(ints) => (ints as &[_]).into(),
            OwnedColumn::Int128(ints) => (ints as &[_]).into(),
            OwnedColumn::Decimal75(precision, scale, decimals) => CommittableColumn::Decimal75(
                *precision,
                *scale,
                decimals
                    .iter()
                    .map(Into::<S>::into)
                    .map(Into::<[u64; 4]>::into)
                    .collect(),
            ),
            OwnedColumn::Scalar(scalars) => (scalars as &[_]).into(),
            OwnedColumn::VarChar(strings) => CommittableColumn::VarChar(
                strings
                    .iter()
                    .map(Into::<S>::into)
                    .map(Into::<[u64; 4]>::into)
                    .collect(),
            ),
            OwnedColumn::VarBinary(bytes) => CommittableColumn::VarBinary(
                bytes
                    .iter()
                    .map(|b| S::from_byte_slice_via_hash(b))
                    .map(Into::<[u64; 4]>::into)
                    .collect(),
            ),
            OwnedColumn::TimestampTZ(tu, tz, times) => {
                CommittableColumn::TimestampTZ(*tu, *tz, times as &[_])
            }
        }
    }
}

impl<'a> From<&'a [u8]> for CommittableColumn<'a> {
    fn from(value: &'a [u8]) -> Self {
        CommittableColumn::Uint8(value)
    }
}
impl<'a> From<&'a [i8]> for CommittableColumn<'a> {
    fn from(value: &'a [i8]) -> Self {
        CommittableColumn::TinyInt(value)
    }
}
impl<'a> From<&'a [i16]> for CommittableColumn<'a> {
    fn from(value: &'a [i16]) -> Self {
        CommittableColumn::SmallInt(value)
    }
}
impl<'a> From<&'a [i32]> for CommittableColumn<'a> {
    fn from(value: &'a [i32]) -> Self {
        CommittableColumn::Int(value)
    }
}

impl<'a> From<&'a [i64]> for CommittableColumn<'a> {
    fn from(value: &'a [i64]) -> Self {
        CommittableColumn::BigInt(value)
    }
}

impl<'a> From<&'a [i128]> for CommittableColumn<'a> {
    fn from(value: &'a [i128]) -> Self {
        CommittableColumn::Int128(value)
    }
}
impl<'a, S: Scalar> From<&'a [S]> for CommittableColumn<'a> {
    fn from(value: &'a [S]) -> Self {
        CommittableColumn::Scalar(
            if_rayon!(value.par_iter(), value.iter())
                .map(RefInto::<[u64; 4]>::ref_into)
                .collect(),
        )
    }
}
impl<'a> From<&'a [bool]> for CommittableColumn<'a> {
    fn from(value: &'a [bool]) -> Self {
        CommittableColumn::Boolean(value)
    }
}

#[cfg(feature = "blitzar")]
impl<'a, 'b> From<&'a CommittableColumn<'b>> for Sequence<'a> {
    fn from(value: &'a CommittableColumn<'b>) -> Self {
        match value {
            CommittableColumn::Uint8(ints) => Sequence::from(*ints),
            CommittableColumn::TinyInt(ints) => Sequence::from(*ints),
            CommittableColumn::SmallInt(ints) => Sequence::from(*ints),
            CommittableColumn::Int(ints) => Sequence::from(*ints),
            CommittableColumn::BigInt(ints) => Sequence::from(*ints),
            CommittableColumn::Int128(ints) => Sequence::from(*ints),
            CommittableColumn::Decimal75(_, _, limbs)
            | CommittableColumn::Scalar(limbs)
            | CommittableColumn::VarChar(limbs)
            | CommittableColumn::VarBinary(limbs) => Sequence::from(limbs),
            CommittableColumn::Boolean(bools) => Sequence::from(*bools),
            CommittableColumn::TimestampTZ(_, _, times) => Sequence::from(*times),
        }
    }
}

#[cfg(all(test, feature = "blitzar"))]
mod tests {
    use super::*;
    use crate::{base::scalar::test_scalar::TestScalar, proof_primitive::dory::DoryScalar};
    use blitzar::compute::compute_curve25519_commitments;
    use curve25519_dalek::ristretto::CompressedRistretto;

    #[test]
    fn we_can_get_type_and_length_of_varbinary_column() {
        // empty case
        let varbinary_committable_column = CommittableColumn::VarBinary(Vec::new());
        assert_eq!(varbinary_committable_column.len(), 0);
        assert!(varbinary_committable_column.is_empty());
        assert_eq!(
            varbinary_committable_column.column_type(),
            ColumnType::VarBinary
        );

        let limbs = vec![[1, 2, 3, 4], [5, 6, 7, 8]];
        let varbinary_committable_column = CommittableColumn::VarBinary(limbs.clone());
        assert_eq!(varbinary_committable_column.len(), 2);
        assert!(!varbinary_committable_column.is_empty());
        assert_eq!(
            varbinary_committable_column.column_type(),
            ColumnType::VarBinary
        );
    }

    #[test]
    fn we_can_convert_from_owned_varbinary_column() {
        // empty case
        let owned_column = OwnedColumn::<TestScalar>::VarBinary(Vec::new());
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(from_owned_column, CommittableColumn::VarBinary(vec![]));

        let byte_data = vec![b"foo".to_vec(), b"bar".to_vec()];
        let owned_column = OwnedColumn::<TestScalar>::VarBinary(byte_data.clone());
        let from_owned_column = CommittableColumn::from(&owned_column);

        match from_owned_column {
            CommittableColumn::VarBinary(limbs) => {
                assert_eq!(limbs.len(), byte_data.len());
            }
            _ => panic!("Expected VarBinary"),
        }
    }

    #[test]
    fn we_can_commit_to_varbinary_column_through_committable_column() {
        let committable_column = CommittableColumn::VarBinary(vec![]);
        let sequence = Sequence::from(&committable_column);
        let mut commitment_buffer = [CompressedRistretto::default()];
        compute_curve25519_commitments(&mut commitment_buffer, &[sequence], 0);
        assert_eq!(commitment_buffer[0], CompressedRistretto::default());

        let hashed_limbs = vec![[1, 2, 3, 4], [5, 6, 7, 8], [9, 10, 11, 12]];
        let committable_column = CommittableColumn::VarBinary(hashed_limbs.clone());

        let sequence_actual = Sequence::from(&committable_column);
        let sequence_expected = Sequence::from(hashed_limbs.as_slice());
        let mut commitment_buffer = [CompressedRistretto::default(); 2];
        compute_curve25519_commitments(
            &mut commitment_buffer,
            &[sequence_actual, sequence_expected],
            0,
        );
        assert_eq!(commitment_buffer[0], commitment_buffer[1]);
    }

    #[test]
    fn we_can_convert_from_owned_decimal75_column_to_committable_column() {
        let decimals = vec![
            TestScalar::from(-1),
            TestScalar::from(1),
            TestScalar::from(2),
        ];
        let decimal_column = OwnedColumn::Decimal75(Precision::new(75).unwrap(), -1, decimals);

        let res_committable_column: CommittableColumn = (&decimal_column).into();
        let test_committable_column: CommittableColumn = CommittableColumn::Decimal75(
            Precision::new(75).unwrap(),
            -1,
            [-1, 1, 2]
                .map(<TestScalar>::from)
                .map(<[u64; 4]>::from)
                .into(),
        );

        assert_eq!(res_committable_column, test_committable_column);
    }

    #[test]
    fn we_can_get_type_and_length_of_timestamp_column() {
        // empty case
        let committable_column =
            CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &[]);
        assert_eq!(committable_column.len(), 0);
        assert!(committable_column.is_empty());
        assert_eq!(
            committable_column.column_type(),
            ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc())
        );

        let committable_column = CommittableColumn::TimestampTZ(
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            &[12, 34, 56],
        );
        assert_eq!(committable_column.len(), 3);
        assert!(!committable_column.is_empty());
        assert_eq!(
            committable_column.column_type(),
            ColumnType::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc())
        );
    }

    #[test]
    fn we_can_get_type_and_length_of_tinyint_column() {
        // empty case
        let tinyint_committable_column = CommittableColumn::TinyInt(&[]);
        assert_eq!(tinyint_committable_column.len(), 0);
        assert!(tinyint_committable_column.is_empty());
        assert_eq!(
            tinyint_committable_column.column_type(),
            ColumnType::TinyInt
        );

        let tinyint_committable_column = CommittableColumn::TinyInt(&[12, 34, 56]);
        assert_eq!(tinyint_committable_column.len(), 3);
        assert!(!tinyint_committable_column.is_empty());
        assert_eq!(
            tinyint_committable_column.column_type(),
            ColumnType::TinyInt
        );
    }

    #[test]
    fn we_can_get_type_and_length_of_smallint_column() {
        // empty case
        let smallint_committable_column = CommittableColumn::SmallInt(&[]);
        assert_eq!(smallint_committable_column.len(), 0);
        assert!(smallint_committable_column.is_empty());
        assert_eq!(
            smallint_committable_column.column_type(),
            ColumnType::SmallInt
        );

        let smallint_committable_column = CommittableColumn::SmallInt(&[12, 34, 56]);
        assert_eq!(smallint_committable_column.len(), 3);
        assert!(!smallint_committable_column.is_empty());
        assert_eq!(
            smallint_committable_column.column_type(),
            ColumnType::SmallInt
        );
    }

    #[test]
    fn we_can_get_type_and_length_of_int_column() {
        // empty case
        let int_committable_column = CommittableColumn::Int(&[]);
        assert_eq!(int_committable_column.len(), 0);
        assert!(int_committable_column.is_empty());
        assert_eq!(int_committable_column.column_type(), ColumnType::Int);

        let int_committable_column = CommittableColumn::Int(&[12, 34, 56]);
        assert_eq!(int_committable_column.len(), 3);
        assert!(!int_committable_column.is_empty());
        assert_eq!(int_committable_column.column_type(), ColumnType::Int);
    }

    #[test]
    fn we_can_get_type_and_length_of_bigint_column() {
        // empty case
        let bigint_committable_column = CommittableColumn::BigInt(&[]);
        assert_eq!(bigint_committable_column.len(), 0);
        assert!(bigint_committable_column.is_empty());
        assert_eq!(bigint_committable_column.column_type(), ColumnType::BigInt);

        let bigint_committable_column = CommittableColumn::BigInt(&[12, 34, 56]);
        assert_eq!(bigint_committable_column.len(), 3);
        assert!(!bigint_committable_column.is_empty());
        assert_eq!(bigint_committable_column.column_type(), ColumnType::BigInt);
    }

    #[test]
    fn we_can_get_type_and_length_of_decimal_column() {
        // empty case
        let decimal_committable_column =
            CommittableColumn::Decimal75(Precision::new(1).unwrap(), 0, [].to_vec());
        assert_eq!(decimal_committable_column.len(), 0);
        assert!(decimal_committable_column.is_empty());
        assert_eq!(
            decimal_committable_column.column_type(),
            ColumnType::Decimal75(Precision::new(1).unwrap(), 0)
        );
        let decimal_committable_column = CommittableColumn::Decimal75(
            Precision::new(10).unwrap(),
            10,
            vec![[12, 0, 0, 0], [34, 0, 0, 0], [56, 0, 0, 0]],
        );
        assert_eq!(decimal_committable_column.len(), 3);
        assert!(!decimal_committable_column.is_empty());
        assert_eq!(
            decimal_committable_column.column_type(),
            ColumnType::Decimal75(Precision::new(10).unwrap(), 10)
        );
    }

    #[test]
    fn we_can_get_type_and_length_of_int128_column() {
        // empty case
        let bigint_committable_column = CommittableColumn::Int128(&[]);
        assert_eq!(bigint_committable_column.len(), 0);
        assert!(bigint_committable_column.is_empty());
        assert_eq!(bigint_committable_column.column_type(), ColumnType::Int128);

        let bigint_committable_column = CommittableColumn::Int128(&[12, 34, 56]);
        assert_eq!(bigint_committable_column.len(), 3);
        assert!(!bigint_committable_column.is_empty());
        assert_eq!(bigint_committable_column.column_type(), ColumnType::Int128);
    }

    #[test]
    fn we_can_get_type_and_length_of_varchar_column() {
        // empty case
        let bigint_committable_column = CommittableColumn::VarChar(Vec::new());
        assert_eq!(bigint_committable_column.len(), 0);
        assert!(bigint_committable_column.is_empty());
        assert_eq!(bigint_committable_column.column_type(), ColumnType::VarChar);

        let bigint_committable_column = CommittableColumn::VarChar(
            ["12", "34", "56"]
                .map(Into::<String>::into)
                .map(Into::<TestScalar>::into)
                .map(Into::<[u64; 4]>::into)
                .into(),
        );
        assert_eq!(bigint_committable_column.len(), 3);
        assert!(!bigint_committable_column.is_empty());
        assert_eq!(bigint_committable_column.column_type(), ColumnType::VarChar);
    }

    #[test]
    fn we_can_get_type_and_length_of_scalar_column() {
        // empty case
        let bigint_committable_column = CommittableColumn::Scalar(Vec::new());
        assert_eq!(bigint_committable_column.len(), 0);
        assert!(bigint_committable_column.is_empty());
        assert_eq!(bigint_committable_column.column_type(), ColumnType::Scalar);

        let bigint_committable_column = CommittableColumn::Scalar(
            [12, 34, 56]
                .map(<TestScalar>::from)
                .map(<[u64; 4]>::from)
                .into(),
        );
        assert_eq!(bigint_committable_column.len(), 3);
        assert!(!bigint_committable_column.is_empty());
        assert_eq!(bigint_committable_column.column_type(), ColumnType::Scalar);
    }

    #[test]
    fn we_can_get_type_and_length_of_boolean_column() {
        // empty case
        let bool_committable_column = CommittableColumn::Boolean(&[]);
        assert_eq!(bool_committable_column.len(), 0);
        assert!(bool_committable_column.is_empty());
        assert_eq!(bool_committable_column.column_type(), ColumnType::Boolean);

        let bool_committable_column = CommittableColumn::Boolean(&[true, false, true]);
        assert_eq!(bool_committable_column.len(), 3);
        assert!(!bool_committable_column.is_empty());
        assert_eq!(bool_committable_column.column_type(), ColumnType::Boolean);
    }

    #[test]
    fn we_can_get_length_of_uint8_column() {
        // empty case
        let bool_committable_column = CommittableColumn::Uint8(&[]);
        assert_eq!(bool_committable_column.len(), 0);
        assert!(bool_committable_column.is_empty());

        let bool_committable_column = CommittableColumn::Uint8(&[12, 34, 56]);
        assert_eq!(bool_committable_column.len(), 3);
        assert!(!bool_committable_column.is_empty());
    }

    #[test]
    fn we_can_convert_from_borrowing_timestamp_column() {
        // empty case
        let from_borrowed_column = CommittableColumn::from(&Column::<TestScalar>::TimestampTZ(
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            &[],
        ));
        assert_eq!(
            from_borrowed_column,
            CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &[])
        );

        // non-empty case
        let timestamps = [1_625_072_400, 1_625_076_000, 1_625_083_200];
        let from_borrowed_column = CommittableColumn::from(&Column::<TestScalar>::TimestampTZ(
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            &timestamps,
        ));
        assert_eq!(
            from_borrowed_column,
            CommittableColumn::TimestampTZ(
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                &timestamps
            )
        );
    }

    #[test]
    fn we_can_convert_from_borrowing_bigint_column() {
        // empty case
        let from_borrowed_column = CommittableColumn::from(&Column::<TestScalar>::BigInt(&[]));
        assert_eq!(from_borrowed_column, CommittableColumn::BigInt(&[]));

        let from_borrowed_column =
            CommittableColumn::from(&Column::<TestScalar>::BigInt(&[12, 34, 56]));
        assert_eq!(
            from_borrowed_column,
            CommittableColumn::BigInt(&[12, 34, 56])
        );
    }

    #[test]
    fn we_can_convert_from_borrowing_tinyint_column() {
        // empty case
        let from_borrowed_column = CommittableColumn::from(&Column::<TestScalar>::TinyInt(&[]));
        assert_eq!(from_borrowed_column, CommittableColumn::TinyInt(&[]));

        let from_borrowed_column =
            CommittableColumn::from(&Column::<TestScalar>::TinyInt(&[12, 34, 56]));
        assert_eq!(
            from_borrowed_column,
            CommittableColumn::TinyInt(&[12, 34, 56])
        );
    }

    #[test]
    fn we_can_convert_from_borrowing_smallint_column() {
        // empty case
        let from_borrowed_column = CommittableColumn::from(&Column::<TestScalar>::SmallInt(&[]));
        assert_eq!(from_borrowed_column, CommittableColumn::SmallInt(&[]));

        let from_borrowed_column =
            CommittableColumn::from(&Column::<TestScalar>::SmallInt(&[12, 34, 56]));
        assert_eq!(
            from_borrowed_column,
            CommittableColumn::SmallInt(&[12, 34, 56])
        );
    }

    #[test]
    fn we_can_convert_from_borrowing_int_column() {
        // empty case
        let from_borrowed_column = CommittableColumn::from(&Column::<TestScalar>::Int(&[]));
        assert_eq!(from_borrowed_column, CommittableColumn::Int(&[]));

        let from_borrowed_column =
            CommittableColumn::from(&Column::<TestScalar>::Int(&[12, 34, 56]));
        assert_eq!(from_borrowed_column, CommittableColumn::Int(&[12, 34, 56]));
    }

    #[test]
    fn we_can_convert_from_borrowing_decimal_column() {
        // Define a non-empty array of TestScalars
        let binding = vec![
            TestScalar::from(-1),
            TestScalar::from(34),
            TestScalar::from(56),
        ];

        let precision = Precision::new(75).unwrap();
        let from_borrowed_column =
            CommittableColumn::from(&Column::Decimal75(precision, 0, &binding));

        let expected_decimals = binding
            .iter()
            .map(|&scalar| scalar.into())
            .collect::<Vec<[u64; 4]>>();

        assert_eq!(
            from_borrowed_column,
            CommittableColumn::Decimal75(Precision::new(75).unwrap(), 0, expected_decimals)
        );
    }

    #[test]
    fn we_can_convert_from_borrowing_int128_column() {
        // empty case
        let from_borrowed_column = CommittableColumn::from(&Column::<TestScalar>::Int128(&[]));
        assert_eq!(from_borrowed_column, CommittableColumn::Int128(&[]));

        let from_borrowed_column =
            CommittableColumn::from(&Column::<TestScalar>::Int128(&[12, 34, 56]));
        assert_eq!(
            from_borrowed_column,
            CommittableColumn::Int128(&[12, 34, 56])
        );
    }

    #[test]
    fn we_can_convert_from_borrowing_varchar_column() {
        // empty case
        let from_borrowed_column =
            CommittableColumn::from(&Column::<TestScalar>::VarChar((&[], &[])));
        assert_eq!(from_borrowed_column, CommittableColumn::VarChar(Vec::new()));

        let varchar_data = ["12", "34", "56"];
        let scalars = varchar_data.map(TestScalar::from);
        let from_borrowed_column =
            CommittableColumn::from(&Column::VarChar((&varchar_data, &scalars)));
        assert_eq!(
            from_borrowed_column,
            CommittableColumn::VarChar(scalars.map(<[u64; 4]>::from).into())
        );
    }

    #[test]
    fn we_can_convert_from_borrowing_scalar_column() {
        // empty case
        let from_borrowed_column = CommittableColumn::from(&Column::<TestScalar>::Scalar(&[]));
        assert_eq!(from_borrowed_column, CommittableColumn::Scalar(Vec::new()));

        let scalars = [12, 34, 56].map(TestScalar::from);
        let from_borrowed_column = CommittableColumn::from(&Column::Scalar(&scalars));
        assert_eq!(
            from_borrowed_column,
            CommittableColumn::Scalar(scalars.map(<[u64; 4]>::from).into())
        );
    }

    #[test]
    fn we_can_convert_from_borrowing_boolean_column() {
        // empty case
        let from_borrowed_column = CommittableColumn::from(&Column::<TestScalar>::Boolean(&[]));
        assert_eq!(from_borrowed_column, CommittableColumn::Boolean(&[]));

        let from_borrowed_column =
            CommittableColumn::from(&Column::<TestScalar>::Boolean(&[true, false, true]));
        assert_eq!(
            from_borrowed_column,
            CommittableColumn::Boolean(&[true, false, true])
        );
    }

    #[test]
    fn we_can_convert_from_owned_bigint_column() {
        // empty case
        let owned_column = OwnedColumn::<TestScalar>::BigInt(Vec::new());
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(from_owned_column, CommittableColumn::BigInt(&[]));

        let owned_column = OwnedColumn::<TestScalar>::BigInt(vec![12, 34, 56]);
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(from_owned_column, CommittableColumn::BigInt(&[12, 34, 56]));
    }

    #[test]
    fn we_can_convert_from_owned_tinyint_column() {
        // empty case
        let owned_column = OwnedColumn::<DoryScalar>::TinyInt(Vec::new());
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(from_owned_column, CommittableColumn::TinyInt(&[]));

        let owned_column = OwnedColumn::<DoryScalar>::TinyInt(vec![12, 34, 56]);
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(from_owned_column, CommittableColumn::TinyInt(&[12, 34, 56]));
    }

    #[test]
    fn we_can_convert_from_owned_smallint_column() {
        // empty case
        let owned_column = OwnedColumn::<DoryScalar>::SmallInt(Vec::new());
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(from_owned_column, CommittableColumn::SmallInt(&[]));

        let owned_column = OwnedColumn::<DoryScalar>::SmallInt(vec![12, 34, 56]);
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(
            from_owned_column,
            CommittableColumn::SmallInt(&[12, 34, 56])
        );
    }

    #[test]
    fn we_can_convert_from_owned_timestamp_column() {
        // empty case
        let owned_column = OwnedColumn::<TestScalar>::TimestampTZ(
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            Vec::new(),
        );
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(
            from_owned_column,
            CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &[])
        );

        // non-empty case
        let timestamps = vec![1_625_072_400, 1_625_076_000, 1_625_083_200];
        let owned_column = OwnedColumn::<TestScalar>::TimestampTZ(
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            timestamps.clone(),
        );
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(
            from_owned_column,
            CommittableColumn::TimestampTZ(
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                &timestamps
            )
        );
    }

    #[test]
    fn we_can_convert_from_owned_int_column() {
        // empty case
        let owned_column = OwnedColumn::<DoryScalar>::Int(Vec::new());
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(from_owned_column, CommittableColumn::Int(&[]));

        let owned_column = OwnedColumn::<DoryScalar>::Int(vec![12, 34, 56]);
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(from_owned_column, CommittableColumn::Int(&[12, 34, 56]));
    }

    #[test]
    fn we_can_convert_from_owned_int128_column() {
        // empty case
        let owned_column = OwnedColumn::<TestScalar>::Int128(Vec::new());
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(from_owned_column, CommittableColumn::Int128(&[]));

        let owned_column = OwnedColumn::<TestScalar>::Int128(vec![12, 34, 56]);
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(from_owned_column, CommittableColumn::Int128(&[12, 34, 56]));
    }

    #[test]
    fn we_can_convert_from_owned_varchar_column() {
        // empty case
        let owned_column = OwnedColumn::<TestScalar>::VarChar(Vec::new());
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(from_owned_column, CommittableColumn::VarChar(Vec::new()));

        let strings = ["12", "34", "56"].map(String::from);
        let owned_column = OwnedColumn::<TestScalar>::VarChar(strings.to_vec());
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(
            from_owned_column,
            CommittableColumn::VarChar(strings.map(TestScalar::from).map(<[u64; 4]>::from).into())
        );
    }

    #[test]
    fn we_can_convert_from_owned_scalar_column() {
        // empty case
        let owned_column = OwnedColumn::<TestScalar>::Scalar(Vec::new());
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(from_owned_column, CommittableColumn::Scalar(Vec::new()));

        let scalars = [12, 34, 56].map(TestScalar::from);
        let owned_column = OwnedColumn::Scalar(scalars.to_vec());
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(
            from_owned_column,
            CommittableColumn::Scalar(scalars.map(<[u64; 4]>::from).into())
        );
    }

    #[test]
    fn we_can_convert_from_owned_boolean_column() {
        // empty case
        let owned_column = OwnedColumn::<DoryScalar>::Boolean(Vec::new());
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(from_owned_column, CommittableColumn::Boolean(&[]));

        let booleans = [true, false, true];
        let owned_column: OwnedColumn<DoryScalar> = OwnedColumn::Boolean(booleans.to_vec());
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(from_owned_column, CommittableColumn::Boolean(&booleans));
    }

    #[test]
    fn we_can_commit_to_bigint_column_through_committable_column() {
        // empty case
        let committable_column = CommittableColumn::BigInt(&[]);
        let sequence = Sequence::from(&committable_column);
        let mut commitment_buffer = [CompressedRistretto::default()];
        compute_curve25519_commitments(&mut commitment_buffer, &[sequence], 0);
        assert_eq!(commitment_buffer[0], CompressedRistretto::default());

        // nonempty case
        let values = [12, 34, 56];
        let committable_column = CommittableColumn::BigInt(&values);

        let sequence_actual = Sequence::from(&committable_column);
        let sequence_expected = Sequence::from(values.as_slice());
        let mut commitment_buffer = [CompressedRistretto::default(); 2];
        compute_curve25519_commitments(
            &mut commitment_buffer,
            &[sequence_actual, sequence_expected],
            0,
        );
        assert_eq!(commitment_buffer[0], commitment_buffer[1]);
    }

    #[test]
    fn we_can_commit_to_uint8_column_through_committable_column() {
        // empty case
        let committable_column = CommittableColumn::Uint8(&[]);
        let sequence = Sequence::from(&committable_column);
        let mut commitment_buffer = [CompressedRistretto::default()];
        compute_curve25519_commitments(&mut commitment_buffer, &[sequence], 0);
        assert_eq!(commitment_buffer[0], CompressedRistretto::default());

        // nonempty case
        let values = [12, 34, 56];
        let committable_column = CommittableColumn::Uint8(&values);

        let sequence_actual = Sequence::from(&committable_column);
        let sequence_expected = Sequence::from(values.as_slice());
        let mut commitment_buffer = [CompressedRistretto::default(); 2];
        compute_curve25519_commitments(
            &mut commitment_buffer,
            &[sequence_actual, sequence_expected],
            0,
        );
        assert_eq!(commitment_buffer[0], commitment_buffer[1]);
    }

    #[test]
    fn we_can_commit_to_tinyint_column_through_committable_column() {
        // empty case
        let committable_column = CommittableColumn::TinyInt(&[]);
        let sequence = Sequence::from(&committable_column);
        let mut commitment_buffer = [CompressedRistretto::default()];
        compute_curve25519_commitments(&mut commitment_buffer, &[sequence], 0);
        assert_eq!(commitment_buffer[0], CompressedRistretto::default());

        // nonempty case
        let values = [12, 34, 56];
        let committable_column = CommittableColumn::TinyInt(&values);

        let sequence_actual = Sequence::from(&committable_column);
        let sequence_expected = Sequence::from(values.as_slice());
        let mut commitment_buffer = [CompressedRistretto::default(); 2];
        compute_curve25519_commitments(
            &mut commitment_buffer,
            &[sequence_actual, sequence_expected],
            0,
        );
        assert_eq!(commitment_buffer[0], commitment_buffer[1]);
    }

    #[test]
    fn we_can_commit_to_smallint_column_through_committable_column() {
        // empty case
        let committable_column = CommittableColumn::SmallInt(&[]);
        let sequence = Sequence::from(&committable_column);
        let mut commitment_buffer = [CompressedRistretto::default()];
        compute_curve25519_commitments(&mut commitment_buffer, &[sequence], 0);
        assert_eq!(commitment_buffer[0], CompressedRistretto::default());

        // nonempty case
        let values = [12, 34, 56];
        let committable_column = CommittableColumn::SmallInt(&values);

        let sequence_actual = Sequence::from(&committable_column);
        let sequence_expected = Sequence::from(values.as_slice());
        let mut commitment_buffer = [CompressedRistretto::default(); 2];
        compute_curve25519_commitments(
            &mut commitment_buffer,
            &[sequence_actual, sequence_expected],
            0,
        );
        assert_eq!(commitment_buffer[0], commitment_buffer[1]);
    }

    #[test]
    fn we_can_commit_to_int_column_through_committable_column() {
        // empty case
        let committable_column = CommittableColumn::Int(&[]);
        let sequence = Sequence::from(&committable_column);
        let mut commitment_buffer = [CompressedRistretto::default()];
        compute_curve25519_commitments(&mut commitment_buffer, &[sequence], 0);
        assert_eq!(commitment_buffer[0], CompressedRistretto::default());

        // nonempty case
        let values = [12, 34, 56];
        let committable_column = CommittableColumn::Int(&values);

        let sequence_actual = Sequence::from(&committable_column);
        let sequence_expected = Sequence::from(values.as_slice());
        let mut commitment_buffer = [CompressedRistretto::default(); 2];
        compute_curve25519_commitments(
            &mut commitment_buffer,
            &[sequence_actual, sequence_expected],
            0,
        );
        assert_eq!(commitment_buffer[0], commitment_buffer[1]);
    }

    #[test]
    fn we_can_commit_to_decimal_column_through_committable_column() {
        // empty case
        let committable_column =
            CommittableColumn::Decimal75(Precision::new(1).unwrap(), 0, [].to_vec());
        let sequence = Sequence::from(&committable_column);
        let mut commitment_buffer = [CompressedRistretto::default()];
        compute_curve25519_commitments(&mut commitment_buffer, &[sequence], 0);
        assert_eq!(commitment_buffer[0], CompressedRistretto::default());

        // nonempty case
        let values = [
            TestScalar::from(12),
            TestScalar::from(34),
            TestScalar::from(56),
        ]
        .map(<[u64; 4]>::from);
        let committable_column =
            CommittableColumn::Decimal75(Precision::new(1).unwrap(), 0, (values).to_vec());

        let sequence_actual = Sequence::from(&committable_column);
        let sequence_expected = Sequence::from(values.as_slice());
        let mut commitment_buffer = [CompressedRistretto::default(); 2];
        compute_curve25519_commitments(
            &mut commitment_buffer,
            &[sequence_actual, sequence_expected],
            0,
        );
        assert_eq!(commitment_buffer[0], commitment_buffer[1]);
    }

    // Committing to Int128 columns is blocked by PROOF-772 without a workaround
    #[test]
    #[ignore = "Blocked by PROOF-772"]
    fn we_can_commit_to_int128_column_through_committable_column() {
        // empty case
        let committable_column = CommittableColumn::Int128(&[]);
        let sequence = Sequence::from(&committable_column);
        let mut commitment_buffer = [CompressedRistretto::default()];
        compute_curve25519_commitments(&mut commitment_buffer, &[sequence], 0);
        assert_eq!(commitment_buffer[0], CompressedRistretto::default());

        // nonempty case
        let values = [12, 34, 56];
        let committable_column = CommittableColumn::Int128(&values);

        let sequence_actual = Sequence::from(&committable_column);
        let sequence_expected = Sequence::from(values.as_slice());
        let mut commitment_buffer = [CompressedRistretto::default(); 2];
        compute_curve25519_commitments(
            &mut commitment_buffer,
            &[sequence_actual, sequence_expected],
            0,
        );
        assert_eq!(commitment_buffer[0], commitment_buffer[1]);
    }

    #[test]
    fn we_can_commit_to_varchar_column_through_committable_column() {
        // empty case
        let committable_column = CommittableColumn::VarChar(vec![]);
        let sequence = Sequence::from(&committable_column);
        let mut commitment_buffer = [CompressedRistretto::default()];
        compute_curve25519_commitments(&mut commitment_buffer, &[sequence], 0);
        assert_eq!(commitment_buffer[0], CompressedRistretto::default());

        // nonempty case
        let values = ["12", "34", "56"].map(String::from);
        let owned_column = OwnedColumn::<TestScalar>::VarChar(values.to_vec());
        let committable_column = CommittableColumn::from(&owned_column);

        let sequence_actual = Sequence::from(&committable_column);
        let scalars = values.map(TestScalar::from).map(<[u64; 4]>::from);
        let sequence_expected = Sequence::from(scalars.as_slice());
        let mut commitment_buffer = [CompressedRistretto::default(); 2];
        compute_curve25519_commitments(
            &mut commitment_buffer,
            &[sequence_actual, sequence_expected],
            0,
        );
        assert_eq!(commitment_buffer[0], commitment_buffer[1]);
    }

    #[test]
    fn we_can_commit_to_scalar_column_through_committable_column() {
        // empty case
        let committable_column = CommittableColumn::Scalar(vec![]);
        let sequence = Sequence::from(&committable_column);
        let mut commitment_buffer = [CompressedRistretto::default()];
        compute_curve25519_commitments(&mut commitment_buffer, &[sequence], 0);
        assert_eq!(commitment_buffer[0], CompressedRistretto::default());

        // nonempty case
        let values = [12, 34, 56].map(TestScalar::from);
        let owned_column = OwnedColumn::Scalar(values.to_vec());
        let committable_column = CommittableColumn::from(&owned_column);

        let sequence_actual = Sequence::from(&committable_column);
        let scalars = values.map(TestScalar::from).map(<[u64; 4]>::from);
        let sequence_expected = Sequence::from(scalars.as_slice());
        let mut commitment_buffer = [CompressedRistretto::default(); 2];
        compute_curve25519_commitments(
            &mut commitment_buffer,
            &[sequence_actual, sequence_expected],
            0,
        );
        assert_eq!(commitment_buffer[0], commitment_buffer[1]);
    }

    #[test]
    fn we_can_commit_to_boolean_column_through_committable_column() {
        // empty case
        let committable_column = CommittableColumn::Boolean(&[]);
        let sequence = Sequence::from(&committable_column);
        let mut commitment_buffer = [CompressedRistretto::default()];
        compute_curve25519_commitments(&mut commitment_buffer, &[sequence], 0);
        assert_eq!(commitment_buffer[0], CompressedRistretto::default());

        // nonempty case
        let values = [true, false, true];
        let committable_column = CommittableColumn::Boolean(&values);

        let sequence_actual = Sequence::from(&committable_column);
        let sequence_expected = Sequence::from(values.as_slice());
        let mut commitment_buffer = [CompressedRistretto::default(); 2];
        compute_curve25519_commitments(
            &mut commitment_buffer,
            &[sequence_actual, sequence_expected],
            0,
        );
        assert_eq!(commitment_buffer[0], commitment_buffer[1]);
    }

    #[test]
    fn we_can_commit_to_timestamp_column_through_committable_column() {
        // Empty case
        let committable_column =
            CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &[]);
        let sequence = Sequence::from(&committable_column);
        let mut commitment_buffer = [CompressedRistretto::default()];
        compute_curve25519_commitments(&mut commitment_buffer, &[sequence], 0);
        assert_eq!(commitment_buffer[0], CompressedRistretto::default());

        // Non-empty case
        let timestamps = [1_625_072_400, 1_625_076_000, 1_625_083_200];
        let committable_column = CommittableColumn::TimestampTZ(
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            &timestamps,
        );

        let sequence_actual = Sequence::from(&committable_column);
        let sequence_expected = Sequence::from(timestamps.as_slice());
        let mut commitment_buffer = [CompressedRistretto::default(); 2];
        compute_curve25519_commitments(
            &mut commitment_buffer,
            &[sequence_actual, sequence_expected],
            0,
        );
        assert_eq!(commitment_buffer[0], commitment_buffer[1]);
    }
}

#[cfg(test)]
mod tests_without_blitzar {
    use super::CommittableColumn;
    use crate::base::{
        database::{Column, ColumnType, OwnedColumn},
        math::decimal::Precision,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::test_scalar::TestScalar,
    };

    // --- len() and is_empty() for every variant ---

    #[test]
    fn boolean_len_empty() {
        let col = CommittableColumn::Boolean(&[]);
        assert_eq!(col.len(), 0);
        assert!(col.is_empty());
    }

    #[test]
    fn boolean_len_nonempty() {
        let col = CommittableColumn::Boolean(&[true, false, true]);
        assert_eq!(col.len(), 3);
        assert!(!col.is_empty());
    }

    #[test]
    fn uint8_len_empty() {
        let col = CommittableColumn::Uint8(&[]);
        assert_eq!(col.len(), 0);
        assert!(col.is_empty());
    }

    #[test]
    fn uint8_len_nonempty() {
        let col = CommittableColumn::Uint8(&[1u8, 2, 3]);
        assert_eq!(col.len(), 3);
        assert!(!col.is_empty());
    }

    #[test]
    fn tinyint_len_empty() {
        let col = CommittableColumn::TinyInt(&[]);
        assert_eq!(col.len(), 0);
        assert!(col.is_empty());
    }

    #[test]
    fn tinyint_len_nonempty() {
        let col = CommittableColumn::TinyInt(&[1i8, -2, 3]);
        assert_eq!(col.len(), 3);
        assert!(!col.is_empty());
    }

    #[test]
    fn smallint_len_empty() {
        let col = CommittableColumn::SmallInt(&[]);
        assert_eq!(col.len(), 0);
        assert!(col.is_empty());
    }

    #[test]
    fn smallint_len_nonempty() {
        let col = CommittableColumn::SmallInt(&[1i16, 2, 3, 4]);
        assert_eq!(col.len(), 4);
        assert!(!col.is_empty());
    }

    #[test]
    fn int_len_empty() {
        let col = CommittableColumn::Int(&[]);
        assert_eq!(col.len(), 0);
        assert!(col.is_empty());
    }

    #[test]
    fn int_len_nonempty() {
        let col = CommittableColumn::Int(&[1i32, 2]);
        assert_eq!(col.len(), 2);
        assert!(!col.is_empty());
    }

    #[test]
    fn bigint_len_empty() {
        let col = CommittableColumn::BigInt(&[]);
        assert_eq!(col.len(), 0);
        assert!(col.is_empty());
    }

    #[test]
    fn bigint_len_nonempty() {
        let col = CommittableColumn::BigInt(&[10i64, 20, 30]);
        assert_eq!(col.len(), 3);
        assert!(!col.is_empty());
    }

    #[test]
    fn int128_len_empty() {
        let col = CommittableColumn::Int128(&[]);
        assert_eq!(col.len(), 0);
        assert!(col.is_empty());
    }

    #[test]
    fn int128_len_nonempty() {
        let col = CommittableColumn::Int128(&[1i128, 2, 3, 4, 5]);
        assert_eq!(col.len(), 5);
        assert!(!col.is_empty());
    }

    #[test]
    fn decimal75_len_empty() {
        let col = CommittableColumn::Decimal75(Precision::new(10).unwrap(), 2, alloc::vec![]);
        assert_eq!(col.len(), 0);
        assert!(col.is_empty());
    }

    #[test]
    fn decimal75_len_nonempty() {
        let limbs = alloc::vec![[0u64; 4], [1u64, 0, 0, 0]];
        let col = CommittableColumn::Decimal75(Precision::new(10).unwrap(), 2, limbs);
        assert_eq!(col.len(), 2);
        assert!(!col.is_empty());
    }

    #[test]
    fn scalar_len_empty() {
        let col = CommittableColumn::Scalar(alloc::vec![]);
        assert_eq!(col.len(), 0);
        assert!(col.is_empty());
    }

    #[test]
    fn scalar_len_nonempty() {
        let limbs = alloc::vec![[1u64; 4], [2u64; 4], [3u64; 4]];
        let col = CommittableColumn::Scalar(limbs);
        assert_eq!(col.len(), 3);
        assert!(!col.is_empty());
    }

    #[test]
    fn varchar_len_empty() {
        let col = CommittableColumn::VarChar(alloc::vec![]);
        assert_eq!(col.len(), 0);
        assert!(col.is_empty());
    }

    #[test]
    fn varchar_len_nonempty() {
        let limbs = alloc::vec![[0u64; 4]; 4];
        let col = CommittableColumn::VarChar(limbs);
        assert_eq!(col.len(), 4);
        assert!(!col.is_empty());
    }

    #[test]
    fn varbinary_len_empty() {
        let col = CommittableColumn::VarBinary(alloc::vec![]);
        assert_eq!(col.len(), 0);
        assert!(col.is_empty());
    }

    #[test]
    fn varbinary_len_nonempty() {
        let limbs = alloc::vec![[9u64; 4], [8u64; 4]];
        let col = CommittableColumn::VarBinary(limbs);
        assert_eq!(col.len(), 2);
        assert!(!col.is_empty());
    }

    #[test]
    fn timestamptz_len_empty() {
        let col = CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &[]);
        assert_eq!(col.len(), 0);
        assert!(col.is_empty());
    }

    #[test]
    fn timestamptz_len_nonempty() {
        let col = CommittableColumn::TimestampTZ(
            PoSQLTimeUnit::Millisecond,
            PoSQLTimeZone::utc(),
            &[100i64, 200, 300],
        );
        assert_eq!(col.len(), 3);
        assert!(!col.is_empty());
    }

    // --- column_type() for every variant ---

    #[test]
    fn boolean_column_type() {
        let col = CommittableColumn::Boolean(&[true]);
        assert_eq!(col.column_type(), ColumnType::Boolean);
    }

    #[test]
    fn uint8_column_type() {
        let col = CommittableColumn::Uint8(&[1u8]);
        assert_eq!(col.column_type(), ColumnType::Uint8);
    }

    #[test]
    fn tinyint_column_type() {
        let col = CommittableColumn::TinyInt(&[1i8]);
        assert_eq!(col.column_type(), ColumnType::TinyInt);
    }

    #[test]
    fn smallint_column_type() {
        let col = CommittableColumn::SmallInt(&[1i16]);
        assert_eq!(col.column_type(), ColumnType::SmallInt);
    }

    #[test]
    fn int_column_type() {
        let col = CommittableColumn::Int(&[1i32]);
        assert_eq!(col.column_type(), ColumnType::Int);
    }

    #[test]
    fn bigint_column_type() {
        let col = CommittableColumn::BigInt(&[1i64]);
        assert_eq!(col.column_type(), ColumnType::BigInt);
    }

    #[test]
    fn int128_column_type() {
        let col = CommittableColumn::Int128(&[1i128]);
        assert_eq!(col.column_type(), ColumnType::Int128);
    }

    #[test]
    fn decimal75_column_type() {
        let prec = Precision::new(20).unwrap();
        let col = CommittableColumn::Decimal75(prec, 5, alloc::vec![]);
        assert_eq!(col.column_type(), ColumnType::Decimal75(prec, 5));
    }

    #[test]
    fn scalar_column_type() {
        let col = CommittableColumn::Scalar(alloc::vec![]);
        assert_eq!(col.column_type(), ColumnType::Scalar);
    }

    #[test]
    fn varchar_column_type() {
        let col = CommittableColumn::VarChar(alloc::vec![]);
        assert_eq!(col.column_type(), ColumnType::VarChar);
    }

    #[test]
    fn varbinary_column_type() {
        let col = CommittableColumn::VarBinary(alloc::vec![]);
        assert_eq!(col.column_type(), ColumnType::VarBinary);
    }

    #[test]
    fn timestamptz_column_type() {
        let col = CommittableColumn::TimestampTZ(
            PoSQLTimeUnit::Nanosecond,
            PoSQLTimeZone::utc(),
            &[],
        );
        assert_eq!(
            col.column_type(),
            ColumnType::TimestampTZ(PoSQLTimeUnit::Nanosecond, PoSQLTimeZone::utc())
        );
    }

    // --- From<&[T]> slice impls ---

    #[test]
    fn from_u8_slice_creates_uint8() {
        let data = [1u8, 2, 3];
        let col = CommittableColumn::from(data.as_slice());
        assert_eq!(col, CommittableColumn::Uint8(&data));
    }

    #[test]
    fn from_i8_slice_creates_tinyint() {
        let data = [1i8, -2, 3];
        let col = CommittableColumn::from(data.as_slice());
        assert_eq!(col, CommittableColumn::TinyInt(&data));
    }

    #[test]
    fn from_i16_slice_creates_smallint() {
        let data = [100i16, -200, 300];
        let col = CommittableColumn::from(data.as_slice());
        assert_eq!(col, CommittableColumn::SmallInt(&data));
    }

    #[test]
    fn from_i32_slice_creates_int() {
        let data = [1i32, 2, 3];
        let col = CommittableColumn::from(data.as_slice());
        assert_eq!(col, CommittableColumn::Int(&data));
    }

    #[test]
    fn from_i64_slice_creates_bigint() {
        let data = [1i64, 2, 3];
        let col = CommittableColumn::from(data.as_slice());
        assert_eq!(col, CommittableColumn::BigInt(&data));
    }

    #[test]
    fn from_i128_slice_creates_int128() {
        let data = [1i128, 2, 3];
        let col = CommittableColumn::from(data.as_slice());
        assert_eq!(col, CommittableColumn::Int128(&data));
    }

    #[test]
    fn from_bool_slice_creates_boolean() {
        let data = [true, false, true];
        let col = CommittableColumn::from(data.as_slice());
        assert_eq!(col, CommittableColumn::Boolean(&data));
    }

    // --- From<&Column<S>> ---

    #[test]
    fn from_column_boolean_creates_boolean() {
        let data = [true, false];
        let col = CommittableColumn::from(&Column::<TestScalar>::Boolean(&data));
        assert_eq!(col, CommittableColumn::Boolean(&data));
    }

    #[test]
    fn from_column_uint8_creates_uint8() {
        let data = [1u8, 2, 3];
        let col = CommittableColumn::from(&Column::<TestScalar>::Uint8(&data));
        assert_eq!(col, CommittableColumn::Uint8(&data));
    }

    #[test]
    fn from_column_tinyint_creates_tinyint() {
        let data = [1i8, -2];
        let col = CommittableColumn::from(&Column::<TestScalar>::TinyInt(&data));
        assert_eq!(col, CommittableColumn::TinyInt(&data));
    }

    #[test]
    fn from_column_smallint_creates_smallint() {
        let data = [100i16, 200];
        let col = CommittableColumn::from(&Column::<TestScalar>::SmallInt(&data));
        assert_eq!(col, CommittableColumn::SmallInt(&data));
    }

    #[test]
    fn from_column_int_creates_int() {
        let data = [10i32, 20, 30];
        let col = CommittableColumn::from(&Column::<TestScalar>::Int(&data));
        assert_eq!(col, CommittableColumn::Int(&data));
    }

    #[test]
    fn from_column_bigint_creates_bigint() {
        let data = [100i64, 200, 300];
        let col = CommittableColumn::from(&Column::<TestScalar>::BigInt(&data));
        assert_eq!(col, CommittableColumn::BigInt(&data));
    }

    #[test]
    fn from_column_int128_creates_int128() {
        let data = [1i128, -2];
        let col = CommittableColumn::from(&Column::<TestScalar>::Int128(&data));
        assert_eq!(col, CommittableColumn::Int128(&data));
    }

    #[test]
    fn from_column_timestamptz_creates_timestamptz() {
        let data = [1000i64, 2000];
        let col = CommittableColumn::from(&Column::<TestScalar>::TimestampTZ(
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            &data,
        ));
        assert_eq!(
            col,
            CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &data)
        );
    }

    #[test]
    fn from_column_scalar_creates_scalar() {
        let scalars = [TestScalar::from(1u64), TestScalar::from(2u64)];
        let col = CommittableColumn::from(&Column::Scalar(&scalars));
        assert_eq!(col.len(), 2);
        assert_eq!(col.column_type(), ColumnType::Scalar);
    }

    #[test]
    fn from_column_varchar_creates_varchar() {
        let strs = ["hello", "world"];
        let scalars = strs.map(TestScalar::from);
        let col = CommittableColumn::from(&Column::VarChar((&strs, &scalars)));
        assert_eq!(col.len(), 2);
        assert_eq!(col.column_type(), ColumnType::VarChar);
    }

    #[test]
    fn from_column_decimal75_creates_decimal75() {
        let prec = Precision::new(10).unwrap();
        let scalars = [TestScalar::from(1u64), TestScalar::from(2u64)];
        let col = CommittableColumn::from(&Column::Decimal75(prec, 3, &scalars));
        assert_eq!(col.len(), 2);
        assert_eq!(col.column_type(), ColumnType::Decimal75(prec, 3));
    }

    // --- From<Column<S>> (by value) ---

    #[test]
    fn from_owned_column_value_bigint() {
        let data = [10i64, 20];
        let col = CommittableColumn::from(Column::<TestScalar>::BigInt(&data));
        assert_eq!(col, CommittableColumn::BigInt(&data));
    }

    // --- From<&OwnedColumn<S>> ---

    #[test]
    fn from_owned_column_boolean() {
        let owned = OwnedColumn::<TestScalar>::Boolean(alloc::vec![true, false]);
        let col = CommittableColumn::from(&owned);
        assert_eq!(col, CommittableColumn::Boolean(&[true, false]));
    }

    #[test]
    fn from_owned_column_uint8() {
        let owned = OwnedColumn::<TestScalar>::Uint8(alloc::vec![1u8, 2, 3]);
        let col = CommittableColumn::from(&owned);
        assert_eq!(col, CommittableColumn::Uint8(&[1u8, 2, 3]));
    }

    #[test]
    fn from_owned_column_bigint() {
        let owned = OwnedColumn::<TestScalar>::BigInt(alloc::vec![100i64, 200]);
        let col = CommittableColumn::from(&owned);
        assert_eq!(col, CommittableColumn::BigInt(&[100i64, 200]));
    }

    #[test]
    fn from_owned_column_int128() {
        let owned = OwnedColumn::<TestScalar>::Int128(alloc::vec![1i128, 2]);
        let col = CommittableColumn::from(&owned);
        assert_eq!(col, CommittableColumn::Int128(&[1i128, 2]));
    }

    #[test]
    fn from_owned_column_varchar_empty() {
        let owned = OwnedColumn::<TestScalar>::VarChar(alloc::vec![]);
        let col = CommittableColumn::from(&owned);
        assert_eq!(col, CommittableColumn::VarChar(alloc::vec![]));
    }

    #[test]
    fn from_owned_column_varchar_nonempty() {
        let strings = alloc::vec![alloc::string::String::from("a"), alloc::string::String::from("b")];
        let owned = OwnedColumn::<TestScalar>::VarChar(strings.clone());
        let col = CommittableColumn::from(&owned);
        assert_eq!(col.len(), 2);
        assert_eq!(col.column_type(), ColumnType::VarChar);
    }

    #[test]
    fn from_owned_column_scalar() {
        let scalars = alloc::vec![TestScalar::from(1u64), TestScalar::from(2u64)];
        let owned = OwnedColumn::Scalar(scalars);
        let col = CommittableColumn::from(&owned);
        assert_eq!(col.len(), 2);
        assert_eq!(col.column_type(), ColumnType::Scalar);
    }

    #[test]
    fn from_owned_column_timestamptz() {
        let owned = OwnedColumn::<TestScalar>::TimestampTZ(
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            alloc::vec![1000i64, 2000],
        );
        let col = CommittableColumn::from(&owned);
        assert_eq!(
            col,
            CommittableColumn::TimestampTZ(
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                &[1000i64, 2000]
            )
        );
    }

    #[test]
    fn from_owned_column_varbinary_empty() {
        let owned = OwnedColumn::<TestScalar>::VarBinary(alloc::vec![]);
        let col = CommittableColumn::from(&owned);
        assert_eq!(col, CommittableColumn::VarBinary(alloc::vec![]));
    }

    #[test]
    fn from_owned_column_varbinary_nonempty() {
        let owned = OwnedColumn::<TestScalar>::VarBinary(alloc::vec![b"foo".to_vec()]);
        let col = CommittableColumn::from(&owned);
        assert_eq!(col.len(), 1);
        assert_eq!(col.column_type(), ColumnType::VarBinary);
    }

    // --- PartialEq ---

    #[test]
    fn same_boolean_columns_are_equal() {
        let a = CommittableColumn::Boolean(&[true, false]);
        let b = CommittableColumn::Boolean(&[true, false]);
        assert_eq!(a, b);
    }

    #[test]
    fn different_variants_are_not_equal() {
        let a = CommittableColumn::Boolean(&[true]);
        let b = CommittableColumn::Uint8(&[1u8]);
        assert_ne!(a, b);
    }
}

