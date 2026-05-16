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
        base::{database::SchemaAccessorImpl, map::indexmap, scalar::test_scalar::TestScalar},
        sql::proof_exprs::{ProofExpr, ScalingCastExpr},
    };

    fn test_schema() -> (TableRef, SchemaAccessorImpl) {
        let table_ref = TableRef::from_names(Some("sxt"), "orders");
        let schema = SchemaAccessorImpl::new(indexmap! {
            table_ref.clone() => vec![
                ("id".into(), ColumnType::Int),
                ("flag".into(), ColumnType::Boolean),
                ("amount".into(), ColumnType::BigInt),
            ],
        });

        (table_ref, schema)
    }

    #[test]
    fn we_can_construct_column_helpers() {
        let (table_ref, schema) = test_schema();

        let id_ref = col_ref(&table_ref, "id", &schema);
        assert_eq!(id_ref.table_ref(), table_ref);
        assert_eq!(id_ref.column_id().value, "id");
        assert_eq!(*id_ref.column_type(), ColumnType::Int);

        assert!(matches!(
            column(&table_ref, "flag", &schema),
            DynProofExpr::Column(_)
        ));
        assert_eq!(
            col_expr(&table_ref, "amount", &schema).data_type(),
            ColumnType::BigInt
        );

        let columns = cols_expr(&table_ref, &["id", "flag"], &schema);
        assert_eq!(columns.len(), 2);
        assert_eq!(columns[0].column_id().value, "id");
        assert_eq!(columns[1].column_id().value, "flag");

        let aliased_columns = cols_expr_plan(&table_ref, &["id", "amount"], &schema);
        assert_eq!(aliased_columns.len(), 2);
        assert_eq!(aliased_columns[0].alias.value, "id");
        assert_eq!(aliased_columns[1].alias.value, "amount");

        assert_eq!(tab(&table_ref).table_ref, table_ref);
    }

    #[test]
    fn we_can_construct_binary_expression_helpers() {
        let lhs = const_int(3);
        let rhs = const_int(4);

        assert!(matches!(
            equal(lhs.clone(), rhs.clone()),
            DynProofExpr::Equals(_)
        ));
        assert!(matches!(
            lt(lhs.clone(), rhs.clone()),
            DynProofExpr::Inequality(_)
        ));
        assert!(matches!(
            gt(lhs.clone(), rhs.clone()),
            DynProofExpr::Inequality(_)
        ));
        assert!(matches!(
            lte(lhs.clone(), rhs.clone()),
            DynProofExpr::Not(_)
        ));
        assert!(matches!(
            gte(lhs.clone(), rhs.clone()),
            DynProofExpr::Not(_)
        ));
        assert!(matches!(
            add(lhs.clone(), rhs.clone()),
            DynProofExpr::Add(_)
        ));
        assert!(matches!(
            subtract(lhs.clone(), rhs.clone()),
            DynProofExpr::Subtract(_)
        ));
        assert!(matches!(
            multiply(lhs.clone(), rhs.clone()),
            DynProofExpr::Multiply(_)
        ));

        let truthy = const_bool(true);
        let falsy = const_bool(false);
        assert!(matches!(not(truthy.clone()), DynProofExpr::Not(_)));
        assert!(matches!(
            and(truthy.clone(), falsy.clone()),
            DynProofExpr::And(_)
        ));
        assert!(matches!(or(truthy, falsy), DynProofExpr::Or(_)));
    }

    #[test]
    fn we_can_construct_cast_and_literal_helpers() {
        assert!(matches!(
            cast(const_smallint(7), ColumnType::BigInt),
            DynProofExpr::Cast(_)
        ));
        assert!(matches!(
            scaling_cast(
                const_decimal75(10, 2, 12345),
                ColumnType::Decimal75(Precision::new(12).unwrap(), 4)
            ),
            DynProofExpr::ScalingCast(ScalingCastExpr { .. })
        ));

        assert_eq!(const_bool(true).data_type(), ColumnType::Boolean);
        assert_eq!(const_smallint(-1).data_type(), ColumnType::SmallInt);
        assert_eq!(const_int(2).data_type(), ColumnType::Int);
        assert_eq!(const_bigint(3).data_type(), ColumnType::BigInt);
        assert_eq!(const_int128(4).data_type(), ColumnType::Int128);
        assert_eq!(const_varchar("hello").data_type(), ColumnType::VarChar);
        assert_eq!(const_varbinary(b"abc").data_type(), ColumnType::VarBinary);
        assert_eq!(
            const_scalar::<TestScalar, _>(5).data_type(),
            ColumnType::Scalar
        );
        assert_eq!(
            const_decimal75(18, 2, 100).data_type(),
            ColumnType::Decimal75(Precision::new(18).unwrap(), 2)
        );
    }

    #[test]
    fn we_can_construct_alias_helpers() {
        let (table_ref, schema) = test_schema();

        let aliased = aliased_placeholder(1, ColumnType::BigInt, "p");
        assert_eq!(aliased.alias.value, "p");
        assert!(matches!(aliased.expr, DynProofExpr::Placeholder(_)));

        let plan = aliased_plan(const_bigint(9), "nine");
        assert_eq!(plan.alias.value, "nine");
        assert!(matches!(plan.expr, DynProofExpr::Literal(_)));

        let column_plan = col_expr_plan(&table_ref, "amount", &schema);
        assert_eq!(column_plan.alias.value, "amount");
        assert!(matches!(column_plan.expr, DynProofExpr::Column(_)));

        let sum = sum_expr(const_bigint(8), "total");
        assert_eq!(sum.alias.value, "total");
        assert!(matches!(sum.expr, DynProofExpr::Literal(_)));
    }
}
