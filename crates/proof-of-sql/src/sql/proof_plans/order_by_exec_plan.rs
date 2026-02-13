use crate::{
    base::{
        database::{
            Column, ColumnField, ColumnRef, LiteralValue, Table, TableEvaluation, TableOptions,
            TableRef,
        },
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
        PlaceholderResult,
    },
    sql::{
        proof::{
            FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate, VerificationBuilder,
        },
        proof_exprs::{DynProofExpr, ProofExpr},
        proof_plans::DynProofPlan,
    },
};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Provable expressions for queries of the form
/// ```ignore
///     SELECT ... FROM ... ORDER BY <order_by_expr>
/// ```
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct OrderByExec {
    pub(super) input: Box<DynProofPlan>,
    pub(super) order_by_exprs: Vec<DynProofExpr>,
}

impl OrderByExec {
    /// Creates a new order by expression.
    pub fn new(input: Box<DynProofPlan>, order_by_exprs: Vec<DynProofExpr>) -> Self {
        Self {
            input,
            order_by_exprs,
        }
    }
}

impl ProofPlan for OrderByExec {
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<TableRef, IndexMap<Ident, S>>,
        chi_eval_map: &IndexMap<TableRef, (S, usize)>,
        params: &[LiteralValue],
    ) -> Result<TableEvaluation<S>, ProofError> {
        let input_evals = self
            .input
            .verifier_evaluate(builder, accessor, chi_eval_map, params)?;
        let accessor = self
            .input
            .get_column_result_fields()
            .iter()
            .map(ColumnField::name)
            .zip(input_evals.column_evals().iter().copied())
            .collect::<IndexMap<_, _>>();
        let order_by_evals: Vec<S> = self
            .order_by_exprs
            .iter()
            .map(|expr| expr.verifier_evaluate(builder, &accessor, input_evals.chi_eval(), params))
            .collect::<Result<Vec<_>, _>>()?;
        let order_by_eval = order_by_evals.first().expect("Only one column is being used for now.");
        
        todo!()
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.input.get_column_result_fields()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        self.input.get_column_references()
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        self.input.get_table_references()
    }
}

impl ProverEvaluate for OrderByExec {
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        todo!()
    }

    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        todo!()
    }
}
