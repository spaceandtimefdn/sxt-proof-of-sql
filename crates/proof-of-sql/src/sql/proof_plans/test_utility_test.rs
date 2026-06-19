//! Coverage tests for the public helpers in
//! [`super::test_utility`] of the `sql::proof_plans` module.
//!
//! The `proof_plans/test_utility.rs` file is `#[cfg(test)]` only — its 11
//! `pub fn` helpers are used by sibling test modules (`table_exec_test`,
//! `projection_exec_test`, `group_by_exec_test`, etc.). This module exercises
//! any helper that the sibling tests don't already cover, with the goal of
//! driving the file toward full coverage for Bounty #560.

use super::test_utility::{
    aggregate, column_field, empty_exec, filter, group_by, legacy_filter, projection, slice_exec,
    sort_merge_join, table_exec, union_exec,
};
use crate::{
    base::database::{ColumnType, TableRef},
    sql::proof_exprs::{AliasedDynProofExpr, ColumnExpr, DynProofExpr, TableExpr},
};
use alloc::{boxed::Box, vec};
use sqlparser::ast::Ident;

/// Build a minimal column for use in test plans.
fn id_column() -> ColumnExpr {
    let tab = TableRef::new("s", "t");
    ColumnExpr::new(crate::base::database::ColumnRef::new(
        tab,
        Ident::new("id"),
        ColumnType::BigInt,
    ))
}

#[test]
fn we_can_call_column_field() {
    let _field = column_field("id", ColumnType::BigInt);
}

#[test]
fn we_can_call_empty_exec() {
    let _plan = empty_exec();
}

#[test]
fn we_can_call_table_exec() {
    let tab = TableRef::new("s", "t");
    let _plan = table_exec(tab, vec![column_field("id", ColumnType::BigInt)]);
}

#[test]
fn we_can_call_projection() {
    let tab = TableRef::new("s", "t");
    let inner = table_exec(tab, vec![column_field("id", ColumnType::BigInt)]);
    let _plan = projection(
        vec![AliasedDynProofExpr {
            expr: DynProofExpr::Column(id_column()),
            alias: Ident::new("id"),
        }],
        inner,
    );
}

#[test]
fn we_can_call_legacy_filter() {
    let tab = TableRef::new("s", "t");
    let schema = vec![column_field("id", ColumnType::BigInt)];
    let table_expr = TableExpr {
        table_ref: tab.clone(),
        schema,
    };
    let _plan = legacy_filter(
        vec![AliasedDynProofExpr {
            expr: DynProofExpr::Column(id_column()),
            alias: Ident::new("id"),
        }],
        table_expr,
        DynProofExpr::Column(id_column()),
    );
}

#[test]
fn we_can_call_filter() {
    let tab = TableRef::new("s", "t");
    let inner = table_exec(tab, vec![column_field("id", ColumnType::BigInt)]);
    let _plan = filter(
        vec![AliasedDynProofExpr {
            expr: DynProofExpr::Column(id_column()),
            alias: Ident::new("id"),
        }],
        inner,
        DynProofExpr::Column(id_column()),
    );
}

#[test]
fn we_can_call_group_by() {
    let tab = TableRef::new("s", "t");
    let schema = vec![column_field("id", ColumnType::BigInt)];
    let table_expr = TableExpr {
        table_ref: tab,
        schema,
    };
    let _plan = group_by(
        vec![id_column()],
        vec![AliasedDynProofExpr {
            expr: DynProofExpr::Column(id_column()),
            alias: Ident::new("count"),
        }],
        "count",
        table_expr,
        DynProofExpr::Column(id_column()),
    );
}

#[test]
fn we_can_call_aggregate() {
    let tab = TableRef::new("s", "t");
    let inner = table_exec(tab, vec![column_field("id", ColumnType::BigInt)]);
    let _plan = aggregate(
        vec![AliasedDynProofExpr {
            expr: DynProofExpr::Column(id_column()),
            alias: Ident::new("id"),
        }],
        vec![AliasedDynProofExpr {
            expr: DynProofExpr::Column(id_column()),
            alias: Ident::new("count"),
        }],
        "count",
        inner,
        DynProofExpr::Column(id_column()),
    );
}

#[test]
fn we_can_call_slice_exec() {
    let tab = TableRef::new("s", "t");
    let inner = table_exec(tab, vec![column_field("id", ColumnType::BigInt)]);
    let _plan = slice_exec(inner, 0, Some(10));
}

#[test]
fn we_can_call_union_exec() {
    // Union needs two compatible input plans; same schema on both sides.
    let tab = TableRef::new("s", "t");
    let schema = vec![column_field("id", ColumnType::BigInt)];
    let left = table_exec(tab.clone(), schema.clone());
    let right = table_exec(tab, schema);
    let _plan = union_exec(vec![left, right]);
}

#[test]
fn we_can_call_sort_merge_join() {
    let tab_l = TableRef::new("s", "l");
    let tab_r = TableRef::new("s", "r");
    let schema = vec![column_field("id", ColumnType::BigInt)];
    let left = Box::new(table_exec(tab_l, schema.clone()));
    let right = Box::new(table_exec(tab_r, schema));
    let _plan = sort_merge_join(*left, *right, vec![0], vec![0], vec![Ident::new("id")]);
}
