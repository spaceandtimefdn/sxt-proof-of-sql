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
    use crate::base::scalar::test_scalar::TestScalar;
    use crate::base::database::Column;

    #[test]
    fn bool_slice_num_bytes_equals_length() {
        let data = [true, false, true];
        assert_eq!(data.as_slice().num_bytes(3), 3);
    }

    #[test]
    fn bool_slice_num_bytes_zero_for_empty() {
        let data: &[bool] = &[];
        assert_eq!(data.num_bytes(0), 0);
    }

    #[test]
    fn bool_slice_write_encodes_true_as_one() {
        let data = [true];
        let mut out = alloc::vec![0u8; 1];
        data.as_slice().write(&mut out, 1);
        assert_eq!(out[0], 1);
    }

    #[test]
    fn bool_slice_write_encodes_false_as_zero() {
        let data = [false];
        let mut out = alloc::vec![0u8; 1];
        data.as_slice().write(&mut out, 1);
        assert_eq!(out[0], 0);
    }

    #[test]
    fn bool_slice_write_returns_written_count() {
        let data = [true, false];
        let mut out = alloc::vec![0u8; 2];
        let written = data.as_slice().write(&mut out, 2);
        assert_eq!(written, 2);
    }

    #[test]
    fn bigint_column_num_bytes_is_correct() {
        let data = alloc::vec![1i64, 2, 3];
        let col = Column::<TestScalar>::BigInt(&data);
        // BigInt uses VarInt encoding which is variable size
        let nb = col.num_bytes(3);
        assert!(nb > 0);
    }

    #[test]
    fn boolean_column_num_bytes_equals_length() {
        let data = alloc::vec![true, false, true];
        let col = Column::<TestScalar>::Boolean(&data);
        assert_eq!(col.num_bytes(3), 3);
    }

    #[test]
    fn boolean_column_write_fills_output() {
        let data = alloc::vec![true];
        let col = Column::<TestScalar>::Boolean(&data);
        let mut out = alloc::vec![0u8; 1];
        let written = col.write(&mut out, 1);
        assert_eq!(written, 1);
        assert_eq!(out[0], 1);
    }
}
