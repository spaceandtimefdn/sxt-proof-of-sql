use super::{Column, ColumnType};
use crate::base::scalar::Scalar;
use alloc::vec::Vec;
use bumpalo::Bump;

/// View each source as a typed slice via `accessor`, then build a new slice that
/// takes, for each row `r`, `sources[row_source[r]]`'s value at `r`.
///
/// # Panics
/// Panics if `accessor` returns `None` for any source (i.e. the sources are not all
/// the same type) or if any `row_source[r]` is out of bounds.
fn select<'a, S, T, F>(
    alloc: &'a Bump,
    sources: &[Column<'a, S>],
    row_source: &[usize],
    accessor: F,
) -> &'a [T]
where
    S: Scalar,
    T: Copy,
    F: Fn(&Column<'a, S>) -> Option<&'a [T]>,
{
    let typed: Vec<&'a [T]> = sources
        .iter()
        .map(|source| accessor(source).expect("sources must share a column type"))
        .collect();
    alloc.alloc_slice_fill_with(row_source.len(), |row_num| {
        typed[row_source[row_num]][row_num]
    })
}

/// Same as [`select`], for the `(values, scalars)` pair layout of `VarChar`/`VarBinary`.
///
/// # Panics
/// Panics under the same conditions as [`select`].
fn select_pair<'a, S, T, F>(
    alloc: &'a Bump,
    sources: &[Column<'a, S>],
    row_source: &[usize],
    accessor: F,
) -> (&'a [T], &'a [S])
where
    S: Scalar,
    T: Copy,
    F: Fn(&Column<'a, S>) -> Option<(&'a [T], &'a [S])>,
{
    let typed: Vec<(&'a [T], &'a [S])> = sources
        .iter()
        .map(|source| accessor(source).expect("sources must share a column type"))
        .collect();
    (
        alloc.alloc_slice_fill_with(row_source.len(), |row_num| {
            typed[row_source[row_num]].0[row_num]
        }),
        alloc.alloc_slice_fill_with(row_source.len(), |row_num| {
            typed[row_source[row_num]].1[row_num]
        }),
    )
}

/// Build a column by taking, for each row `r`, the value at row `r` of
/// `sources[row_source[r]]`.
///
/// This is the row-wise selection underlying `CASE` (and any other conditional
/// pick): `row_source[r]` names which source column supplies row `r`.
///
/// All sources must share the column type, and the result has that type. Because
/// the prover copies real values, every column type is supported, including
/// `VarChar`/`VarBinary`.
///
/// # Panics
/// Panics if the sources are not all of the same type, if `sources` is empty, or
/// if any `row_source[r]` is out of bounds. Callers are expected to guarantee
/// these (e.g. a `CaseExpr` validated at construction).
pub(crate) fn select_column<'a, S: Scalar>(
    alloc: &'a Bump,
    sources: &[Column<'a, S>],
    row_source: &[usize],
) -> Column<'a, S> {
    let column_type = sources
        .first()
        .expect("select_column requires at least one source")
        .column_type();

    match column_type {
        ColumnType::Boolean => {
            Column::Boolean(select(alloc, sources, row_source, Column::as_boolean))
        }
        ColumnType::Uint8 => Column::Uint8(select(alloc, sources, row_source, Column::as_uint8)),
        ColumnType::TinyInt => {
            Column::TinyInt(select(alloc, sources, row_source, Column::as_tinyint))
        }
        ColumnType::SmallInt => {
            Column::SmallInt(select(alloc, sources, row_source, Column::as_smallint))
        }
        ColumnType::Int => Column::Int(select(alloc, sources, row_source, Column::as_int)),
        ColumnType::BigInt => Column::BigInt(select(alloc, sources, row_source, Column::as_bigint)),
        ColumnType::Int128 => Column::Int128(select(alloc, sources, row_source, Column::as_int128)),
        ColumnType::Scalar => Column::Scalar(select(alloc, sources, row_source, Column::as_scalar)),
        ColumnType::Decimal75(precision, scale) => Column::Decimal75(
            precision,
            scale,
            select(alloc, sources, row_source, Column::as_decimal75),
        ),
        ColumnType::TimestampTZ(tu, tz) => Column::TimestampTZ(
            tu,
            tz,
            select(alloc, sources, row_source, Column::as_timestamptz),
        ),
        ColumnType::VarChar => {
            Column::VarChar(select_pair(alloc, sources, row_source, Column::as_varchar))
        }
        ColumnType::VarBinary => Column::VarBinary(select_pair(
            alloc,
            sources,
            row_source,
            Column::as_varbinary,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn we_can_select_from_int_sources() {
        let bump = Bump::new();
        let a: Column<TestScalar> = Column::Int(&[10, 11, 12, 13]);
        let b: Column<TestScalar> = Column::Int(&[20, 21, 22, 23]);
        // pick a, b, b, a
        let result = select_column(&bump, &[a, b], &[0, 1, 1, 0]);
        assert_eq!(result, Column::Int(&[10, 21, 22, 13]));
    }

    #[test]
    fn we_can_select_from_varchar_sources() {
        let bump = Bump::new();
        let a_vals = ["x", "y", "z"];
        let b_vals = ["p", "q", "r"];
        let a_scalars = a_vals.iter().map(TestScalar::from).collect::<Vec<_>>();
        let b_scalars = b_vals.iter().map(TestScalar::from).collect::<Vec<_>>();
        let a: Column<TestScalar> = Column::VarChar((&a_vals, &a_scalars));
        let b: Column<TestScalar> = Column::VarChar((&b_vals, &b_scalars));
        // pick b, a, b
        let result = select_column(&bump, &[a, b], &[1, 0, 1]);
        let expected_vals = ["p", "y", "r"];
        let expected_scalars = expected_vals
            .iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        assert_eq!(result, Column::VarChar((&expected_vals, &expected_scalars)));
    }

    #[test]
    fn we_can_select_preserving_decimal_precision_and_scale() {
        let bump = Bump::new();
        let precision = crate::base::math::decimal::Precision::new(10).unwrap();
        let a_vals = [TestScalar::from(1), TestScalar::from(2)];
        let b_vals = [TestScalar::from(3), TestScalar::from(4)];
        let a = Column::Decimal75(precision, 2, &a_vals);
        let b = Column::Decimal75(precision, 2, &b_vals);
        let result = select_column(&bump, &[a, b], &[1, 0]);
        let expected = [TestScalar::from(3), TestScalar::from(2)];
        assert_eq!(result, Column::Decimal75(precision, 2, &expected));
    }
}
