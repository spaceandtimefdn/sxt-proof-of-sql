use super::{
    FinalRoundBuilder, ProofPlan, ProverEvaluate, VerifiableQueryResult, VerificationBuilder,
};
use crate::{
    base::{
        commitment::InnerProductProof,
        database::{
            owned_table_utility::{bigint, owned_table},
            table_utility::*,
            ColumnField, ColumnRef, ColumnType, LiteralValue, OwnedTable, OwnedTableTestAccessor,
            Table, TableEvaluation, TableRef,
        },
        map::{indexset, IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::proof::{FirstRoundBuilder, QueryData},
};
use bumpalo::Bump;
use serde::Serialize;
use sqlparser::ast::Ident;

#[derive(Debug, Serialize, Default)]
pub(super) struct EmptyTestQueryExpr {
    pub(super) length: usize,
    pub(super) columns: usize,
}
impl ProverEvaluate for EmptyTestQueryExpr {
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        _table_map: &IndexMap<TableRef, Table<'a, S>>,
        _params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        let zeros = vec![0_i64; self.length];
        builder.produce_chi_evaluation_length(self.length);
        Ok(table_with_row_count(
            (1..=self.columns)
                .map(|i| borrowed_bigint(format!("a{i}").as_str(), zeros.clone(), alloc)),
            self.length,
        ))
    }

    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        _table_map: &IndexMap<TableRef, Table<'a, S>>,
        _params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        let zeros = vec![0_i64; self.length];
        let res: &[_] = alloc.alloc_slice_copy(&zeros);
        let _ = std::iter::repeat_with(|| builder.produce_intermediate_mle(res))
            .take(self.columns)
            .collect::<Vec<_>>();
        Ok(table_with_row_count(
            (1..=self.columns)
                .map(|i| borrowed_bigint(format!("a{i}").as_str(), zeros.clone(), alloc)),
            self.length,
        ))
    }
}
impl ProofPlan for EmptyTestQueryExpr {
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        _accessor: &IndexMap<TableRef, IndexMap<Ident, S>>,
        _result: Option<&OwnedTable<S>>,
        _chi_eval_map: &IndexMap<TableRef, (S, usize)>,
        _params: &[LiteralValue],
    ) -> Result<TableEvaluation<S>, ProofError> {
        assert_eq!(
            builder.try_consume_final_round_mle_evaluations(self.columns)?,
            vec![S::ZERO; self.columns]
        );
        Ok(TableEvaluation::new(
            vec![S::ZERO; self.columns],
            builder.try_consume_chi_evaluation()?,
        ))
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        (1..=self.columns)
            .map(|i| ColumnField::new(format!("a{i}").as_str().into(), ColumnType::BigInt))
            .collect()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        indexset! {}
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        indexset![TableRef::new("sxt", "test")]
    }
}

#[test]
fn we_can_verify_queries_on_an_empty_table() {
    let expr = EmptyTestQueryExpr {
        columns: 1,
        ..Default::default()
    };
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        TableRef::new("sxt", "test"),
        owned_table([bigint("a1", [0_i64; 0])]),
        0,
        (),
    );
    let res = VerifiableQueryResult::<InnerProductProof>::new(&expr, &accessor, &(), &[]).unwrap();
    let QueryData {
        verification_hash: _,
        table,
    } = res.verify(&expr, &accessor, &(), &[]).unwrap();
    let expected_res = owned_table([bigint("a1", [0; 0])]);
    assert_eq!(table, expected_res);
}
