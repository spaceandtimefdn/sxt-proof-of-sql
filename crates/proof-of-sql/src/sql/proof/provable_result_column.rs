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
        math::decimal::Precision,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::{test_scalar::TestScalar, ScalarExt},
    };
    use alloc::vec;

    fn assert_writes_exactly(column: impl ProvableResultColumn, length: u64) {
        let num_bytes = column.num_bytes(length);
        let mut out = vec![0_u8; num_bytes];
        assert_eq!(column.write(&mut out, length), num_bytes);
    }

    #[test]
    fn slice_and_array_result_columns_write_serialized_bytes() {
        let slice = &[123_i64, -456_i64][..];
        assert_writes_exactly(slice, 2);

        let array = [7_i16, -8_i16, 9_i16];
        assert_writes_exactly(array, 3);
    }

    #[test]
    #[should_panic(expected = "assertion `left == right` failed")]
    fn slice_result_columns_reject_length_mismatch() {
        let slice = &[1_i64][..];
        let _ = slice.num_bytes(2);
    }

    #[test]
    fn database_column_variants_write_serialized_bytes() {
        let booleans = [true, false];
        let uint8s = [1_u8, 2_u8];
        let tinyints = [-1_i8, 2_i8];
        let smallints = [-3_i16, 4_i16];
        let ints = [-5_i32, 6_i32];
        let bigints = [-7_i64, 8_i64];
        let int128s = [-9_i128, 10_i128];
        let scalars = [TestScalar::from(11_i64), TestScalar::from(12_i64)];
        let decimal_scalars = [TestScalar::from(13_i64), TestScalar::from(14_i64)];
        let strings = ["alpha", "beta"];
        let string_scalars = [TestScalar::from("alpha"), TestScalar::from("beta")];
        let bytes: [&[u8]; 2] = [&b"left"[..], &b"right"[..]];
        let byte_scalars = [
            TestScalar::from_byte_slice_via_hash(bytes[0]),
            TestScalar::from_byte_slice_via_hash(bytes[1]),
        ];
        let timestamps = [1_700_000_000_i64, 1_700_000_001_i64];

        let columns = [
            Column::Boolean(&booleans),
            Column::Uint8(&uint8s),
            Column::TinyInt(&tinyints),
            Column::SmallInt(&smallints),
            Column::Int(&ints),
            Column::BigInt(&bigints),
            Column::Int128(&int128s),
            Column::Decimal75(Precision::new(10).unwrap(), 2, &decimal_scalars),
            Column::Scalar(&scalars),
            Column::VarChar((&strings, &string_scalars)),
            Column::VarBinary((&bytes, &byte_scalars)),
            Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &timestamps),
        ];

        for column in columns {
            assert_writes_exactly(column, 2);
        }
    }
}
