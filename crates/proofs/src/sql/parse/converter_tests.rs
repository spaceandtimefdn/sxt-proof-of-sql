use crate::base::database::{TableRef, TestAccessor};
use crate::record_batch;
use crate::sql::ast::test_utility::{
    and, col_result, cols_result, const_v, equal, filter, not, or, tab,
};
use crate::sql::parse::{Converter, QueryExpr};
use crate::sql::transform::test_utility::{composite_result, orders, result};
use proofs_sql::intermediate_ast::OrderByDirection::{Asc, Desc};

use arrow::record_batch::RecordBatch;
use proofs_sql::sql::SelectStatementParser;

fn query_to_provable_ast(table: TableRef, query: &str, accessor: &TestAccessor) -> QueryExpr {
    let intermediate_ast = SelectStatementParser::new().parse(query).unwrap();
    Converter::default()
        .visit_intermediate_ast(&intermediate_ast, accessor, table.schema_id())
        .unwrap()
}

fn invalid_query_to_provable_ast(table: TableRef, query: &str, accessor: &TestAccessor) {
    let intermediate_ast = SelectStatementParser::new().parse(query).unwrap();
    assert!(Converter::default()
        .visit_intermediate_ast(&intermediate_ast, accessor, table.schema_id())
        .is_err());
}

pub fn record_batch_to_accessor(table: TableRef, data: RecordBatch, offset: usize) -> TestAccessor {
    let mut accessor = TestAccessor::new();
    accessor.add_table(table, data, offset);
    accessor
}

#[test]
fn we_can_convert_an_ast_with_one_column() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => [3]
        ),
        0,
    );
    let ast = query_to_provable_ast(t, "select a from sxt_tab where a = 3", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a"], &accessor),
            tab(t),
            equal(t, "a", 3, &accessor),
        ),
        result(),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_two_columns() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => Vec::<i64>::new(),
            "b" => Vec::<i64>::new(),
            "c" => Vec::<i64>::new(),
        ),
        0_usize,
    );
    let ast = query_to_provable_ast(t, "select a,  b from sxt_tab where c = 123", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a", "b"], &accessor),
            tab(t),
            equal(t, "c", 123, &accessor),
        ),
        result(),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_all_result_columns_with_select_star() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "b" => [5, 6],
            "a" => [3, 2],
        ),
        0_usize,
    );
    let ast = query_to_provable_ast(t, "select * from sxt_tab where a = 3", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["b", "a"], &accessor),
            tab(t),
            equal(t, "a", 3, &accessor),
        ),
        result(),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_all_result_columns_with_more_complex_select_star() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "b" => [5, 6],
            "a" => [3, 2],
            "c" => [78, 8]
        ),
        0_usize,
    );
    let ast = query_to_provable_ast(t, "select a, *, b,* from sxt_tab where a = 3", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a", "b", "a", "c", "b", "b", "a", "c"], &accessor),
            tab(t),
            equal(t, "a", 3, &accessor),
        ),
        result(),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_one_positive_cond() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => Vec::<i64>::new(),
            "b" => Vec::<i64>::new(),
        ),
        0_usize,
    );
    let ast = query_to_provable_ast(t, "select a from sxt_tab where b = +4", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a"], &accessor),
            tab(t),
            equal(t, "b", 4, &accessor),
        ),
        result(),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_one_not_equals_cond() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => Vec::<i64>::new(),
            "b" => Vec::<i64>::new(),
        ),
        0_usize,
    );
    let ast = query_to_provable_ast(t, "select a from sxt_tab where b <> +4", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a"], &accessor),
            tab(t),
            not(equal(t, "b", 4, &accessor)),
        ),
        result(),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_one_negative_cond() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => Vec::<i64>::new(),
            "b" => Vec::<i64>::new(),
        ),
        0_usize,
    );
    let ast = query_to_provable_ast(t, "select a from sxt_tab where b = -4", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a"], &accessor),
            tab(t),
            equal(t, "b", -4, &accessor),
        ),
        result(),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_cond_and() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => Vec::<i64>::new(),
            "b" => Vec::<i64>::new(),
            "c" => Vec::<i64>::new(),
        ),
        0_usize,
    );
    let ast = query_to_provable_ast(
        t,
        "select a from sxt_tab where (b = 3) and (c = -2)",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a"], &accessor),
            tab(t),
            and(equal(t, "b", 3, &accessor), equal(t, "c", -2, &accessor)),
        ),
        result(),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_cond_or() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => Vec::<i64>::new(),
            "b" => Vec::<i64>::new(),
            "c" => Vec::<i64>::new(),
        ),
        0_usize,
    );
    let ast = query_to_provable_ast(
        t,
        "select a from sxt_tab where (b = 3) or (c = -2)",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a"], &accessor),
            tab(t),
            or(equal(t, "b", 3, &accessor), equal(t, "c", -2, &accessor)),
        ),
        result(),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_conds_or_not() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => Vec::<i64>::new(),
            "b" => Vec::<i64>::new(),
            "c" => Vec::<i64>::new(),
        ),
        0_usize,
    );
    let ast = query_to_provable_ast(
        t,
        "select a from sxt_tab where (b = 3) or (not (c = -2))",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a"], &accessor),
            tab(t),
            or(
                equal(t, "b", 3, &accessor),
                not(equal(t, "c", -2, &accessor)),
            ),
        ),
        result(),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_conds_not_and_or() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => Vec::<i64>::new(),
            "b" => Vec::<i64>::new(),
            "c" => Vec::<i64>::new(),
            "f" => Vec::<i64>::new(),
        ),
        0_usize,
    );
    let ast = query_to_provable_ast(
        t,
        "select a from sxt_tab where not (((f = 45) or (c = -2)) and (b = 3))",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a"], &accessor),
            tab(t),
            not(and(
                or(equal(t, "f", 45, &accessor), equal(t, "c", -2, &accessor)),
                equal(t, "b", 3, &accessor),
            )),
        ),
        result(),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_the_min_i64_filter_value() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => [3],
        ),
        0,
    );
    let ast = query_to_provable_ast(
        t,
        "select a from sxt_tab where a = -9223372036854775808",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a"], &accessor),
            tab(t),
            equal(t, "a", i64::MIN, &accessor),
        ),
        result(),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_with_the_max_i64_filter_value() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => [3],
        ),
        0,
    );
    let ast = query_to_provable_ast(
        t,
        "select a from sxt_tab where a = 9223372036854775807",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a"], &accessor),
            tab(t),
            equal(t, "a", i64::MAX, &accessor),
        ),
        result(),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_using_as_rename_keyword() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => Vec::<i64>::new(),
            "b" => Vec::<i64>::new(),
        ),
        0,
    );
    let ast = query_to_provable_ast(
        t,
        "select a as b_rename from sxt_tab where b = +4",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            vec![col_result(t, "a", "b_rename", &accessor)],
            tab(t),
            equal(t, "b", 4, &accessor),
        ),
        result(),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_cannot_convert_an_ast_with_a_nonexistent_column() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "b" => [3],
        ),
        0,
    );
    invalid_query_to_provable_ast(t, "select * from sxt_tab where a = 3", &accessor);
}

#[test]
fn we_cannot_convert_an_ast_with_a_column_type_different_than_equal_literal() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "b" => ["abc"],
        ),
        0,
    );
    invalid_query_to_provable_ast(t, "select * from sxt_tab where b = 123", &accessor);
}

#[test]
fn we_can_convert_an_ast_with_a_schema() {
    let t = "eth.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => [3],
        ),
        0,
    );
    let ast = query_to_provable_ast(t, "select a from eth.sxt_tab where a = 3", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a"], &accessor),
            tab(t),
            equal(t, "a", 3, &accessor),
        ),
        result(),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_convert_an_ast_without_any_filter() {
    let t = "eth.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "a" => [3],
        ),
        0,
    );
    let expected_ast = QueryExpr::new(
        filter(cols_result(t, &["a"], &accessor), tab(t), const_v(true)),
        result(),
    );
    let queries = ["select * from eth.sxt_tab", "select a from eth.sxt_tab"];
    for query in queries {
        let ast = query_to_provable_ast(t, query, &accessor);
        assert_eq!(ast, expected_ast);
    }
}

#[test]
fn we_can_parse_order_by_with_a_single_column() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "b" => [5, 6],
            "a" => [3, 2],
        ),
        0_usize,
    );
    let ast = query_to_provable_ast(t, "select * from sxt_tab where a = 3 order by b", &accessor);
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["b", "a"], &accessor),
            tab(t),
            equal(t, "a", 3, &accessor),
        ),
        composite_result(orders(&["b"], &[Asc])),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_order_by_with_multiple_columns() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "b" => [5, 6, -7],
            "a" => [3, 2, 3],
        ),
        0_usize,
    );
    let ast = query_to_provable_ast(
        t,
        "select a, b from sxt_tab where a = 3 order by b desc, a asc",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            cols_result(t, &["a", "b"], &accessor),
            tab(t),
            equal(t, "a", 3, &accessor),
        ),
        composite_result(orders(&["b", "a"], &[Desc, Asc])),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_order_by_referencing_an_alias_associated_with_column_b_but_with_name_equals_column_a_also_renamed(
) {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "salary" => [5, 6],
            "name" => ["abc", "ed"],
        ),
        0,
    );
    let ast = query_to_provable_ast(
        t,
        "select salary as s, name as salary from sxt_tab where salary = 5 order by salary desc",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            vec![
                col_result(t, "salary", "s", &accessor),
                col_result(t, "name", "salary", &accessor),
            ],
            tab(t),
            equal(t, "salary", 5, &accessor),
        ),
        composite_result(orders(&["salary"], &[Desc])),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_can_parse_order_by_remapping_the_column_name_reference_to_an_existing_alias() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "salary" => [5, 6],
        ),
        0,
    );
    let ast = query_to_provable_ast(
        t,
        "select salary as s from sxt_tab order by salary",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            vec![col_result(t, "salary", "s", &accessor)],
            tab(t),
            const_v(true),
        ),
        composite_result(orders(&["s"], &[Asc])),
    );
    assert_eq!(ast, expected_ast);
}

#[test]
fn we_cannot_parse_order_by_referencing_an_alias_associated_with_column_b_but_with_name_equals_column_a_not_renamed(
) {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "salary" => [5, 6],
            "name" => ["abc", "ed"],
        ),
        0,
    );
    invalid_query_to_provable_ast(
        t,
        "select salary, name as salary from sxt_tab where salary = 5 order by salary desc",
        &accessor,
    );
}

#[test]
fn we_cannot_parse_order_by_referencing_an_existing_column_not_appearing_in_the_result_select_list()
{
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "b" => [5, 6],
            "a" => [3, 2],
        ),
        0,
    );

    // Note: While this operation is acceptable with PostgreSQL, we do not currently support it.
    invalid_query_to_provable_ast(t, "select a from sxt_tab order by b desc", &accessor);
}

#[test]
fn we_cannot_parse_order_by_referencing_an_alias_name_associated_with_two_different_columns() {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "salary" => [5, 6],
            "name" => ["abc", "ed"],
        ),
        0,
    );
    invalid_query_to_provable_ast(
        t,
        "select salary as s, name as s from sxt_tab order by s desc",
        &accessor,
    );

    invalid_query_to_provable_ast(
        t,
        "select salary as name, name from sxt_tab order by name desc",
        &accessor,
    );

    // Note: While this is not ambiguous with PostgreSQL,
    // it currently is with our code because there is
    // no way to differentiate between the two columns
    // in the record batch since they share the same name.
    invalid_query_to_provable_ast(
        t,
        "select salary as name, name from sxt_tab order by salary desc",
        &accessor,
    );

    // Note: This is not ambiguous with PostgreSQL either,
    // but it is with our code for the reasons mentioned above.
    invalid_query_to_provable_ast(
        t,
        "select salary as s, name as s from sxt_tab order by salary desc",
        &accessor,
    );
}

#[test]
fn we_can_parse_order_by_queries_with_the_same_column_name_appearing_more_than_once_and_with_different_alias_name(
) {
    let t = "sxt.sxt_tab".parse().unwrap();
    let accessor = record_batch_to_accessor(
        t,
        record_batch!(
            "salary" => [5, 6],
            "name" => ["abc", "ed"],
        ),
        0,
    );

    let ast = query_to_provable_ast(
        t,
        "select salary as s, name, salary as d from sxt_tab order by salary desc",
        &accessor,
    );
    let expected_ast = QueryExpr::new(
        filter(
            vec![
                col_result(t, "salary", "s", &accessor),
                col_result(t, "name", "name", &accessor),
                col_result(t, "salary", "d", &accessor),
            ],
            tab(t),
            const_v(true),
        ),
        composite_result(orders(&["s"], &[Desc])),
    );
    assert_eq!(ast, expected_ast);

    let intermediate_ast = SelectStatementParser::new()
        .parse("select salary as s, name, salary as d from sxt_tab order by s desc")
        .unwrap();
    assert!(Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor, t.schema_id())
        .is_ok());

    let intermediate_ast = SelectStatementParser::new()
        .parse("select salary as s, name, salary as d from sxt_tab order by d desc")
        .unwrap();
    assert!(Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor, t.schema_id())
        .is_ok());

    let intermediate_ast = SelectStatementParser::new()
        .parse("select salary as s, name, salary as s from sxt_tab order by s desc")
        .unwrap();
    assert!(Converter::default()
        .visit_intermediate_ast(&intermediate_ast, &accessor, t.schema_id())
        .is_ok());
}
