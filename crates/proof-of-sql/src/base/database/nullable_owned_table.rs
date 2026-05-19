use super::{
    ColumnField, NullableOwnedColumn, NullableTable, OwnedColumn, OwnedTable, OwnedTableError,
};
use crate::base::{map::IndexMap, scalar::Scalar};
use alloc::{format, vec::Vec};
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// An owned table whose columns can carry nullable row-presence data.
#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub struct NullableOwnedTable<S: Scalar> {
    table: IndexMap<Ident, NullableOwnedColumn<S>>,
}

impl<S: Scalar> NullableOwnedTable<S> {
    /// Creates a new [`NullableOwnedTable`].
    pub fn try_new(
        table: IndexMap<Ident, NullableOwnedColumn<S>>,
    ) -> Result<Self, OwnedTableError> {
        if table.is_empty() {
            return Ok(Self { table });
        }
        let num_rows = table[0].len();
        if table.values().any(|column| column.len() != num_rows) {
            Err(OwnedTableError::ColumnLengthMismatch)
        } else {
            Ok(Self { table })
        }
    }

    /// Creates a new [`NullableOwnedTable`] from an iterator.
    pub fn try_from_iter<T: IntoIterator<Item = (Ident, NullableOwnedColumn<S>)>>(
        iter: T,
    ) -> Result<Self, OwnedTableError> {
        Self::try_new(IndexMap::from_iter(iter))
    }

    /// Number of columns in the table.
    #[must_use]
    pub fn num_columns(&self) -> usize {
        self.table.len()
    }

    /// Number of rows in the table.
    #[must_use]
    pub fn num_rows(&self) -> usize {
        if self.table.is_empty() {
            0
        } else {
            self.table[0].len()
        }
    }

    /// Whether the table has no columns.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Returns the columns of this table.
    #[must_use]
    pub fn into_inner(self) -> IndexMap<Ident, NullableOwnedColumn<S>> {
        self.table
    }

    /// Returns the columns of this table by reference.
    #[must_use]
    pub const fn inner_table(&self) -> &IndexMap<Ident, NullableOwnedColumn<S>> {
        &self.table
    }

    /// Return the schema of this table as a `Vec` of `ColumnField`s.
    #[must_use]
    pub fn schema(&self) -> Vec<ColumnField> {
        self.table
            .iter()
            .map(|(name, column)| {
                if column.is_nullable() {
                    ColumnField::new_nullable(name.clone(), column.values().column_type())
                } else {
                    ColumnField::new(name.clone(), column.values().column_type())
                }
            })
            .collect()
    }

    /// Returns the column names as an iterator.
    pub fn column_names(&self) -> impl Iterator<Item = &Ident> {
        self.table.keys()
    }

    /// Returns the column with the given position.
    #[must_use]
    pub fn column_by_index(&self, index: usize) -> Option<&NullableOwnedColumn<S>> {
        self.table.get_index(index).map(|(_, v)| v)
    }

    /// Returns the generated proof-column name for a nullable column's row-presence data.
    #[must_use]
    pub fn presence_column_name(column_name: &Ident) -> Ident {
        Ident::new(format!("__posql_presence_{}", column_name.value))
    }

    /// Returns a non-null proof table with backing values plus presence columns for nullable data.
    #[must_use]
    pub fn values_and_presence_table(&self) -> OwnedTable<S> {
        OwnedTable::try_from_iter(self.table.iter().flat_map(|(name, column)| {
            let value_column = (name.clone(), column.values().clone());
            let presence_column = column.presence().map(|presence| {
                (
                    Self::presence_column_name(name),
                    OwnedColumn::Boolean(presence.to_vec()),
                )
            });
            core::iter::once(value_column).chain(presence_column)
        }))
        .expect("Generated value and presence columns should have matching lengths")
    }

    /// Returns the schema for [`Self::values_and_presence_table`].
    #[must_use]
    pub fn values_and_presence_schema(&self) -> Vec<ColumnField> {
        self.table
            .iter()
            .flat_map(|(name, column)| {
                let value_field = if column.is_nullable() {
                    ColumnField::new_nullable(name.clone(), column.values().column_type())
                } else {
                    ColumnField::new(name.clone(), column.values().column_type())
                };
                let presence_field = column.presence().map(|_| {
                    ColumnField::new(Self::presence_column_name(name), super::ColumnType::Boolean)
                });
                core::iter::once(value_field).chain(presence_field)
            })
            .collect()
    }
}

impl<S: Scalar> PartialEq for NullableOwnedTable<S> {
    fn eq(&self, other: &Self) -> bool {
        self.table == other.table
            && self
                .table
                .keys()
                .zip(other.table.keys())
                .all(|(a, b)| a == b)
    }
}

impl<'a, S: Scalar> From<NullableTable<'a, S>> for NullableOwnedTable<S> {
    fn from(value: NullableTable<'a, S>) -> Self {
        NullableOwnedTable::from(&value)
    }
}

#[cfg(test)]
impl<S: Scalar> core::ops::Index<&str> for NullableOwnedTable<S> {
    type Output = NullableOwnedColumn<S>;

    fn index(&self, index: &str) -> &Self::Output {
        self.table.get(&Ident::new(index)).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{
        commitment::naive_evaluation_proof::NaiveEvaluationProof,
        database::{
            owned_table_utility::{bigint, owned_table},
            Column, ColumnRef, ColumnType, LiteralValue, NullableColumn, OwnedColumn,
            OwnedTableTestAccessor, TableRef,
        },
        map::indexmap,
        scalar::test_scalar::TestScalar,
    };
    use crate::sql::{
        proof::VerifiableQueryResult,
        proof_exprs::{AliasedDynProofExpr, DynProofExpr},
        proof_plans::test_utility::{filter, table_exec},
    };
    use alloc::vec;

    #[test]
    fn nullable_owned_table_rejects_column_length_mismatches() {
        let result = NullableOwnedTable::try_new(indexmap! {
            "id".into() => NullableOwnedColumn::<TestScalar>::new_nonnullable(
                OwnedColumn::BigInt(vec![1, 2])
            ),
            "amount".into() => NullableOwnedColumn::<TestScalar>::try_new(
                OwnedColumn::BigInt(vec![10]),
                Some(vec![true])
            ).unwrap(),
        });

        assert_eq!(result, Err(OwnedTableError::ColumnLengthMismatch));
    }

    #[test]
    fn nullable_table_converts_to_owned_table() {
        let borrowed = NullableTable::try_new(indexmap! {
            "id".into() => NullableColumn::<TestScalar>::new_nonnullable(Column::BigInt(&[1, 2])),
            "amount".into() => NullableColumn::<TestScalar>::try_new(
                Column::BigInt(&[10, 20]),
                Some(&[true, false])
            ).unwrap(),
        })
        .unwrap();

        let owned = NullableOwnedTable::from(&borrowed);

        assert_eq!(
            owned["id"],
            NullableOwnedColumn::new_nonnullable(OwnedColumn::BigInt(vec![1, 2]))
        );
        assert_eq!(
            owned["amount"],
            NullableOwnedColumn::try_new(
                OwnedColumn::BigInt(vec![10, 20]),
                Some(vec![true, false])
            )
            .unwrap()
        );
        assert_eq!(owned.num_rows(), 2);
        assert_eq!(owned.num_columns(), 2);
    }

    #[test]
    fn nullable_table_expands_to_values_and_presence_columns() {
        let nullable_table = nullable_table_for_proof();

        let proof_table = nullable_table.values_and_presence_table();
        let proof_schema = nullable_table.values_and_presence_schema();

        assert_eq!(
            proof_table["amount"],
            OwnedColumn::<TestScalar>::BigInt(vec![10, 0, 30, 50])
        );
        assert_eq!(
            proof_table["__posql_presence_amount"],
            OwnedColumn::<TestScalar>::Boolean(vec![true, false, true, true])
        );
        assert_eq!(proof_schema.len(), 3);
        assert!(proof_schema[1].is_nullable());
        assert_eq!(
            proof_schema[2].name(),
            Ident::new("__posql_presence_amount")
        );
        assert_eq!(proof_schema[2].data_type(), ColumnType::Boolean);
    }

    #[test]
    fn nullable_presence_columns_can_drive_a_non_trivial_query_proof() {
        let table_ref = TableRef::new("sxt", "nullable");
        let nullable_table = nullable_table_for_proof();
        let schema = nullable_table.values_and_presence_schema();
        let proof_table = nullable_table.values_and_presence_table();
        let accessor = OwnedTableTestAccessor::<NaiveEvaluationProof>::new_from_table(
            table_ref.clone(),
            proof_table,
            0,
            (),
        );
        let amount_ref =
            ColumnRef::new_nullable(table_ref.clone(), Ident::new("amount"), ColumnType::BigInt);
        let amount_presence_ref = ColumnRef::new(
            table_ref.clone(),
            NullableOwnedTable::<TestScalar>::presence_column_name(&Ident::new("amount")),
            ColumnType::Boolean,
        );
        let amount_gt_15 = DynProofExpr::try_new_inequality(
            DynProofExpr::new_column(amount_ref.clone()),
            DynProofExpr::new_literal(LiteralValue::BigInt(15)),
            false,
        )
        .unwrap();
        let where_clause =
            DynProofExpr::try_new_and(DynProofExpr::new_column(amount_presence_ref), amount_gt_15)
                .unwrap();
        let plan = filter(
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::new_column(amount_ref),
                alias: Ident::new("amount"),
            }],
            table_exec(table_ref, schema),
            where_clause,
        );

        let verifiable_result =
            VerifiableQueryResult::<NaiveEvaluationProof>::new(&plan, &accessor, &(), &[]).unwrap();
        let verified = verifiable_result
            .verify(&plan, &accessor, &(), &[])
            .unwrap();

        assert_eq!(
            verified.table,
            owned_table([bigint("amount", [30_i64, 50])])
        );
    }

    fn nullable_table_for_proof() -> NullableOwnedTable<TestScalar> {
        NullableOwnedTable::try_new(indexmap! {
            "id".into() => NullableOwnedColumn::new_nonnullable(
                OwnedColumn::<TestScalar>::BigInt(vec![1, 2, 3, 4])
            ),
            "amount".into() => NullableOwnedColumn::try_new(
                OwnedColumn::<TestScalar>::BigInt(vec![10, 0, 30, 50]),
                Some(vec![true, false, true, true])
            ).unwrap(),
        })
        .unwrap()
    }
}
