/// Tests for DynProofExpr covering uncovered branches
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::{
            commitment::InnerProductProof,
            database::{
                owned_table_utility::{bigint, boolean, owned_table, varchar},
                ColumnRef, ColumnType, OwnedTableTestAccessor, TestAccessor,
            },
            scalar::Curve25519Scalar,
        },
        sql::proof_exprs::{ColumnExpr, DynProofExpr, LiteralExpr, ProofExpr},
    };
    use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};

    fn make_accessor() -> OwnedTableTestAccessor<InnerProductProof> {
        let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
        accessor.add_table(
            "sxt.test".parse().unwrap(),
            owned_table([bigint("a", [1i64, 2, 3]), boolean("b", [true, false, true])]),
            0,
        );
        accessor
    }

    #[test]
    fn test_column_expr_data_type() {
        let accessor = make_accessor();
        let table_ref = "sxt.test".parse().unwrap();
        let col_ref = ColumnRef::new(table_ref, "a".parse().unwrap(), ColumnType::BigInt);
        let expr = DynProofExpr::try_new_column(col_ref).unwrap();
        assert_eq!(expr.data_type(), ColumnType::BigInt);
    }

    #[test]
    fn test_literal_expr_boolean() {
        let expr = DynProofExpr::new_literal(LiteralValue::Boolean(true));
        assert_eq!(expr.data_type(), ColumnType::Boolean);
    }

    #[test]
    fn test_literal_expr_bigint() {
        let expr = DynProofExpr::new_literal(LiteralValue::BigInt(42));
        assert_eq!(expr.data_type(), ColumnType::BigInt);
    }

    #[test]
    fn test_try_new_equals_column_types_must_match() {
        let accessor = make_accessor();
        let table_ref = "sxt.test".parse().unwrap();
        let col_ref_a = ColumnRef::new(table_ref, "a".parse().unwrap(), ColumnType::BigInt);
        let col_a = DynProofExpr::try_new_column(col_ref_a).unwrap();
        let lit_true = DynProofExpr::new_literal(LiteralValue::Boolean(true));
        // Comparing BigInt column with Boolean literal should fail
        let result = DynProofExpr::try_new_equals(col_a, lit_true);
        assert!(result.is_err());
    }
}
