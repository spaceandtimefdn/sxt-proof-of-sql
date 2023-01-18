use crate::base::database::{ColumnRef, ColumnType, TableRef};
use crate::sql::ast::EqualsExpr;
use crate::sql::ast::FilterExpr;
use crate::sql::ast::FilterResultExpr;
use crate::sql::ast::TableExpr;
use crate::sql::ast::{AndExpr, NotExpr, OrExpr};
use crate::sql::proof::QueryExpr;
use proofs_sql::{Identifier, ResourceId};

use arrow::datatypes::Field;
use arrow::datatypes::Schema;
use curve25519_dalek::scalar::Scalar;
use std::collections::HashSet;
use std::sync::Arc;

#[test]
fn we_can_correctly_fetch_the_query_result_schema() {
    let table_ref = TableRef::new(ResourceId::try_new("sxt", "sxt_tab").unwrap());
    let provable_ast = FilterExpr::new(
        vec![
            FilterResultExpr::new(
                ColumnRef::new(
                    table_ref.clone(),
                    Identifier::try_new("a").unwrap(),
                    ColumnType::BigInt,
                ),
                "a".to_string(),
            ),
            FilterResultExpr::new(
                ColumnRef::new(
                    table_ref.clone(),
                    Identifier::try_new("b").unwrap(),
                    ColumnType::BigInt,
                ),
                "b".to_string(),
            ),
        ],
        TableExpr {
            table_ref: table_ref.clone(),
        },
        Box::new(EqualsExpr::new(
            ColumnRef::new(
                table_ref,
                Identifier::try_new("c").unwrap(),
                ColumnType::BigInt,
            ),
            Scalar::from(123_u64),
        )),
    );

    let result_schema = provable_ast.get_result_schema();

    assert_eq!(
        result_schema,
        Arc::new(Schema::new(vec![
            Field::new("a", (&ColumnType::BigInt).into(), false,),
            Field::new("b", (&ColumnType::BigInt).into(), false,)
        ]))
    );
}

#[test]
fn we_can_correctly_fetch_all_the_referenced_columns() {
    let table_ref = TableRef::new(ResourceId::try_new("sxt", "sxt_tab").unwrap());
    let provable_ast = FilterExpr::new(
        vec![
            FilterResultExpr::new(
                ColumnRef::new(
                    table_ref.clone(),
                    Identifier::try_new("a").unwrap(),
                    ColumnType::BigInt,
                ),
                "a".to_string(),
            ),
            FilterResultExpr::new(
                ColumnRef::new(
                    table_ref.clone(),
                    Identifier::try_new("f").unwrap(),
                    ColumnType::BigInt,
                ),
                "f".to_string(),
            ),
        ],
        TableExpr {
            table_ref: table_ref.clone(),
        },
        Box::new(NotExpr::new(Box::new(AndExpr::new(
            Box::new(OrExpr::new(
                Box::new(EqualsExpr::new(
                    ColumnRef::new(
                        table_ref.clone(),
                        Identifier::try_new("f").unwrap(),
                        ColumnType::BigInt,
                    ),
                    Scalar::from(45_u64),
                )),
                Box::new(EqualsExpr::new(
                    ColumnRef::new(
                        table_ref.clone(),
                        Identifier::try_new("c").unwrap(),
                        ColumnType::BigInt,
                    ),
                    -Scalar::from(2_u64),
                )),
            )),
            Box::new(EqualsExpr::new(
                ColumnRef::new(
                    table_ref.clone(),
                    Identifier::try_new("b").unwrap(),
                    ColumnType::BigInt,
                ),
                Scalar::from(3_u64),
            )),
        )))),
    );

    let ref_columns = provable_ast.get_column_references();

    assert_eq!(
        ref_columns,
        HashSet::from([
            ColumnRef::new(
                table_ref.clone(),
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt
            ),
            ColumnRef::new(
                table_ref.clone(),
                Identifier::try_new("f").unwrap(),
                ColumnType::BigInt
            ),
            ColumnRef::new(
                table_ref.clone(),
                Identifier::try_new("c").unwrap(),
                ColumnType::BigInt
            ),
            ColumnRef::new(
                table_ref,
                Identifier::try_new("b").unwrap(),
                ColumnType::BigInt
            )
        ])
    );
}
