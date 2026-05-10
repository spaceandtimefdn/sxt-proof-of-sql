use super::{slice_operation::apply_slice_to_indexes, Column, ColumnOperationResult, ColumnType};
use crate::base::scalar::Scalar;
use bumpalo::Bump;

/// Apply a `Column` to a vector of indexes, returning a new `Column` with the
/// values at the given indexes. Repetitions are allowed.
///
/// # Panics
/// Panics if any of the indexes are out of bounds.
pub(crate) fn apply_column_to_indexes<'a, S>(
    column: &Column<'a, S>,
    alloc: &'a Bump,
    indexes: &[usize],
) -> ColumnOperationResult<Column<'a, S>>
where
    S: Scalar,
{
    match column.column_type() {
        ColumnType::Boolean => {
            let raw_values = apply_slice_to_indexes(
                column.as_boolean().expect("Column types should match"),
                indexes,
            )?;
            Ok(Column::Boolean(alloc.alloc_slice_copy(&raw_values) as &[_]))
        }
        ColumnType::TinyInt => {
            let raw_values = apply_slice_to_indexes(
                column.as_tinyint().expect("Column types should match"),
                indexes,
            )?;
            Ok(Column::TinyInt(alloc.alloc_slice_copy(&raw_values) as &[_]))
        }
        ColumnType::Uint8 => {
            let raw_values = apply_slice_to_indexes(
                column.as_uint8().expect("Column types should match"),
                indexes,
            )?;
            Ok(Column::Uint8(alloc.alloc_slice_copy(&raw_values) as &[_]))
        }
        ColumnType::SmallInt => {
            let raw_values = apply_slice_to_indexes(
                column.as_smallint().expect("Column types should match"),
                indexes,
            )?;
            Ok(Column::SmallInt(alloc.alloc_slice_copy(&raw_values) as &[_]))
        }
        ColumnType::Int => {
            let raw_values = apply_slice_to_indexes(
                column.as_int().expect("Column types should match"),
                indexes,
            )?;
            Ok(Column::Int(alloc.alloc_slice_copy(&raw_values) as &[_]))
        }
        ColumnType::BigInt => {
            let raw_values = apply_slice_to_indexes(
                column.as_bigint().expect("Column types should match"),
                indexes,
            )?;
            Ok(Column::BigInt(alloc.alloc_slice_copy(&raw_values) as &[_]))
        }
        ColumnType::Int128 => {
            let raw_values = apply_slice_to_indexes(
                column.as_int128().expect("Column types should match"),
                indexes,
            )?;
            Ok(Column::Int128(alloc.alloc_slice_copy(&raw_values) as &[_]))
        }
        ColumnType::Scalar => {
            let raw_values = apply_slice_to_indexes(
                column.as_scalar().expect("Column types should match"),
                indexes,
            )?;
            Ok(Column::Scalar(alloc.alloc_slice_copy(&raw_values) as &[_]))
        }
        ColumnType::Decimal75(precision, scale) => {
            let raw_values = apply_slice_to_indexes(
                column.as_decimal75().expect("Column types should match"),
                indexes,
            )?;
            Ok(Column::Decimal75(
                precision,
                scale,
                alloc.alloc_slice_copy(&raw_values) as &[_],
            ))
        }
        ColumnType::VarChar => {
            let (raw_values, raw_scalars) = column.as_varchar().expect("Column types should match");
            let raw_values = apply_slice_to_indexes(raw_values, indexes)?;
            let scalars = apply_slice_to_indexes(raw_scalars, indexes)?;
            Ok(Column::VarChar((
                alloc.alloc_slice_clone(&raw_values) as &[_],
                alloc.alloc_slice_copy(&scalars) as &[_],
            )))
        }

        ColumnType::VarBinary => {
            let (raw_values, raw_scalars) =
                column.as_varbinary().expect("Column types should match");
            let raw_values = apply_slice_to_indexes(raw_values, indexes)?;
            let scalars = apply_slice_to_indexes(raw_scalars, indexes)?;
            Ok(Column::VarBinary((
                alloc.alloc_slice_clone(&raw_values) as &[_],
                alloc.alloc_slice_copy(&scalars) as &[_],
            )))
        }
        ColumnType::TimestampTZ(tu, tz) => {
            let raw_values = apply_slice_to_indexes(
                column.as_timestamptz().expect("Column types should match"),
                indexes,
            )?;
            Ok(Column::TimestampTZ(
                tu,
                tz,
                alloc.alloc_slice_copy(&raw_values) as &[_],
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{
        database::ColumnOperationError,
        math::decimal::Precision,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
        scalar::test_scalar::TestScalar,
    };

    #[test]
    fn test_apply_index_op() {
        let bump = Bump::new();
        let column: Column<TestScalar> = Column::Int(&[1, 2, 3, 4, 5]);
        let indexes = [1, 3, 1, 2];
        let result = apply_column_to_indexes(&column, &bump, &indexes).unwrap();
        assert_eq!(result, Column::Int(&[2, 4, 2, 3]));

        let scalars = [1, 2, 3].iter().map(TestScalar::from).collect::<Vec<_>>();
        let column: Column<TestScalar> = Column::Scalar(&scalars);
        let indexes = [1, 1, 1];
        let result = apply_column_to_indexes(&column, &bump, &indexes).unwrap();
        let expected_scalars = [2, 2, 2].iter().map(TestScalar::from).collect::<Vec<_>>();
        assert_eq!(result, Column::Scalar(&expected_scalars));

        let strings = vec!["a", "b", "c"];
        let scalars = strings.iter().map(TestScalar::from).collect::<Vec<_>>();
        let column: Column<TestScalar> = Column::VarChar((&strings, &scalars));
        let indexes = [2, 1, 1];
        let result = apply_column_to_indexes(&column, &bump, &indexes).unwrap();
        let expected_strings = vec!["c", "b", "b"];
        let expected_scalars = expected_strings
            .iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        assert_eq!(
            result,
            Column::VarChar((&expected_strings, &expected_scalars))
        );
    }

    #[test]
    fn test_apply_index_op_out_of_bound() {
        let bump = Bump::new();
        let column: Column<TestScalar> = Column::Int(&[1, 2, 3, 4, 5]);
        let indexes = [1, 3, 1, 2, 5];
        let result = apply_column_to_indexes(&column, &bump, &indexes);
        assert!(matches!(
            result,
            Err(ColumnOperationError::IndexOutOfBounds { .. })
        ));
    }

    #[test]
    fn test_apply_index_op_remaining_column_variants() {
        let bump = Bump::new();
        let indexes = [2, 0, 2];

        let column: Column<TestScalar> = Column::Boolean(&[false, true, true]);
        let result = apply_column_to_indexes(&column, &bump, &indexes).unwrap();
        assert_eq!(result, Column::Boolean(&[true, false, true]));

        let column: Column<TestScalar> = Column::Uint8(&[3, 5, 8]);
        let result = apply_column_to_indexes(&column, &bump, &indexes).unwrap();
        assert_eq!(result, Column::Uint8(&[8, 3, 8]));

        let column: Column<TestScalar> = Column::TinyInt(&[-3, 5, 8]);
        let result = apply_column_to_indexes(&column, &bump, &indexes).unwrap();
        assert_eq!(result, Column::TinyInt(&[8, -3, 8]));

        let column: Column<TestScalar> = Column::SmallInt(&[-300, 500, 800]);
        let result = apply_column_to_indexes(&column, &bump, &indexes).unwrap();
        assert_eq!(result, Column::SmallInt(&[800, -300, 800]));

        let column: Column<TestScalar> = Column::BigInt(&[-3_000, 5_000, 8_000]);
        let result = apply_column_to_indexes(&column, &bump, &indexes).unwrap();
        assert_eq!(result, Column::BigInt(&[8_000, -3_000, 8_000]));

        let column: Column<TestScalar> = Column::Int128(&[-30_000, 50_000, 80_000]);
        let result = apply_column_to_indexes(&column, &bump, &indexes).unwrap();
        assert_eq!(result, Column::Int128(&[80_000, -30_000, 80_000]));

        let precision = Precision::new(75).unwrap();
        let decimals = [11, 22, 33]
            .iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let column: Column<TestScalar> = Column::Decimal75(precision, 2, &decimals);
        let result = apply_column_to_indexes(&column, &bump, &indexes).unwrap();
        let expected_decimals = [33, 11, 33]
            .iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        assert_eq!(
            result,
            Column::Decimal75(precision, 2, expected_decimals.as_slice())
        );

        let column: Column<TestScalar> = Column::TimestampTZ(
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            &[100, 200, 300],
        );
        let result = apply_column_to_indexes(&column, &bump, &indexes).unwrap();
        assert_eq!(
            result,
            Column::TimestampTZ(
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                &[300, 100, 300]
            )
        );
    }

    #[test]
    fn test_apply_index_op_varbinary() {
        let bump = Bump::new();

        let raw_bytes: Vec<&[u8]> = vec![b"foo".as_ref(), b"bar".as_ref(), b"baz".as_ref()];
        let scalars: Vec<TestScalar> = raw_bytes
            .iter()
            .map(|b| TestScalar::from_le_bytes_mod_order(b))
            .collect();

        let column = Column::VarBinary((raw_bytes.as_slice(), scalars.as_slice()));

        let indexes = [2, 0];

        let result = apply_column_to_indexes(&column, &bump, &indexes).unwrap();
        let expected_bytes = vec![b"baz".as_ref(), b"foo".as_ref()];
        let expected_scalars: Vec<TestScalar> = expected_bytes
            .iter()
            .map(|b| TestScalar::from_le_bytes_mod_order(b))
            .collect();
        let expected = Column::VarBinary((expected_bytes.as_slice(), expected_scalars.as_slice()));

        assert_eq!(result, expected);
    }
}
