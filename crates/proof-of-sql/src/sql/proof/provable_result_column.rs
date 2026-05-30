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
        math::decimal::Precision,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::{test_scalar::TestScalar, ScalarExt},
    };
    use alloc::{vec, vec::Vec};

    fn encode_column(column: Column<'_, TestScalar>, length: u64) -> Vec<u8> {
        let mut out = vec![0; column.num_bytes(length)];
        let written = column.write(&mut out, length);
        assert_eq!(written, out.len());
        out
    }

    fn encode_slice<'a, T: ProvableResultElement<'a>>(column: &'a [T]) -> Vec<u8> {
        let length = column.len() as u64;
        let mut out = vec![0; column.num_bytes(length)];
        let written = column.write(&mut out, length);
        assert_eq!(written, out.len());
        out
    }

    fn assert_column_serializes_like_slice<'a, T: ProvableResultElement<'a>>(
        column: Column<'_, TestScalar>,
        values: &'a [T],
    ) {
        let length = values.len() as u64;
        assert_eq!(column.num_bytes(length), values.num_bytes(length));
        assert_eq!(encode_column(column, length), encode_slice(values));
    }

    #[test]
    fn columns_report_bytes_and_write_like_their_value_slices() {
        let bools = [true, false];
        assert_column_serializes_like_slice(Column::Boolean(&bools), &bools);

        let uints = [7_u8, 9];
        assert_column_serializes_like_slice(Column::Uint8(&uints), &uints);

        let tiny_ints = [-8_i8, 12];
        assert_column_serializes_like_slice(Column::TinyInt(&tiny_ints), &tiny_ints);

        let small_ints = [-16_i16, 24];
        assert_column_serializes_like_slice(Column::SmallInt(&small_ints), &small_ints);

        let ints = [-32_i32, 48];
        assert_column_serializes_like_slice(Column::Int(&ints), &ints);

        let big_ints = [-64_i64, 96];
        assert_column_serializes_like_slice(Column::BigInt(&big_ints), &big_ints);
        assert_column_serializes_like_slice(
            Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &big_ints),
            &big_ints,
        );

        let int128s = [-128_i128, 192];
        assert_column_serializes_like_slice(Column::Int128(&int128s), &int128s);

        let scalars = [TestScalar::from(3_u8), TestScalar::from(5_u8)];
        assert_column_serializes_like_slice(Column::Scalar(&scalars), &scalars);
        assert_column_serializes_like_slice(
            Column::Decimal75(Precision::new(9).unwrap(), 2, &scalars),
            &scalars,
        );

        let strings = ["proof", "sql"];
        let string_scalars = [TestScalar::from(strings[0]), TestScalar::from(strings[1])];
        assert_column_serializes_like_slice(Column::VarChar((&strings, &string_scalars)), &strings);

        let first_bytes = [1_u8, 2, 3];
        let second_bytes = [5_u8, 8];
        let bytes: [&[u8]; 2] = [&first_bytes, &second_bytes];
        let byte_scalars = [
            TestScalar::from_byte_slice_via_hash(&first_bytes),
            TestScalar::from_byte_slice_via_hash(&second_bytes),
        ];
        assert_column_serializes_like_slice(Column::VarBinary((&bytes, &byte_scalars)), &bytes);
    }

    #[test]
    fn arrays_delegate_result_column_serialization_to_slices() {
        let values = [1_i64, -2, 300];
        let length = values.len() as u64;
        let mut array_out = vec![0; values.num_bytes(length)];
        let mut slice_out = vec![0; (&values[..]).num_bytes(length)];

        let array_written = values.write(&mut array_out, length);
        let slice_written = (&values[..]).write(&mut slice_out, length);

        assert_eq!(array_written, slice_written);
        assert_eq!(array_out, slice_out);
    }
}
