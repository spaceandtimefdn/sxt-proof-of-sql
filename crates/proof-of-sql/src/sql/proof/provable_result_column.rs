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
    use crate::sql::proof::decode_multiple_elements;
    use alloc::vec;

    #[test]
    fn slice_column_reports_and_writes_encoded_bytes() {
        let column = [121_i64, -345_i64, 666_i64];
        let column_slice = &column[..];
        let expected_len = column
            .iter()
            .map(ProvableResultElement::required_bytes)
            .sum();
        let mut out = vec![0_u8; expected_len];

        assert_eq!(column_slice.num_bytes(column.len() as u64), expected_len);
        assert_eq!(
            column_slice.write(&mut out, column.len() as u64),
            expected_len
        );

        let (decoded, bytes_read) = decode_multiple_elements::<i64>(&out, column.len()).unwrap();
        assert_eq!(decoded, column);
        assert_eq!(bytes_read, expected_len);
    }

    #[test]
    fn array_column_delegates_to_slice_serialization() {
        let column = ["alpha", "", "omega"];
        let expected_len = column
            .iter()
            .map(ProvableResultElement::required_bytes)
            .sum();
        let mut out = vec![0_u8; expected_len];

        assert_eq!(column.num_bytes(column.len() as u64), expected_len);
        assert_eq!(column.write(&mut out, column.len() as u64), expected_len);

        let (decoded, bytes_read) = decode_multiple_elements::<&str>(&out, column.len()).unwrap();
        assert_eq!(decoded, column);
        assert_eq!(bytes_read, expected_len);
    }
}
