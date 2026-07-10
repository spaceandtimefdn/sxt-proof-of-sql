use super::{plans::EVMDynProofPlan, EVMProofPlanError, EVMProofPlanResult};
use crate::{
    base::{
        database::{
            ColumnField, ColumnRef, ColumnType, LiteralValue, Table, TableEvaluation, TableRef,
        },
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{
            FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate, VerificationBuilder,
        },
        proof_plans::DynProofPlan,
    },
};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use bumpalo::Bump;
use core::str::FromStr;
use itertools::Itertools;
use serde::{Deserialize, Serialize, Serializer};
use sqlparser::ast::Ident;

#[derive(Debug, PartialEq, Clone)]
/// An implementation of `ProofPlan` that allows for EVM compatible serialization.
/// Serialization should be done using bincode with fixint, big-endian encoding in order to be compatible with EVM.
///
/// This is simply a wrapper around a `DynProofPlan`.
pub struct EVMProofPlan {
    inner: DynProofPlan,
}

impl EVMProofPlan {
    /// Create a new `EVMProofPlan` from a `DynProofPlan`.
    #[must_use]
    pub fn new(plan: DynProofPlan) -> Self {
        Self { inner: plan }
    }
    /// Get the inner `DynProofPlan`.
    #[must_use]
    pub fn into_inner(self) -> DynProofPlan {
        self.inner
    }
    /// Get a reference to the inner `DynProofPlan`.
    #[must_use]
    pub fn inner(&self) -> &DynProofPlan {
        &self.inner
    }
}

#[derive(Serialize, Deserialize)]
struct CompactPlan {
    tables: Vec<String>,
    columns: Vec<(usize, String, ColumnType)>,
    output_column_names: Vec<String>,
    plan: EVMDynProofPlan,
}

impl TryFrom<&EVMProofPlan> for CompactPlan {
    type Error = EVMProofPlanError;

    fn try_from(value: &EVMProofPlan) -> Result<Self, Self::Error> {
        let table_refs = value.get_table_references();
        let column_refs = value.get_column_references();
        let output_column_names = value
            .get_column_result_fields()
            .iter()
            .map(|field| field.name().to_string())
            .collect();

        let plan = EVMDynProofPlan::try_from_proof_plan(value.inner(), &table_refs, &column_refs)?;
        let columns = column_refs
            .into_iter()
            .map(|column_ref| -> EVMProofPlanResult<_> {
                let table_index = table_refs
                    .get_index_of(&column_ref.table_ref())
                    .ok_or(EVMProofPlanError::TableNotFound)?;
                Ok((
                    table_index,
                    column_ref.column_id().to_string(),
                    *column_ref.column_type(),
                ))
            })
            .try_collect()?;
        let tables = table_refs.iter().map(ToString::to_string).collect();

        Ok(Self {
            tables,
            columns,
            output_column_names,
            plan,
        })
    }
}

impl TryFrom<CompactPlan> for EVMProofPlan {
    type Error = EVMProofPlanError;

    fn try_from(value: CompactPlan) -> Result<Self, Self::Error> {
        let table_refs: IndexSet<TableRef> = value
            .tables
            .iter()
            .map(|table| TableRef::from_str(table).map_err(|_| EVMProofPlanError::InvalidTableName))
            .try_collect()?;
        let table_refs_clone = table_refs.clone();
        let column_refs: IndexSet<ColumnRef> = value
            .columns
            .iter()
            .map(|(i, ident, column_type)| -> EVMProofPlanResult<_> {
                let table_ref = table_refs_clone
                    .get_index(*i)
                    .cloned()
                    .ok_or(EVMProofPlanError::TableNotFound)?;
                Ok(ColumnRef::new(table_ref, Ident::new(ident), *column_type))
            })
            .try_collect()?;
        let output_column_names: IndexSet<String> = value.output_column_names.into_iter().collect();
        Ok(Self {
            inner: value.plan.try_into_proof_plan(
                &table_refs,
                &column_refs,
                Some(&output_column_names),
            )?,
        })
    }
}

impl Serialize for EVMProofPlan {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        CompactPlan::try_from(self)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for EVMProofPlan {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        CompactPlan::deserialize(deserializer)?
            .try_into()
            .map_err(serde::de::Error::custom)
    }
}

impl ProofPlan for EVMProofPlan {
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<TableRef, IndexMap<Ident, S>>,
        chi_eval_map: &IndexMap<TableRef, (S, usize)>,
        params: &[LiteralValue],
    ) -> Result<TableEvaluation<S>, ProofError> {
        self.inner()
            .verifier_evaluate(builder, accessor, chi_eval_map, params)
    }
    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.inner().get_column_result_fields()
    }
    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        self.inner().get_column_references()
    }
    fn get_table_references(&self) -> IndexSet<TableRef> {
        self.inner().get_table_references()
    }
}
impl ProverEvaluate for EVMProofPlan {
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        self.inner()
            .first_round_evaluate(builder, alloc, table_map, params)
    }
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Table<'a, S>> {
        self.inner()
            .final_round_evaluate(builder, alloc, table_map, params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::{
            database::{table_utility::*, ColumnField},
            map::{indexmap, indexset},
            scalar::test_scalar::TestScalar,
        },
        sql::{
            evm_proof_plan::plans::EVMEmptyExec,
            proof::{
                mock_verification_builder::MockVerificationBuilder, FinalRoundBuilder,
                FirstRoundBuilder,
            },
            proof_plans::{EmptyExec, TableExec},
        },
    };
    use alloc::collections::VecDeque;

    fn test_table_plan() -> (TableRef, EVMProofPlan) {
        let table_ref = TableRef::new("namespace", "table");
        let plan = DynProofPlan::Table(TableExec::new(
            table_ref.clone(),
            vec![ColumnField::new("a".into(), ColumnType::BigInt)],
        ));
        (table_ref, EVMProofPlan::new(plan))
    }

    #[test]
    fn evm_proof_plan_exposes_and_consumes_inner_plan() {
        let plan = DynProofPlan::Empty(EmptyExec::new());
        let evm_plan = EVMProofPlan::new(plan.clone());

        assert_eq!(evm_plan.inner(), &plan);
        assert_eq!(evm_plan.into_inner(), plan);
    }

    #[test]
    fn compact_plan_rejects_invalid_table_names() {
        let compact_plan = CompactPlan {
            tables: vec!["too.many.parts".to_string()],
            columns: Vec::new(),
            output_column_names: Vec::new(),
            plan: EVMDynProofPlan::Empty(EVMEmptyExec {}),
        };

        assert_eq!(
            EVMProofPlan::try_from(compact_plan).unwrap_err(),
            EVMProofPlanError::InvalidTableName
        );
    }

    #[test]
    fn compact_plan_rejects_columns_with_missing_table_indexes() {
        let compact_plan = CompactPlan {
            tables: vec![TableRef::new("namespace", "table").to_string()],
            columns: vec![(1, "a".to_string(), ColumnType::BigInt)],
            output_column_names: Vec::new(),
            plan: EVMDynProofPlan::Empty(EVMEmptyExec {}),
        };

        assert_eq!(
            EVMProofPlan::try_from(compact_plan).unwrap_err(),
            EVMProofPlanError::TableNotFound
        );
    }

    #[test]
    fn evm_proof_plan_delegates_metadata_and_verifier_evaluation() {
        let (table_ref, evm_plan) = test_table_plan();
        let column_ref = ColumnRef::new(table_ref.clone(), "a".into(), ColumnType::BigInt);

        assert_eq!(
            evm_plan.get_column_result_fields(),
            vec![ColumnField::new("a".into(), ColumnType::BigInt)]
        );
        assert_eq!(evm_plan.get_column_references(), indexset! { column_ref });
        assert_eq!(
            evm_plan.get_table_references(),
            indexset! { table_ref.clone() }
        );

        let accessor = indexmap! {
            table_ref.clone() => indexmap! {
                "a".into() => TestScalar::from(7),
            },
        };
        let chi_eval_map = indexmap! {
            table_ref => (TestScalar::from(11), 2),
        };
        let mut builder = MockVerificationBuilder::<TestScalar>::new(
            Vec::new(),
            0,
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        );

        let result = evm_plan
            .verifier_evaluate(&mut builder, &accessor, &chi_eval_map, &[])
            .unwrap();

        assert_eq!(result.column_evals(), &[TestScalar::from(7)]);
        assert_eq!(result.chi(), (TestScalar::from(11), 2));
    }

    #[test]
    fn evm_proof_plan_delegates_prover_evaluation_rounds() {
        let (table_ref, evm_plan) = test_table_plan();
        let alloc = Bump::new();
        let input_table = table::<TestScalar>([borrowed_bigint("a", [1_i64, 2], &alloc)]);
        let table_map = indexmap! {
            table_ref => input_table.clone(),
        };

        let mut first_round_builder = FirstRoundBuilder::new(0);
        let first_round_result = evm_plan
            .first_round_evaluate(&mut first_round_builder, &alloc, &table_map, &[])
            .unwrap();
        assert_eq!(first_round_result, input_table);

        let mut final_round_builder = FinalRoundBuilder::new(0, VecDeque::new());
        let final_round_result = evm_plan
            .final_round_evaluate(&mut final_round_builder, &alloc, &table_map, &[])
            .unwrap();
        assert_eq!(final_round_result, input_table);
    }
}
