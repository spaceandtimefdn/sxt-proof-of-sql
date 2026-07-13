use super::LegacyFilterExec;
use crate::{
    base::database::{ColumnRef, ColumnType, LiteralValue, TableRef},
    sql::proof_exprs::{test_utility::*, ColumnExpr, DynProofExpr, LiteralExpr},
};
use sqlparser::ast::Ident;

#[test]
fn we_can_access_legacy_filter_exec_components() {
    let table_ref = TableRef::new("sxt", "legacy_filter_accessor_tab");
    let table = tab(&table_ref);
    let aliased_results = vec![aliased_plan(
        DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
            table_ref.clone(),
            Ident::new("result_col"),
            ColumnType::BigInt,
        ))),
        "result_col",
    )];
    let where_clause = DynProofExpr::Literal(LiteralExpr::new(LiteralValue::Boolean(true)));

    let provable_ast =
        LegacyFilterExec::new(aliased_results.clone(), table.clone(), where_clause.clone());

    assert_eq!(provable_ast.aliased_results(), aliased_results.as_slice());
    assert_eq!(provable_ast.table(), &table);
    assert_eq!(provable_ast.where_clause(), &where_clause);
}
