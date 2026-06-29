use super::{decode_and_convert, decode_multiple_elements, ProvableResultColumn, QueryError};
use crate::base::{
    database::{Column, ColumnField, ColumnType, OwnedColumn, OwnedTable, Table},
    polynomial::compute_evaluation_vector,
    scalar::{Scalar, ScalarExt},
};
use alloc::{vec, vec::Vec};
use num_traits::Zero;
use serde::{Deserialize, Serialize};

/// An intermediate form of a query result that can be transformed
/// to either the finalized query result form or a query error
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ProvableQueryResult {
    num_columns: u64,
    pub(crate) table_length: u64,
    data: Vec<u8>,
}

// TODO: Handle truncation properly. The `expect(clippy::cast_possible_truncation)` is a temporary fix and should be replaced with proper logic to manage possible truncation scenarios.
impl ProvableQueryResult {
    #[expect(clippy::cast_possible_truncation)]
    /// The number of columns in the result
    #[must_use]
    pub fn num_columns(&self) -> usize {
        self.num_columns as usize
    }
    /// A mutable reference to the number of columns in the result. Because the struct is deserialized from untrusted data, it
    /// cannot maintain any invariant on its data members; hence, this function is available to allow for easy manipulation for testing.
    #[cfg(test)]
    pub fn num_columns_mut(&mut self) -> &mut u64 {
        &mut self.num_columns
    }

    #[expect(clippy::cast_possible_truncation)]
    /// The number of rows in the result
    #[must_use]
    pub fn table_length(&self) -> usize {
        self.table_length as usize
    }
    /// A mutable reference to the underlying encoded data of the result. Because the struct is deserialized from untrusted data, it
    /// cannot maintain any invariant on its data members; hence, this function is available to allow for easy manipulation for testing.
    #[cfg(test)]
    pub fn data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }
    /// This function is available to allow for easy creation for testing.
    #[cfg(test)]
    #[must_use]
    pub fn new_from_raw_data(num_columns: u64, table_length: u64, data: Vec<u8>) -> Self {
        Self {
            num_columns,
            table_length,
            data,
        }
    }

    /// Form intermediate query result from index rows and result columns
    /// # Panics
    ///
    /// Will panic if `table_length` is somehow larger than the length of some column
    /// which should never happen.
    #[must_use]
    pub fn new<'a, S: Scalar>(table_length: u64, columns: &'a [Column<'a, S>]) -> Self {
        assert!(columns
            .iter()
            .all(|column| table_length == column.len() as u64));
        let mut sz = 0;
        for col in columns {
            sz += col.num_bytes(table_length);
        }
        let mut data = vec![0u8; sz];
        let mut sz = 0;
        for col in columns {
            sz += col.write(&mut data[sz..], table_length);
        }
        ProvableQueryResult {
            num_columns: columns.len() as u64,
            table_length,
            data,
        }
    }

    #[expect(clippy::cast_possible_truncation)]
    /// Given an evaluation vector, compute the evaluation of the intermediate result
    /// columns as spare multilinear extensions
    ///
    /// # Panics
    /// This function will panic if the length of `evaluation_point` does not match `self.num_columns`.
    /// It will also panic if the `data` array is not properly formatted for the expected column types.
    pub fn evaluate<S: Scalar>(
        &self,
        evaluation_point: &[S],
        output_length: usize,
        column_result_fields: &[ColumnField],
    ) -> Result<Vec<S>, QueryError> {
        if self.num_columns as usize != column_result_fields.len() {
            return Err(QueryError::InvalidColumnCount);
        }
        let mut evaluation_vec = vec![Zero::zero(); output_length];
        compute_evaluation_vector(&mut evaluation_vec, evaluation_point);
        let mut offset: usize = 0;
        let mut res = Vec::with_capacity(self.num_columns as usize);

        for field in column_result_fields {
            let mut val = S::zero();
            for entry in evaluation_vec.iter().take(output_length) {
                let (x, sz) = match field.data_type() {
                    ColumnType::Boolean => decode_and_convert::<bool, S>(&self.data[offset..]),
                    ColumnType::Uint8 => decode_and_convert::<u8, S>(&self.data[offset..]),
                    ColumnType::TinyInt => decode_and_convert::<i8, S>(&self.data[offset..]),
                    ColumnType::SmallInt => decode_and_convert::<i16, S>(&self.data[offset..]),
                    ColumnType::Int => decode_and_convert::<i32, S>(&self.data[offset..]),
                    ColumnType::BigInt => decode_and_convert::<i64, S>(&self.data[offset..]),
                    ColumnType::Int128 => decode_and_convert::<i128, S>(&self.data[offset..]),
                    ColumnType::Decimal75(_, _) | ColumnType::Scalar => {
                        decode_and_convert::<S, S>(&self.data[offset..])
                    }

                    ColumnType::VarChar => decode_and_convert::<&str, S>(&self.data[offset..]),
                    ColumnType::VarBinary => {
                        let (raw_bytes, used) =
                            decode_and_convert::<&[u8], &[u8]>(&self.data[offset..])?;
                        let x = S::from_byte_slice_via_hash(raw_bytes);
                        Ok((x, used))
                    }
                    ColumnType::TimestampTZ(_, _) => {
                        decode_and_convert::<i64, S>(&self.data[offset..])
                    }
                }?;
                val += *entry * x;
                offset += sz;
            }
            res.push(val);
        }
        if offset != self.data.len() {
            return Err(QueryError::MiscellaneousEvaluationError);
        }

        Ok(res)
    }

    #[expect(
        clippy::missing_panics_doc,
        reason = "Assertions ensure preconditions are met, eliminating the possibility of panic."
    )]
    /// Convert the intermediate query result into a final query result
    ///
    /// The result is essentially an `OwnedTable` type.
    pub fn to_owned_table<S: Scalar>(
        &self,
        column_result_fields: &[ColumnField],
    ) -> Result<OwnedTable<S>, QueryError> {
        if column_result_fields.len() != self.num_columns() {
            return Err(QueryError::InvalidColumnCount);
        }

        let n = self.table_length();
        let mut offset: usize = 0;

        let owned_table = OwnedTable::try_new(
            column_result_fields
                .iter()
                .map(|field| match field.data_type() {
                    ColumnType::Boolean => {
                        let (col, num_read) = decode_multiple_elements(&self.data[offset..], n)?;
                        offset += num_read;
                        Ok((field.name(), OwnedColumn::Boolean(col)))
                    }
                    ColumnType::Uint8 => {
                        let (col, num_read) = decode_multiple_elements(&self.data[offset..], n)?;
                        offset += num_read;
                        Ok((field.name(), OwnedColumn::Uint8(col)))
                    }
                    ColumnType::TinyInt => {
                        let (col, num_read) = decode_multiple_elements(&self.data[offset..], n)?;
                        offset += num_read;
                        Ok((field.name(), OwnedColumn::TinyInt(col)))
                    }
                    ColumnType::SmallInt => {
                        let (col, num_read) = decode_multiple_elements(&self.data[offset..], n)?;
                        offset += num_read;
                        Ok((field.name(), OwnedColumn::SmallInt(col)))
                    }
                    ColumnType::Int => {
                        let (col, num_read) = decode_multiple_elements(&self.data[offset..], n)?;
                        offset += num_read;
                        Ok((field.name(), OwnedColumn::Int(col)))
                    }
                    ColumnType::BigInt => {
                        let (col, num_read) = decode_multiple_elements(&self.data[offset..], n)?;
                        offset += num_read;
                        Ok((field.name(), OwnedColumn::BigInt(col)))
                    }
                    ColumnType::Int128 => {
                        let (col, num_read) = decode_multiple_elements(&self.data[offset..], n)?;
                        offset += num_read;
                        Ok((field.name(), OwnedColumn::Int128(col)))
                    }
                    ColumnType::VarChar => {
                        let (col, num_read) = decode_multiple_elements(&self.data[offset..], n)?;
                        offset += num_read;
                        Ok((field.name(), OwnedColumn::VarChar(col)))
                    }
                    ColumnType::VarBinary => {
                        // Manually specify the item type: `&[u8]`
                        let (decoded_slices, num_read) =
                            decode_multiple_elements::<&[u8]>(&self.data[offset..], n)?;
                        offset += num_read;

                        // Convert those slices to owned `Vec<u8>`
                        let col_vec = decoded_slices.into_iter().map(<[u8]>::to_vec).collect();

                        Ok((field.name(), OwnedColumn::VarBinary(col_vec)))
                    }
                    ColumnType::Scalar => {
                        let (col, num_read) = decode_multiple_elements(&self.data[offset..], n)?;
                        offset += num_read;
                        Ok((field.name(), OwnedColumn::Scalar(col)))
                    }
                    ColumnType::Decimal75(precision, scale) => {
                        let (col, num_read) = decode_multiple_elements(&self.data[offset..], n)?;
                        offset += num_read;
                        Ok((field.name(), OwnedColumn::Decimal75(precision, scale, col)))
                    }
                    ColumnType::TimestampTZ(tu, tz) => {
                        let (col, num_read) = decode_multiple_elements(&self.data[offset..], n)?;
                        offset += num_read;
                        Ok((field.name(), OwnedColumn::TimestampTZ(tu, tz, col)))
                    }
                })
                .collect::<Result<_, QueryError>>()?,
        )?;

        assert_eq!(offset, self.data.len());
        assert_eq!(owned_table.num_columns(), self.num_columns());

        Ok(owned_table)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{
        database::{
            owned_table_utility::{
                bigint, boolean, decimal75, int, int128, owned_table, scalar, smallint,
                timestamptz, tinyint, uint8, varbinary, varchar,
            },
            table_utility::{borrowed_bigint, borrowed_boolean, table},
        },
        math::decimal::Precision,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::test_scalar::TestScalar,
    };
    use bumpalo::Bump;

    fn mixed_fields() -> Vec<ColumnField> {
        vec![
            ColumnField::new("flag".into(), ColumnType::Boolean),
            ColumnField::new("uint8_col".into(), ColumnType::Uint8),
            ColumnField::new("tinyint_col".into(), ColumnType::TinyInt),
            ColumnField::new("smallint_col".into(), ColumnType::SmallInt),
            ColumnField::new("int_col".into(), ColumnType::Int),
            ColumnField::new("bigint_col".into(), ColumnType::BigInt),
            ColumnField::new("int128_col".into(), ColumnType::Int128),
            ColumnField::new("varchar_col".into(), ColumnType::VarChar),
            ColumnField::new("varbinary_col".into(), ColumnType::VarBinary),
            ColumnField::new("scalar_col".into(), ColumnType::Scalar),
            ColumnField::new(
                "decimal_col".into(),
                ColumnType::Decimal75(Precision::new(75).unwrap(), 2),
            ),
            ColumnField::new(
                "timestamp_col".into(),
                ColumnType::TimestampTZ(PoSQLTimeUnit::Millisecond, PoSQLTimeZone::utc()),
            ),
        ]
    }

    #[test]
    fn we_can_evaluate_all_column_types_without_arrow() {
        let bool_values = [true];
        let uint8_values = [7_u8];
        let tinyint_values = [-2_i8];
        let smallint_values = [-3_i16];
        let int_values = [-4_i32];
        let bigint_values = [-5_i64];
        let int128_values = [-6_i128];
        let varchar_values = ["abc"];
        let varchar_scalars = [TestScalar::from("abc")];
        let raw_bytes = [b"raw".as_slice()];
        let binary_scalars = [TestScalar::from_byte_slice_via_hash(raw_bytes[0])];
        let scalar_values = [TestScalar::from(11_u64)];
        let decimal_values = [TestScalar::from(13_u64)];
        let timestamp_values = [1_234_i64];
        let columns = [
            Column::Boolean(&bool_values),
            Column::Uint8(&uint8_values),
            Column::TinyInt(&tinyint_values),
            Column::SmallInt(&smallint_values),
            Column::Int(&int_values),
            Column::BigInt(&bigint_values),
            Column::Int128(&int128_values),
            Column::VarChar((&varchar_values, &varchar_scalars)),
            Column::VarBinary((&raw_bytes, &binary_scalars)),
            Column::Scalar(&scalar_values),
            Column::Decimal75(Precision::new(75).unwrap(), 2, &decimal_values),
            Column::TimestampTZ(
                PoSQLTimeUnit::Millisecond,
                PoSQLTimeZone::utc(),
                &timestamp_values,
            ),
        ];
        let result = ProvableQueryResult::new(1, &columns);

        let evaluations = result
            .evaluate::<TestScalar>(&[], 1, &mixed_fields())
            .unwrap();

        assert_eq!(
            evaluations,
            vec![
                TestScalar::from(1_u64),
                TestScalar::from(7_u64),
                -TestScalar::from(2_u64),
                -TestScalar::from(3_u64),
                -TestScalar::from(4_u64),
                -TestScalar::from(5_u64),
                -TestScalar::from(6_u64),
                TestScalar::from("abc"),
                TestScalar::from_byte_slice_via_hash(raw_bytes[0]),
                TestScalar::from(11_u64),
                TestScalar::from(13_u64),
                TestScalar::from(1_234_u64),
            ]
        );
    }

    #[test]
    fn we_can_convert_all_column_types_to_owned_table_without_arrow() {
        let bool_values = [true, false];
        let uint8_values = [7_u8, 8];
        let tinyint_values = [-2_i8, 3];
        let smallint_values = [-3_i16, 4];
        let int_values = [-4_i32, 5];
        let bigint_values = [-5_i64, 6];
        let int128_values = [-6_i128, 7];
        let varchar_values = ["abc", "de"];
        let varchar_scalars = [TestScalar::from("abc"), TestScalar::from("de")];
        let raw_bytes = [b"raw".as_slice(), b"bytes".as_slice()];
        let binary_scalars = [
            TestScalar::from_byte_slice_via_hash(raw_bytes[0]),
            TestScalar::from_byte_slice_via_hash(raw_bytes[1]),
        ];
        let scalar_values = [TestScalar::from(11_u64), TestScalar::from(12_u64)];
        let decimal_values = [TestScalar::from(13_u64), TestScalar::from(14_u64)];
        let timestamp_values = [1_234_i64, 5_678];
        let columns = [
            Column::Boolean(&bool_values),
            Column::Uint8(&uint8_values),
            Column::TinyInt(&tinyint_values),
            Column::SmallInt(&smallint_values),
            Column::Int(&int_values),
            Column::BigInt(&bigint_values),
            Column::Int128(&int128_values),
            Column::VarChar((&varchar_values, &varchar_scalars)),
            Column::VarBinary((&raw_bytes, &binary_scalars)),
            Column::Scalar(&scalar_values),
            Column::Decimal75(Precision::new(75).unwrap(), 2, &decimal_values),
            Column::TimestampTZ(
                PoSQLTimeUnit::Millisecond,
                PoSQLTimeZone::utc(),
                &timestamp_values,
            ),
        ];
        let result = ProvableQueryResult::new(2, &columns);
        let expected = owned_table::<TestScalar>([
            boolean("flag", bool_values),
            uint8("uint8_col", uint8_values),
            tinyint("tinyint_col", tinyint_values),
            smallint("smallint_col", smallint_values),
            int("int_col", int_values),
            bigint("bigint_col", bigint_values),
            int128("int128_col", int128_values),
            varchar("varchar_col", varchar_values),
            varbinary(
                "varbinary_col",
                [raw_bytes[0].to_vec(), raw_bytes[1].to_vec()],
            ),
            scalar("scalar_col", scalar_values),
            decimal75("decimal_col", 75, 2, decimal_values),
            timestamptz(
                "timestamp_col",
                PoSQLTimeUnit::Millisecond,
                PoSQLTimeZone::utc(),
                timestamp_values,
            ),
        ]);

        assert_eq!(
            result
                .to_owned_table::<TestScalar>(&mixed_fields())
                .unwrap(),
            expected
        );
    }

    #[test]
    fn we_can_create_result_from_table_without_arrow() {
        let alloc = Bump::new();
        let source = table::<TestScalar>([
            borrowed_bigint("amount", [10_i64, 12], &alloc),
            borrowed_boolean("flag", [true, false], &alloc),
        ]);
        let fields = source.schema();

        let result = ProvableQueryResult::from(source);
        let expected = owned_table::<TestScalar>([
            bigint("amount", [10_i64, 12]),
            boolean("flag", [true, false]),
        ]);

        assert_eq!(result.num_columns(), 2);
        assert_eq!(result.table_length(), 2);
        assert_eq!(
            result.to_owned_table::<TestScalar>(&fields).unwrap(),
            expected
        );
    }

    #[test]
    fn conversion_and_evaluation_report_invalid_encoded_shapes() {
        let columns: [Column<TestScalar>; 1] = [Column::BigInt(&[10_i64])];
        let mut result = ProvableQueryResult::new(1, &columns);
        let fields = [ColumnField::new("amount".into(), ColumnType::BigInt)];

        *result.num_columns_mut() = 2;
        assert!(matches!(
            result.evaluate::<TestScalar>(&[], 1, &fields),
            Err(QueryError::InvalidColumnCount)
        ));
        assert!(matches!(
            result.to_owned_table::<TestScalar>(&fields),
            Err(QueryError::InvalidColumnCount)
        ));

        *result.num_columns_mut() = 1;
        result.data_mut().push(1);
        assert!(matches!(
            result.evaluate::<TestScalar>(&[], 1, &fields),
            Err(QueryError::MiscellaneousEvaluationError)
        ));
    }
}

impl<S: Scalar> From<Table<'_, S>> for ProvableQueryResult {
    fn from(table: Table<S>) -> Self {
        let num_rows = table.num_rows();
        let columns = table
            .into_inner()
            .into_iter()
            .map(|(_, col)| col)
            .collect::<Vec<_>>();
        Self::new(num_rows as u64, &columns)
    }
}
