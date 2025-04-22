use super::{plans::EVMDynProofPlan, EVMProofPlanError, EVMProofPlanResult};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::str::FromStr;
use itertools::Itertools;
use indexmap::{IndexMap, IndexSet};
use proof_of_sql::{
    base::database::{
        ColumnField, ColumnRef, ColumnType, TableRef,
    },
    sql::{
        proof::ProofPlan,
        proof_plans::DynProofPlan,
    },
};
use serde::{Deserialize, Serialize, Serializer};
use sqlparser::ast::Ident;

#[derive(Debug)]
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
        let table_refs = value.inner.get_table_references();
        let column_refs = value.inner.get_column_references();
        let output_column_names = value
            .inner
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
                &output_column_names,
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