//! This module proves provable expressions.
mod proof_expr;
pub(crate) use proof_expr::DecimalProofExpr;
pub use proof_expr::ProofExpr;
#[cfg(all(test, feature = "blitzar"))]
mod proof_expr_test;

mod aliased_dyn_proof_expr;
pub use aliased_dyn_proof_expr::AliasedDynProofExpr;

mod add_expr;
pub(crate) use add_expr::AddExpr;
mod subtract_expr;
pub(crate) use subtract_expr::SubtractExpr;
#[cfg(all(test, feature = "blitzar"))]
mod add_subtract_expr_test;

mod multiply_expr;
use multiply_expr::MultiplyExpr;
#[cfg(all(test, feature = "blitzar"))]
mod multiply_expr_test;

mod dyn_proof_expr;
pub use dyn_proof_expr::DynProofExpr;

mod literal_expr;
pub(crate) use literal_expr::LiteralExpr;
#[cfg(all(test, feature = "blitzar"))]
mod literal_expr_test;

mod placeholder_expr;
pub(crate) use placeholder_expr::PlaceholderExpr;
#[cfg(all(test, feature = "blitzar"))]
mod placeholder_expr_test;

mod and_expr;
pub(crate) use and_expr::AndExpr;
#[cfg(all(test, feature = "blitzar"))]
mod and_expr_test;

mod inequality_expr;
use inequality_expr::InequalityExpr;
#[cfg(all(test, feature = "blitzar"))]
mod inequality_expr_test;

mod or_expr;
use or_expr::OrExpr;
#[cfg(all(test, feature = "blitzar"))]
mod or_expr_test;

mod not_expr;
use not_expr::NotExpr;
#[cfg(all(test, feature = "blitzar"))]
mod not_expr_test;

mod numerical_util;
pub(crate) use numerical_util::{add_subtract_columns, multiply_columns};
#[cfg(test)]
pub(crate) use numerical_util::{divide_columns, modulo_columns};

mod equals_expr;
pub(crate) use equals_expr::EqualsExpr;
#[cfg(all(test, feature = "blitzar"))]
mod equals_expr_test;

mod table_expr;
pub use table_expr::TableExpr;

#[cfg(test)]
pub(crate) mod test_utility;

mod column_expr;
pub use column_expr::ColumnExpr;
#[cfg(all(test, feature = "blitzar"))]
mod column_expr_test;

mod cast_expr;
pub(crate) use cast_expr::CastExpr;
#[cfg(all(test, feature = "blitzar"))]
mod cast_expr_test;

mod scaling_cast_expr;
pub(crate) use scaling_cast_expr::ScalingCastExpr;
#[cfg(all(test, feature = "blitzar"))]
mod scaling_cast_expr_test;
