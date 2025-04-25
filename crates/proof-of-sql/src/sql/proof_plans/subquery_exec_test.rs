use crate::{
    base::{
        database::{
            owned_table_utility::{bigint, decimal75, owned_table, tinyint},
            ColumnField, ColumnRef, ColumnType, OwnedTableTestAccessor, TableRef, TestAccessor,
        },
        math::decimal::Precision,
    },
    sql::{
        proof::{ProofPlan, VerifiableQueryResult},
        proof_exprs::{
            test_utility::{aliased_plan, cols_expr, column, const_bool, multiply, sum_expr, tab},
            ColumnExpr, DynProofExpr,
        },
        proof_plans::{
            subquery_exec::SubqueryExec,
            test_utility::{group_by, projection, sort_merge_join, table_exec},
            DynProofPlan, TableExec,
        },
    },
};
use blitzar::proof::InnerProductProof;
use sqlparser::ast::Ident;

#[test]
fn we_can_correctly_fetch_inputs_and_outputs() {
    let table_ref = TableRef::new("sxt", "sxt_tab");
    let subquery_alias = TableRef::new("", "query_alias");
    let a = Ident::new("a");
    let b = Ident::new("b");
    let provable_ast = SubqueryExec::new(
        subquery_alias.clone(),
        Box::new(projection(
            vec![
                aliased_plan(
                    DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
                        table_ref.clone(),
                        a.clone(),
                        ColumnType::BigInt,
                    ))),
                    "alias_a",
                ),
                aliased_plan(
                    DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
                        table_ref.clone(),
                        b.clone(),
                        ColumnType::BigInt,
                    ))),
                    "alias_b",
                ),
            ],
            DynProofPlan::Table(TableExec::new(
                table_ref.clone(),
                vec![
                    ColumnField::new("a".into(), ColumnType::BigInt),
                    ColumnField::new("b".into(), ColumnType::BigInt),
                ],
            )),
        )),
    );
    let input_fields = provable_ast.get_column_references();
    assert_eq!(
        input_fields,
        [
            ColumnRef::new(table_ref.clone(), a, ColumnType::BigInt),
            ColumnRef::new(table_ref.clone(), b, ColumnType::BigInt),
            ColumnRef::new(subquery_alias.clone(), "alias_a".into(), ColumnType::BigInt),
            ColumnRef::new(subquery_alias, "alias_b".into(), ColumnType::BigInt)
        ]
        .into()
    );
    let output_fields = provable_ast.get_column_result_fields();
    assert_eq!(
        output_fields,
        vec![
            ColumnField::new("alias_a".into(), ColumnType::BigInt),
            ColumnField::new("alias_b".into(), ColumnType::BigInt),
        ]
    );
}

#[test]
fn we_can_join_to_a_group_by() {
    let rating_id_ident = "rating_id";
    let stars_ident = "stars";
    let count_ident = "_count_";
    let weight_ident = "weight";
    let stars_sum = "stars_sum";
    let stars_average = "stars_average";
    let rating_table = owned_table([
        bigint(rating_id_ident, [1, 2, 2, 4, 3, 3, 2, 2, 2]),
        decimal75(stars_ident, 2, 0, [9, 5, 7, 8, 5, 6, 7, 8, 6]),
    ]);
    let weight_table = owned_table([
        tinyint(count_ident, [1, 2]),
        decimal75(weight_ident, 2, 1, [10, 5]),
    ]);
    let rating_table_ref = TableRef::new("sxt", "rating");
    let weight_table_ref = TableRef::new("", "weight_lookup");
    let rater_rating_sum_ref = TableRef::new("", "rater_rating_sum");
    let mut data_accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    data_accessor.add_table(rating_table_ref.clone(), rating_table, 0);
    data_accessor.add_table(weight_table_ref.clone(), weight_table, 0);
    let expr = projection(
        vec![aliased_plan(
            multiply(
                column(&weight_table_ref, weight_ident, &data_accessor),
                column(&rater_rating_sum_ref, stars_sum, &data_accessor),
            ),
            stars_average,
        )],
        sort_merge_join(
            group_by(
                cols_expr(&rating_table_ref, &[rating_id_ident], &data_accessor),
                vec![sum_expr(
                    column(&rating_table_ref, stars_ident, &data_accessor),
                    stars_sum,
                )],
                count_ident,
                tab(&rating_table_ref),
                const_bool(true),
            ),
            table_exec(
                weight_table_ref,
                vec![
                    ColumnField::new(count_ident.into(), ColumnType::TinyInt),
                    ColumnField::new(
                        weight_ident.into(),
                        ColumnType::Decimal75(Precision::new(2).unwrap(), 1),
                    ),
                ],
            ),
            vec![0],
            vec![0],
            vec![
                rating_id_ident.into(),
                stars_sum.into(),
                count_ident.into(),
                weight_ident.into(),
            ],
        ),
    );
    let res: VerifiableQueryResult<InnerProductProof> =
        VerifiableQueryResult::new(&expr, &data_accessor, &(), &[]).unwrap();
    let result_table = res.verify(&expr, &data_accessor, &(), &[]).unwrap().table;
    let expected = owned_table([decimal75(stars_average, 3, 1, [90, 80, 55])]);
    assert_eq!(result_table, expected);
}
