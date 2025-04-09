use super::{
    cast_expr::CastExpr, decimal_scaling_cast_expr::DecimalScalingCastExpr, AddSubtractExpr,
    AndExpr, ColumnExpr, EqualsExpr, InequalityExpr, LiteralExpr, MultiplyExpr, NotExpr, OrExpr,
    PlaceholderExpr, ProofExpr,
};
use crate::{
    base::{
        database::{
            try_add_subtract_column_types, try_cast_types, Column, ColumnRef, ColumnType,
            LiteralValue, Table,
        },
        map::{IndexMap, IndexSet},
        proof::{PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::{
        proof::{FinalRoundBuilder, VerificationBuilder},
        util::try_binary_operation_type,
        AnalyzeError, AnalyzeResult,
    },
};
use alloc::{boxed::Box, string::ToString};
use bumpalo::Bump;
use core::fmt::Debug;
use serde::{Deserialize, Serialize};
use sqlparser::ast::BinaryOperator;

/// Enum of AST column expression types that implement `ProofExpr`. Is itself a `ProofExpr`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[enum_dispatch::enum_dispatch]
pub enum DynProofExpr {
    /// Column
    Column(ColumnExpr),
    /// Provable logical AND expression
    And(AndExpr),
    /// Provable logical OR expression
    Or(OrExpr),
    /// Provable logical NOT expression
    Not(NotExpr),
    /// Provable CONST expression
    Literal(LiteralExpr),
    /// Provable placeholder expression
    Placeholder(PlaceholderExpr),
    /// Provable AST expression for an equals expression
    Equals(EqualsExpr),
    /// Provable AST expression for an inequality expression
    Inequality(InequalityExpr),
    /// Provable numeric `+` / `-` expression
    AddSubtract(AddSubtractExpr),
    /// Provable numeric `*` expression
    Multiply(MultiplyExpr),
    /// Provable CAST expression
    Cast(CastExpr),
    /// Provable expression for casting numeric expressions to decimal expressions
    DecimalScalingCast(DecimalScalingCastExpr),
}
impl DynProofExpr {
    /// Create column expression
    #[must_use]
    pub fn new_column(column_ref: ColumnRef) -> Self {
        Self::Column(ColumnExpr::new(column_ref))
    }
    /// Create logical AND expression
    pub fn try_new_and(lhs: DynProofExpr, rhs: DynProofExpr) -> AnalyzeResult<Self> {
        lhs.check_data_type(ColumnType::Boolean)?;
        rhs.check_data_type(ColumnType::Boolean)?;
        Ok(Self::And(AndExpr::new(Box::new(lhs), Box::new(rhs))))
    }
    /// Create logical OR expression
    pub fn try_new_or(lhs: DynProofExpr, rhs: DynProofExpr) -> AnalyzeResult<Self> {
        lhs.check_data_type(ColumnType::Boolean)?;
        rhs.check_data_type(ColumnType::Boolean)?;
        Ok(Self::Or(OrExpr::new(Box::new(lhs), Box::new(rhs))))
    }
    /// Create logical NOT expression
    pub fn try_new_not(expr: DynProofExpr) -> AnalyzeResult<Self> {
        expr.check_data_type(ColumnType::Boolean)?;
        Ok(Self::Not(NotExpr::new(Box::new(expr))))
    }
    /// Create CONST expression
    #[must_use]
    pub fn new_literal(value: LiteralValue) -> Self {
        Self::Literal(LiteralExpr::new(value))
    }
    /// Create placeholder expression
    pub fn try_new_placeholder(id: usize, column_type: ColumnType) -> AnalyzeResult<Self> {
        Ok(Self::Placeholder(PlaceholderExpr::try_new(
            id,
            column_type,
        )?))
    }
    /// Create a new equals expression
    pub fn try_new_equals(lhs: DynProofExpr, rhs: DynProofExpr) -> AnalyzeResult<Self> {
        let lhs_datatype = lhs.data_type();
        let rhs_datatype = rhs.data_type();
        if try_binary_operation_type(lhs_datatype, rhs_datatype, &BinaryOperator::Eq).is_some() {
            Ok(Self::Equals(EqualsExpr::new(Box::new(lhs), Box::new(rhs))))
        } else {
            Err(AnalyzeError::DataTypeMismatch {
                left_type: lhs_datatype.to_string(),
                right_type: rhs_datatype.to_string(),
            })
        }
    }
    /// Create a new inequality expression
    pub fn try_new_inequality(
        lhs: DynProofExpr,
        rhs: DynProofExpr,
        is_lt: bool,
    ) -> AnalyzeResult<Self> {
        let lhs_datatype = lhs.data_type();
        let rhs_datatype = rhs.data_type();
        if try_binary_operation_type(lhs_datatype, rhs_datatype, &BinaryOperator::Lt).is_some() {
            Ok(Self::Inequality(InequalityExpr::new(
                Box::new(lhs),
                Box::new(rhs),
                is_lt,
            )))
        } else {
            Err(AnalyzeError::DataTypeMismatch {
                left_type: lhs_datatype.to_string(),
                right_type: rhs_datatype.to_string(),
            })
        }
    }

    /// Create a new add expression
    pub fn try_new_add(lhs: DynProofExpr, rhs: DynProofExpr) -> AnalyzeResult<Self> {
        DynProofExpr::try_new_add_or_subtract(lhs, rhs, false)
    }

    /// Create a new subtract expression
    pub fn try_new_subtract(lhs: DynProofExpr, rhs: DynProofExpr) -> AnalyzeResult<Self> {
        DynProofExpr::try_new_add_or_subtract(lhs, rhs, true)
    }

    fn try_new_add_or_subtract(
        lhs: DynProofExpr,
        rhs: DynProofExpr,
        is_subtract: bool,
    ) -> AnalyzeResult<Self> {
        let lhs_datatype = lhs.data_type();
        let rhs_datatype = rhs.data_type();
        let result_type =
            try_add_subtract_column_types(lhs_datatype, rhs_datatype).map_err(|_| {
                AnalyzeError::DataTypeMismatch {
                    left_type: lhs_datatype.to_string(),
                    right_type: rhs_datatype.to_string(),
                }
            })?;
        let lhs = if lhs_datatype == result_type {
            lhs
        } else {
            DynProofExpr::try_new_decimal_scaling_cast(lhs, result_type)?
        };
        let rhs = if rhs_datatype == result_type {
            rhs
        } else {
            DynProofExpr::try_new_decimal_scaling_cast(rhs, result_type)?
        };
        let add_subtract_expr = AddSubtractExpr::try_new(Box::new(lhs), Box::new(rhs), is_subtract)
            .map(Self::AddSubtract)
            .map_err(|_| AnalyzeError::DataTypeMismatch {
                left_type: lhs_datatype.to_string(),
                right_type: rhs_datatype.to_string(),
            })?;
        // Casting to account for returned scalars
        DynProofExpr::try_new_cast(add_subtract_expr, result_type)
    }

    /// Create a new multiply expression
    pub fn try_new_multiply(lhs: DynProofExpr, rhs: DynProofExpr) -> AnalyzeResult<Self> {
        let lhs_datatype = lhs.data_type();
        let rhs_datatype = rhs.data_type();
        if try_binary_operation_type(lhs_datatype, rhs_datatype, &BinaryOperator::Multiply)
            .is_some()
        {
            Ok(Self::Multiply(MultiplyExpr::new(
                Box::new(lhs),
                Box::new(rhs),
            )))
        } else {
            Err(AnalyzeError::DataTypeMismatch {
                left_type: lhs_datatype.to_string(),
                right_type: rhs_datatype.to_string(),
            })
        }
    }

    /// Create a new cast expression
    pub fn try_new_cast(from_column: DynProofExpr, to_datatype: ColumnType) -> AnalyzeResult<Self> {
        let from_datatype = from_column.data_type();
        try_cast_types(from_datatype, to_datatype)
            .map(|()| Self::Cast(CastExpr::new(Box::new(from_column), to_datatype)))
            .map_err(|_| AnalyzeError::DataTypeMismatch {
                left_type: from_datatype.to_string(),
                right_type: to_datatype.to_string(),
            })
    }

    /// Create a new decimal scale cast expression
    pub fn try_new_decimal_scaling_cast(
        from_expr: DynProofExpr,
        to_datatype: ColumnType,
    ) -> AnalyzeResult<Self> {
        let from_datatype = from_expr.data_type();
        DecimalScalingCastExpr::try_new(Box::new(from_expr), to_datatype)
            .map(DynProofExpr::DecimalScalingCast)
            .map_err(|_| AnalyzeError::DataTypeMismatch {
                left_type: from_datatype.to_string(),
                right_type: to_datatype.to_string(),
            })
    }

    /// Check that the plan has the correct data type
    fn check_data_type(&self, data_type: ColumnType) -> AnalyzeResult<()> {
        if self.data_type() == data_type {
            Ok(())
        } else {
            Err(AnalyzeError::InvalidDataType {
                actual: self.data_type(),
                expected: data_type,
            })
        }
    }
}
