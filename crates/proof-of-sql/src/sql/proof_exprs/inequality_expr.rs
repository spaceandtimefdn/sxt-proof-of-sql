use super::{add_subtract_columns, DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{try_inequality_types, Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{FinalRoundBuilder, VerificationBuilder},
        proof_gadgets::{
            final_round_evaluate_sign, first_round_evaluate_sign, verifier_evaluate_sign,
        },
        AnalyzeError, AnalyzeResult,
    },
    utils::log,
};
use alloc::{boxed::Box, string::ToString};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Provable AST expression for an inequality expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InequalityExpr {
    lhs: Box<DynProofExpr>,
    rhs: Box<DynProofExpr>,
    is_lt: bool,
}

impl InequalityExpr {
    /// Create a new less than or equal
    pub fn try_new(
        lhs: Box<DynProofExpr>,
        rhs: Box<DynProofExpr>,
        is_lt: bool,
    ) -> AnalyzeResult<Self> {
        let left_datatype = lhs.data_type();
        let right_datatype = rhs.data_type();
        try_inequality_types(left_datatype, right_datatype)
            .map(|()| Self { lhs, rhs, is_lt })
            .map_err(|_| AnalyzeError::DataTypeMismatch {
                left_type: left_datatype.to_string(),
                right_type: right_datatype.to_string(),
            })
    }

    /// Get the left-hand side expression
    pub fn lhs(&self) -> &DynProofExpr {
        &self.lhs
    }

    /// Get the right-hand side expression
    pub fn rhs(&self) -> &DynProofExpr {
        &self.rhs
    }

    /// Get whether this is a less-than comparison
    pub fn is_lt(&self) -> bool {
        self.is_lt
    }
}

impl ProofExpr for InequalityExpr {
    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean
    }

    #[tracing::instrument(
        name = "InequalityExpr::first_round_evaluate",
        level = "debug",
        skip_all
    )]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let lhs_column = self.lhs.first_round_evaluate(alloc, table, params)?;
        let rhs_column = self.rhs.first_round_evaluate(alloc, table, params)?;
        let table_length = table.num_rows();
        let diff = if self.is_lt {
            add_subtract_columns(lhs_column, rhs_column, alloc, true)
        } else {
            add_subtract_columns(rhs_column, lhs_column, alloc, true)
        };

        // (sign(diff) == -1)
        let res = Column::Boolean(first_round_evaluate_sign(table_length, alloc, diff));

        log::log_memory_usage("End");

        Ok(res)
    }

    #[tracing::instrument(
        name = "InequalityExpr::final_round_evaluate",
        level = "debug",
        skip_all
    )]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let lhs_column = self
            .lhs
            .final_round_evaluate(builder, alloc, table, params)?;
        let rhs_column = self
            .rhs
            .final_round_evaluate(builder, alloc, table, params)?;
        let diff = if self.is_lt {
            add_subtract_columns(lhs_column, rhs_column, alloc, true)
        } else {
            add_subtract_columns(rhs_column, lhs_column, alloc, true)
        };

        // (sign(diff) == -1)
        let res = Column::Boolean(final_round_evaluate_sign(builder, alloc, diff));

        log::log_memory_usage("End");

        Ok(res)
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<Ident, S>,
        chi_eval: S,
        params: &[LiteralValue],
    ) -> Result<S, ProofError> {
        let lhs_eval = self
            .lhs
            .verifier_evaluate(builder, accessor, chi_eval, params)?;
        let rhs_eval = self
            .rhs
            .verifier_evaluate(builder, accessor, chi_eval, params)?;
        let diff_eval = if self.is_lt {
            lhs_eval - rhs_eval
        } else {
            rhs_eval - lhs_eval
        };

        // sign(diff) == -1
        verifier_evaluate_sign(builder, diff_eval, chi_eval, None)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}

#[cfg(test)]
mod tests_inequality {
    use crate::{
        base::database::{ColumnType, LiteralValue},
        sql::proof_exprs::{DynProofExpr, InequalityExpr, ProofExpr},
    };

    fn bigint_expr(n: i64) -> DynProofExpr {
        DynProofExpr::new_literal(LiteralValue::BigInt(n))
    }
    fn bool_expr() -> DynProofExpr {
        DynProofExpr::new_literal(LiteralValue::Boolean(true))
    }

    #[test]
    fn try_new_with_numerics_returns_ok() {
        assert!(InequalityExpr::try_new(
            alloc::boxed::Box::new(bigint_expr(1)),
            alloc::boxed::Box::new(bigint_expr(2)),
            true,
        )
        .is_ok());
    }

    #[test]
    fn try_new_with_incompatible_types_returns_err() {
        assert!(InequalityExpr::try_new(
            alloc::boxed::Box::new(bigint_expr(1)),
            alloc::boxed::Box::new(bool_expr()),
            false,
        )
        .is_err());
    }

    #[test]
    fn data_type_is_boolean() {
        let e = InequalityExpr::try_new(
            alloc::boxed::Box::new(bigint_expr(0)),
            alloc::boxed::Box::new(bigint_expr(1)),
            true,
        )
        .unwrap();
        assert_eq!(e.data_type(), ColumnType::Boolean);
    }

    #[test]
    fn is_lt_stores_flag_correctly() {
        let lt = InequalityExpr::try_new(
            alloc::boxed::Box::new(bigint_expr(0)),
            alloc::boxed::Box::new(bigint_expr(1)),
            true,
        )
        .unwrap();
        let lte = InequalityExpr::try_new(
            alloc::boxed::Box::new(bigint_expr(0)),
            alloc::boxed::Box::new(bigint_expr(1)),
            false,
        )
        .unwrap();
        assert!(lt.is_lt());
        assert!(!lte.is_lt());
    }

    #[test]
    fn lhs_has_correct_type() {
        let e = InequalityExpr::try_new(
            alloc::boxed::Box::new(bigint_expr(3)),
            alloc::boxed::Box::new(bigint_expr(5)),
            true,
        )
        .unwrap();
        assert_eq!(e.lhs().data_type(), ColumnType::BigInt);
    }

    #[test]
    fn equality_holds_for_same_values() {
        let a = InequalityExpr::try_new(
            alloc::boxed::Box::new(bigint_expr(1)),
            alloc::boxed::Box::new(bigint_expr(2)),
            true,
        )
        .unwrap();
        let b = InequalityExpr::try_new(
            alloc::boxed::Box::new(bigint_expr(1)),
            alloc::boxed::Box::new(bigint_expr(2)),
            true,
        )
        .unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn debug_contains_struct_name() {
        let e = InequalityExpr::try_new(
            alloc::boxed::Box::new(bigint_expr(0)),
            alloc::boxed::Box::new(bigint_expr(1)),
            false,
        )
        .unwrap();
        assert!(alloc::format!("{e:?}").contains("InequalityExpr"));
    }
}
