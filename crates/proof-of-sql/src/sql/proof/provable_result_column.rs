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

    #[test]
    fn slice_result_column_reports_and_writes_encoded_length() {
        let values = [1_u64, 128, 16_384];
        let column = &values[..];
        let expected_len = values
            .iter()
            .map(ProvableResultElement::required_bytes)
            .sum();

        assert_eq!(column.num_bytes(values.len() as u64), expected_len);

        let mut out = vec![0_u8; expected_len];
        let written = column.write(&mut out, values.len() as u64);

        assert_eq!(written, expected_len);

        let mut offset = 0;
        for expected in values {
            let (decoded, bytes_read) =
                <u64 as ProvableResultElement>::decode(&out[offset..]).unwrap();
            assert_eq!(decoded, expected);
            offset += bytes_read;
        }
        assert_eq!(offset, written);
    }

    #[test]
    fn array_result_column_delegates_to_slice_encoding() {
        let values = [3_u64, 5, 8, 13];
        let expected_len = (&values[..]).num_bytes(values.len() as u64);

        assert_eq!(values.num_bytes(values.len() as u64), expected_len);

        let mut from_array = vec![0_u8; expected_len];
        let mut from_slice = vec![0_u8; expected_len];

        assert_eq!(
            values.write(&mut from_array, values.len() as u64),
            expected_len
        );
        assert_eq!(
            (&values[..]).write(&mut from_slice, values.len() as u64),
            expected_len
        );
        assert_eq!(from_array, from_slice);
    }

    #[test]
    #[should_panic]
    fn slice_result_column_rejects_mismatched_length() {
        let values = [1_u64, 2, 3];

        let _ = (&values[..]).num_bytes((values.len() + 1) as u64);
    }
}
