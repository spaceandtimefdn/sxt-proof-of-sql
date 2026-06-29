use super::{AliasedDynProofExpr, ColumnExpr, DynProofExpr, TableExpr};
use crate::base::{
    database::{ColumnRef, ColumnType, LiteralValue, SchemaAccessor, TableRef},
    math::{decimal::Precision, i256::I256},
    scalar::Scalar,
};
use sqlparser::ast::Ident;

pub fn col_ref(tab: &TableRef, name: &str, accessor: &impl SchemaAccessor) -> ColumnRef {
    let name: Ident = name.into();
    let type_col = accessor.lookup_column(tab, &name).unwrap();
    ColumnRef::new(tab.clone(), name, type_col)
}

/// # Panics
/// Panics if:
/// - `accessor.lookup_column()` returns `None`, indicating the column is not found.
pub fn column(tab: &TableRef, name: &str, accessor: &impl SchemaAccessor) -> DynProofExpr {
    let name: Ident = name.into();
    let type_col = accessor.lookup_column(tab, &name).unwrap();
    DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(tab.clone(), name, type_col)))
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_equals()` returns an error.
pub fn equal(left: DynProofExpr, right: DynProofExpr) -> DynProofExpr {
    DynProofExpr::try_new_equals(left, right).unwrap()
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_inequality()` returns an error.
pub fn lt(left: DynProofExpr, right: DynProofExpr) -> DynProofExpr {
    DynProofExpr::try_new_inequality(left, right, true).unwrap()
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_inequality()` returns an error.
pub fn gt(left: DynProofExpr, right: DynProofExpr) -> DynProofExpr {
    DynProofExpr::try_new_inequality(left, right, false).unwrap()
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_inequality()` returns an error.
pub fn lte(left: DynProofExpr, right: DynProofExpr) -> DynProofExpr {
    not(DynProofExpr::try_new_inequality(left, right, false).unwrap())
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_inequality()` returns an error.
pub fn gte(left: DynProofExpr, right: DynProofExpr) -> DynProofExpr {
    not(DynProofExpr::try_new_inequality(left, right, true).unwrap())
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_not()` returns an error.
pub fn not(expr: DynProofExpr) -> DynProofExpr {
    DynProofExpr::try_new_not(expr).unwrap()
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_and()` returns an error.
pub fn and(left: DynProofExpr, right: DynProofExpr) -> DynProofExpr {
    DynProofExpr::try_new_and(left, right).unwrap()
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_or()` returns an error.
pub fn or(left: DynProofExpr, right: DynProofExpr) -> DynProofExpr {
    DynProofExpr::try_new_or(left, right).unwrap()
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_add()` returns an error.
pub fn add(left: DynProofExpr, right: DynProofExpr) -> DynProofExpr {
    DynProofExpr::try_new_add(left, right).unwrap()
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_subtract()` returns an error.
pub fn subtract(left: DynProofExpr, right: DynProofExpr) -> DynProofExpr {
    DynProofExpr::try_new_subtract(left, right).unwrap()
}

/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_multiply()` returns an error.
pub fn multiply(left: DynProofExpr, right: DynProofExpr) -> DynProofExpr {
    DynProofExpr::try_new_multiply(left, right).unwrap()
}

pub fn cast(left: DynProofExpr, right: ColumnType) -> DynProofExpr {
    DynProofExpr::try_new_cast(left, right).unwrap()
}

pub fn scaling_cast(left: DynProofExpr, right: ColumnType) -> DynProofExpr {
    DynProofExpr::try_new_scaling_cast(left, right).unwrap()
}

pub fn const_bool(val: bool) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::Boolean(val))
}

pub fn const_smallint(val: i16) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::SmallInt(val))
}

pub fn const_int(val: i32) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::Int(val))
}

pub fn const_bigint(val: i64) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::BigInt(val))
}

pub fn const_int128(val: i128) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::Int128(val))
}

pub fn const_varchar(val: &str) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::VarChar(val.to_string()))
}

/// Creates a new `DynProofExpr::Literal` expression for a varbinary value.
pub fn const_varbinary(val: &[u8]) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::VarBinary(val.to_vec()))
}

/// Create a constant scalar value. Used if we don't want to specify column types.
pub fn const_scalar<S: Scalar, T: Into<S>>(val: T) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::Scalar(val.into().into()))
}

/// # Panics
/// Panics if:
/// - `Precision::new(precision)` fails, meaning the provided precision is invalid.
pub fn const_decimal75<T: Into<I256>>(precision: u8, scale: i8, val: T) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::Decimal75(
        Precision::new(precision).unwrap(),
        scale,
        val.into(),
    ))
}

/// Aliased placeholder expression
pub fn aliased_placeholder(index: usize, col_type: ColumnType, alias: &str) -> AliasedDynProofExpr {
    AliasedDynProofExpr {
        expr: DynProofExpr::try_new_placeholder(index, col_type).unwrap(),
        alias: alias.into(),
    }
}

pub fn tab(tab: &TableRef) -> TableExpr {
    TableExpr {
        table_ref: tab.clone(),
    }
}

/// # Panics
/// Panics if:
/// - `alias.parse()` fails to parse the provided alias string.
pub fn aliased_plan(expr: DynProofExpr, alias: &str) -> AliasedDynProofExpr {
    AliasedDynProofExpr {
        expr,
        alias: alias.into(),
    }
}

/// # Panics
/// Panics if:
/// - `name.parse()` fails to parse the provided column name.
/// - `col_ref()` fails to find the referenced column, leading to a panic in the column accessor.
pub fn col_expr_plan(
    tab: &TableRef,
    name: &str,
    accessor: &impl SchemaAccessor,
) -> AliasedDynProofExpr {
    AliasedDynProofExpr {
        expr: DynProofExpr::Column(ColumnExpr::new(col_ref(tab, name, accessor))),
        alias: name.into(),
    }
}

pub fn cols_expr_plan(
    tab: &TableRef,
    names: &[&str],
    accessor: &impl SchemaAccessor,
) -> Vec<AliasedDynProofExpr> {
    names
        .iter()
        .map(|name| col_expr_plan(tab, name, accessor))
        .collect()
}

pub fn col_expr(tab: &TableRef, name: &str, accessor: &impl SchemaAccessor) -> ColumnExpr {
    ColumnExpr::new(col_ref(tab, name, accessor))
}

pub fn cols_expr(
    tab: &TableRef,
    names: &[&str],
    accessor: &impl SchemaAccessor,
) -> Vec<ColumnExpr> {
    names
        .iter()
        .map(|name| col_expr(tab, name, accessor))
        .collect()
}

/// # Panics
/// Panics if:
/// - `alias.parse()` fails to parse the provided alias string.
pub fn sum_expr(expr: DynProofExpr, alias: &str) -> AliasedDynProofExpr {
    AliasedDynProofExpr {
        expr,
        alias: alias.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::{
            commitment::naive_evaluation_proof::NaiveEvaluationProof,
            database::{table_utility::*, ColumnType, TableRef, TableTestAccessor},
            math::decimal::Precision,
            scalar::test_scalar::TestScalar,
        },
        sql::proof_exprs::ProofExpr,
    };
    use bumpalo::Bump;

    #[test]
    fn test_utility_builds_proof_expressions_and_aliases() {
        let alloc = Bump::new();
        let table_ref = TableRef::new("sxt", "proof_expr_test_utility");
        let data = table([
            borrowed_bigint("id", [1, 2], &alloc),
            borrowed_int("quantity", [3, 4], &alloc),
            borrowed_boolean("is_active", [true, false], &alloc),
            borrowed_varchar("label", ["a", "b"], &alloc),
        ]);
        let accessor = TableTestAccessor::<NaiveEvaluationProof>::new_from_table(
            table_ref.clone(),
            data,
            0,
            (),
        );

        let id_ref = col_ref(&table_ref, "id", &accessor);
        assert_eq!(id_ref.table_ref(), table_ref);
        assert_eq!(id_ref.column_id().to_string(), "id");
        assert_eq!(*id_ref.column_type(), ColumnType::BigInt);

        assert!(matches!(
            column(&table_ref, "id", &accessor),
            DynProofExpr::Column(_)
        ));
        assert_eq!(
            col_expr(&table_ref, "quantity", &accessor).data_type(),
            ColumnType::Int
        );
        assert_eq!(
            cols_expr(&table_ref, &["id", "quantity"], &accessor).len(),
            2
        );

        assert_eq!(
            equal(const_int(1), const_int(1)).data_type(),
            ColumnType::Boolean
        );
        assert_eq!(
            lt(const_int(1), const_int(2)).data_type(),
            ColumnType::Boolean
        );
        assert_eq!(
            gt(const_int(2), const_int(1)).data_type(),
            ColumnType::Boolean
        );
        assert_eq!(
            lte(const_int(1), const_int(2)).data_type(),
            ColumnType::Boolean
        );
        assert_eq!(
            gte(const_int(2), const_int(1)).data_type(),
            ColumnType::Boolean
        );
        assert_eq!(not(const_bool(false)).data_type(), ColumnType::Boolean);
        assert_eq!(
            and(const_bool(true), const_bool(false)).data_type(),
            ColumnType::Boolean
        );
        assert_eq!(
            or(const_bool(true), const_bool(false)).data_type(),
            ColumnType::Boolean
        );

        assert!(matches!(
            add(const_bigint(1), const_bigint(2)),
            DynProofExpr::Add(_)
        ));
        assert!(matches!(
            subtract(const_bigint(3), const_bigint(2)),
            DynProofExpr::Subtract(_)
        ));
        assert!(matches!(
            multiply(const_bigint(3), const_bigint(2)),
            DynProofExpr::Multiply(_)
        ));
        assert_eq!(
            cast(const_smallint(7), ColumnType::BigInt).data_type(),
            ColumnType::BigInt
        );

        let scaled_decimal_type = ColumnType::Decimal75(Precision::new(11).unwrap(), 1);
        assert_eq!(
            scaling_cast(const_int(12), scaled_decimal_type).data_type(),
            scaled_decimal_type
        );
        let literal_decimal_type = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        assert_eq!(
            const_decimal75(10, 2, 123_i32).data_type(),
            literal_decimal_type
        );

        assert_eq!(const_bool(true).data_type(), ColumnType::Boolean);
        assert_eq!(const_smallint(1).data_type(), ColumnType::SmallInt);
        assert_eq!(const_int(1).data_type(), ColumnType::Int);
        assert_eq!(const_bigint(1).data_type(), ColumnType::BigInt);
        assert_eq!(const_int128(1).data_type(), ColumnType::Int128);
        assert_eq!(const_varchar("value").data_type(), ColumnType::VarChar);
        assert_eq!(const_varbinary(b"value").data_type(), ColumnType::VarBinary);
        assert_eq!(
            const_scalar::<TestScalar, _>(1_i64).data_type(),
            ColumnType::Scalar
        );

        let placeholder = aliased_placeholder(1, ColumnType::BigInt, "p");
        assert_eq!(placeholder.alias.to_string(), "p");
        assert!(matches!(placeholder.expr, DynProofExpr::Placeholder(_)));

        assert_eq!(tab(&table_ref).table_ref, table_ref);

        let aliased_literal = aliased_plan(const_bool(true), "flag");
        assert_eq!(aliased_literal.alias.to_string(), "flag");
        assert_eq!(aliased_literal.expr.data_type(), ColumnType::Boolean);

        let aliased_column = col_expr_plan(&table_ref, "id", &accessor);
        assert_eq!(aliased_column.alias.to_string(), "id");
        assert_eq!(aliased_column.expr.data_type(), ColumnType::BigInt);
        assert_eq!(
            cols_expr_plan(&table_ref, &["id", "quantity"], &accessor).len(),
            2
        );

        let aliased_sum = sum_expr(const_bigint(10), "total");
        assert_eq!(aliased_sum.alias.to_string(), "total");
        assert_eq!(aliased_sum.expr.data_type(), ColumnType::BigInt);
    }
}
