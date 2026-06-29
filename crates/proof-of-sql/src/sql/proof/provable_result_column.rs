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
        scalar::test_scalar::TestScalar,
    };

    fn encoded_column(column: Column<'_, TestScalar>, length: u64) -> Vec<u8> {
        let mut out = vec![0; column.num_bytes(length)];
        let bytes_written = column.write(&mut out, length);
        assert_eq!(bytes_written, out.len());
        out
    }

    fn encoded_slice<'a, T: ProvableResultElement<'a>>(slice: &[T], length: u64) -> Vec<u8> {
        let mut out = vec![0; slice.num_bytes(length)];
        let bytes_written = slice.write(&mut out, length);
        assert_eq!(bytes_written, out.len());
        out
    }

    #[test]
    fn we_can_write_array_columns_through_slice_delegation() {
        let values = [1_i16, -2, 3];
        let slice = &values[..];
        let mut array_out = vec![0; values.num_bytes(values.len() as u64)];
        let mut slice_out = vec![0; slice.num_bytes(slice.len() as u64)];

        assert_eq!(array_out.len(), slice_out.len());
        assert_eq!(
            values.write(&mut array_out, values.len() as u64),
            slice.write(&mut slice_out, slice.len() as u64)
        );
        assert_eq!(array_out, slice_out);
    }

    #[test]
    fn we_can_write_integer_column_variants() {
        assert_eq!(
            encoded_column(Column::Uint8(&[1, 2, 3]), 3),
            encoded_slice(&[1_u8, 2, 3], 3)
        );
        assert_eq!(
            encoded_column(Column::TinyInt(&[-1, 2, -3]), 3),
            encoded_slice(&[-1_i8, 2, -3], 3)
        );
        assert_eq!(
            encoded_column(Column::SmallInt(&[-10, 20]), 2),
            encoded_slice(&[-10_i16, 20], 2)
        );
        assert_eq!(
            encoded_column(Column::Int(&[-100, 200]), 2),
            encoded_slice(&[-100_i32, 200], 2)
        );
        assert_eq!(
            encoded_column(Column::BigInt(&[-1000, 2000]), 2),
            encoded_slice(&[-1000_i64, 2000], 2)
        );
        assert_eq!(
            encoded_column(Column::Int128(&[-10000, 20000]), 2),
            encoded_slice(&[-10000_i128, 20000], 2)
        );
    }

    #[test]
    fn we_can_write_boolean_and_text_column_variants() {
        let strings = ["alpha", "beta"];
        let string_hashes = [TestScalar::from(1_u64), TestScalar::from(2_u64)];
        let bytes: [&[u8]; 2] = [b"one".as_slice(), b"two".as_slice()];
        let byte_hashes = [TestScalar::from(3_u64), TestScalar::from(4_u64)];

        assert_eq!(
            encoded_column(Column::Boolean(&[true, false, true]), 3),
            encoded_slice(&[true, false, true], 3)
        );
        assert_eq!(
            encoded_column(Column::VarChar((&strings, &string_hashes)), 2),
            encoded_slice(&strings, 2)
        );
        assert_eq!(
            encoded_column(Column::VarBinary((&bytes, &byte_hashes)), 2),
            encoded_slice(&bytes, 2)
        );
    }

    #[test]
    fn we_can_write_scalar_backed_column_variants() {
        let scalars = [TestScalar::from(7_u64), TestScalar::from(11_u64)];
        let precision = Precision::new(9).unwrap();

        assert_eq!(
            encoded_column(Column::Scalar(&scalars), 2),
            encoded_slice(&scalars, 2)
        );
        assert_eq!(
            encoded_column(Column::Decimal75(precision, 2, &scalars), 2),
            encoded_slice(&scalars, 2)
        );
    }

    #[test]
    fn we_can_write_timestamp_columns_as_bigints() {
        let timestamps = [1_700_000_000_i64, 1_700_000_001_i64];

        assert_eq!(
            encoded_column(
                Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &timestamps),
                2
            ),
            encoded_slice(&timestamps, 2)
        );
    }
}
