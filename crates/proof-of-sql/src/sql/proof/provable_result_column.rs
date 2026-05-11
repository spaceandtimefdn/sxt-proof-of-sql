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
    use super::ProvableResultColumn;
    use crate::base::{
        database::Column,
        math::decimal::Precision,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::test_scalar::TestScalar,
    };
    use alloc::vec::Vec;

    fn assert_column_encodes_like_slice<'a, T>(column: Column<'a, TestScalar>, expected: &'a [T])
    where
        T: crate::sql::proof::ProvableResultElement<'a>,
    {
        let length = expected.len() as u64;
        let expected_len = expected.num_bytes(length);
        assert_eq!(column.num_bytes(length), expected_len);

        let mut column_bytes = vec![0; expected_len];
        let mut expected_bytes = vec![0; expected_len];

        assert_eq!(column.write(&mut column_bytes, length), expected_len);
        assert_eq!(expected.write(&mut expected_bytes, length), expected_len);
        assert_eq!(column_bytes, expected_bytes);
    }

    #[test]
    fn columns_delegate_result_serialization_to_their_values() {
        let booleans = [true, false];
        assert_column_encodes_like_slice(Column::Boolean(&booleans), &booleans);

        let uints = [1_u8, 2];
        assert_column_encodes_like_slice(Column::Uint8(&uints), &uints);

        let tinyints = [-1_i8, 2];
        assert_column_encodes_like_slice(Column::TinyInt(&tinyints), &tinyints);

        let smallints = [-10_i16, 20];
        assert_column_encodes_like_slice(Column::SmallInt(&smallints), &smallints);

        let ints = [-100_i32, 200];
        assert_column_encodes_like_slice(Column::Int(&ints), &ints);

        let bigints = [-1_000_i64, 2_000];
        assert_column_encodes_like_slice(Column::BigInt(&bigints), &bigints);
        assert_column_encodes_like_slice(
            Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &bigints),
            &bigints,
        );

        let int128s = [-10_000_i128, 20_000];
        assert_column_encodes_like_slice(Column::Int128(&int128s), &int128s);

        let scalars = [TestScalar::from(11), TestScalar::from(22)];
        assert_column_encodes_like_slice(Column::Scalar(&scalars), &scalars);
        assert_column_encodes_like_slice(
            Column::Decimal75(Precision::new(75).unwrap(), 0, &scalars),
            &scalars,
        );

        let strings = ["alpha", "beta"];
        let string_scalars = strings
            .iter()
            .map(|value| TestScalar::from(*value))
            .collect::<Vec<_>>();
        assert_column_encodes_like_slice(Column::VarChar((&strings, &string_scalars)), &strings);

        let bytes = [b"alpha".as_slice(), b"beta".as_slice()];
        let byte_scalars = bytes
            .iter()
            .map(|value| TestScalar::from_le_bytes_mod_order(value))
            .collect::<Vec<_>>();
        assert_column_encodes_like_slice(Column::VarBinary((&bytes, &byte_scalars)), &bytes);
    }

    #[test]
    fn arrays_delegate_result_serialization_to_their_slices() {
        let values = [1_i32, 2, 3];
        let length = values.len() as u64;
        let expected_len = (&values[..]).num_bytes(length);
        let mut array_bytes = vec![0; expected_len];
        let mut slice_bytes = vec![0; expected_len];

        assert_eq!(values.num_bytes(length), expected_len);
        assert_eq!(values.write(&mut array_bytes, length), expected_len);
        assert_eq!((&values[..]).write(&mut slice_bytes, length), expected_len);
        assert_eq!(array_bytes, slice_bytes);
    }
}
