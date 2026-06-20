use crate::{
    base::{database::Column, scalar::Scalar},
    sql::proof::ProvableResultElement,
};

/// Interface for serializing an intermediate result column
pub trait ProvableResultColumn {
    /// The number of bytes of the serialized result column
    fn num_bytes(&self, length: u64) -> usize;

    /// Serialize the result column
    fn write(&self, out: &mut [u8], length: u64) -> usize;
}

impl<'a, T: ProvableResultElement<'a>> ProvableResultColumn for &[T] {
    fn num_bytes(&self, length: u64) -> usize {
        assert_eq!(self.len() as u64, length);
        self.iter().map(ProvableResultElement::required_bytes).sum()
    }

    fn write(&self, out: &mut [u8], length: u64) -> usize {
        let mut res = 0;
        for i in 0..length {
            let index: usize = usize::try_from(i).expect("Index out of bounds");
            res += self[index].encode(&mut out[res..]);
        }
        res
    }
}

impl<S: Scalar> ProvableResultColumn for Column<'_, S> {
    fn num_bytes(&self, length: u64) -> usize {
        match self {
            Column::Boolean(col) => col.num_bytes(length),
            Column::Uint8(col) => col.num_bytes(length),
            Column::TinyInt(col) => col.num_bytes(length),
            Column::SmallInt(col) => col.num_bytes(length),
            Column::Int(col) => col.num_bytes(length),
            Column::BigInt(col) | Column::TimestampTZ(_, _, col) => col.num_bytes(length),
            Column::Int128(col) => col.num_bytes(length),
            Column::Decimal75(_, _, col) | Column::Scalar(col) => col.num_bytes(length),
            Column::VarChar((col, _)) => col.num_bytes(length),
            Column::VarBinary((col, _)) => col.num_bytes(length),
        }
    }

    fn write(&self, out: &mut [u8], length: u64) -> usize {
        match self {
            Column::Boolean(col) => col.write(out, length),
            Column::Uint8(col) => col.write(out, length),
            Column::TinyInt(col) => col.write(out, length),
            Column::SmallInt(col) => col.write(out, length),
            Column::Int(col) => col.write(out, length),
            Column::BigInt(col) | Column::TimestampTZ(_, _, col) => col.write(out, length),
            Column::Int128(col) => col.write(out, length),
            Column::Decimal75(_, _, col) | Column::Scalar(col) => col.write(out, length),
            Column::VarChar((col, _)) => col.write(out, length),
            Column::VarBinary((col, _)) => col.write(out, length),
        }
    }
}

impl<'a, T: ProvableResultElement<'a>, const N: usize> ProvableResultColumn for [T; N] {
    fn num_bytes(&self, length: u64) -> usize {
        (&self[..]).num_bytes(length)
    }

    fn write(&self, out: &mut [u8], length: u64) -> usize {
        (&self[..]).write(out, length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{
        database::Column,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::test_scalar::TestScalar,
    };
    use alloc::vec;

    fn assert_column_serializes_like_slice<'a, T>(column: Column<'a, TestScalar>, values: &'a [T])
    where
        T: ProvableResultElement<'a>,
    {
        let length = values.len() as u64;
        let expected_len = values.num_bytes(length);
        let mut expected = vec![0_u8; expected_len];
        let expected_written = values.write(&mut expected, length);

        assert_eq!(column.num_bytes(length), expected_len);
        let mut actual = vec![0_u8; expected_len];
        assert_eq!(column.write(&mut actual, length), expected_written);
        assert_eq!(actual, expected);
    }

    #[test]
    fn column_delegates_serialization_for_fixed_width_variants() {
        let booleans = [true, false, true];
        assert_column_serializes_like_slice(Column::Boolean(&booleans), &booleans);

        let uint8s = [1_u8, 127, 255];
        assert_column_serializes_like_slice(Column::Uint8(&uint8s), &uint8s);

        let tinyints = [-1_i8, 0, 42];
        assert_column_serializes_like_slice(Column::TinyInt(&tinyints), &tinyints);

        let smallints = [-300_i16, 0, 300];
        assert_column_serializes_like_slice(Column::SmallInt(&smallints), &smallints);

        let ints = [-10_000_i32, 0, 10_000];
        assert_column_serializes_like_slice(Column::Int(&ints), &ints);

        let bigints = [-1_000_000_i64, 0, 1_000_000];
        assert_column_serializes_like_slice(Column::BigInt(&bigints), &bigints);

        let int128s = [-1_000_000_i128, 0, 1_000_000];
        assert_column_serializes_like_slice(Column::Int128(&int128s), &int128s);

        let timestamps = [1_700_000_000_i64, 1_700_000_001, 1_700_000_002];
        assert_column_serializes_like_slice(
            Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &timestamps),
            &timestamps,
        );
    }

    #[test]
    fn column_delegates_serialization_for_scalar_and_variable_width_variants() {
        let scalars: [TestScalar; 3] = [7_i64.into(), 8_i64.into(), (-9_i64).into()];
        assert_column_serializes_like_slice(Column::Scalar(&scalars), &scalars);
        assert_column_serializes_like_slice(
            Column::Decimal75(
                crate::base::math::decimal::Precision::new(12).unwrap(),
                2,
                &scalars,
            ),
            &scalars,
        );

        let strings = ["alice", "bob", "carol"];
        assert_column_serializes_like_slice(Column::VarChar((&strings, &scalars)), &strings);

        let raw_a = [1_u8, 2, 3];
        let raw_b = [4_u8, 5];
        let raw_c = [6_u8, 7, 8, 9];
        let binaries: [&[u8]; 3] = [&raw_a, &raw_b, &raw_c];
        assert_column_serializes_like_slice(Column::VarBinary((&binaries, &scalars)), &binaries);
    }

    #[test]
    fn array_impl_delegates_to_slice_impl() {
        let values = [1_u8, 127, 128, 255];
        let expected_len = (&values[..]).num_bytes(values.len() as u64);
        let mut expected = vec![0_u8; expected_len];
        let expected_written = (&values[..]).write(&mut expected, values.len() as u64);

        assert_eq!(values.num_bytes(values.len() as u64), expected_len);
        let mut actual = vec![0_u8; expected_len];
        assert_eq!(
            values.write(&mut actual, values.len() as u64),
            expected_written
        );
        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic(expected = "assertion `left == right` failed")]
    fn slice_num_bytes_panics_when_length_mismatches() {
        let values = [1_u8, 2, 3];
        let _ = (&values[..]).num_bytes(2);
    }
}
