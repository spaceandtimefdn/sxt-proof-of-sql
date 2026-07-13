//! Tests for fold_util functions.

#[cfg(test)]
mod fold_util_test {
    use crate::sql::proof_plans::fold_util::{fold_columns, fold_vals};
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn test_fold_vals_empty() {
        let vals: [TestScalar; 0] = [];
        let result = fold_vals(TestScalar::TWO, &vals);
        assert_eq!(result, TestScalar::ZERO);
    }

    #[test]
    fn test_fold_vals_single() {
        let vals = [TestScalar::from(5u64)];
        let result = fold_vals(TestScalar::TWO, &vals);
        assert_eq!(result, TestScalar::from(5u64));
    }

    #[test]
    fn test_fold_vals_multiple() {
        // With beta=2, fold 1, 2, 3 = 1*2^2 + 2*2 + 3 = 4 + 4 + 3 = 11
        let vals = [TestScalar::ONE, TestScalar::TWO, TestScalar::from(3u64)];
        let result = fold_vals(TestScalar::TWO, &vals);
        // 1*4 + 2*2 + 3 = 4 + 4 + 3 = 11
        assert_eq!(result, TestScalar::from(11u64));
    }

    #[test]
    fn test_fold_vals_with_zero_beta() {
        // With beta=0, fold a, b = a*0^2 + b*0 = 0
        let vals = [TestScalar::ONE, TestScalar::TWO];
        let result = fold_vals(TestScalar::ZERO, &vals);
        assert_eq!(result, TestScalar::ZERO);
    }

    #[test]
    fn test_fold_columns_basic() {
        use crate::base::database::Column;
        use crate::base::polynomial::MultilinearExtension;
        
        let mut res = vec![TestScalar::ZERO; 3];
        let col: Column<TestScalar> = Column::Int128(vec![1i128, 2, 3].into());
        fold_columns(&mut res, TestScalar::ONE, TestScalar::TWO, &[col]);
        // The first column contributes: 1*1, 2*1, 3*1 = 1, 2, 3
        assert_eq!(res[0], TestScalar::ONE);
        assert_eq!(res[1], TestScalar::TWO);
        assert_eq!(res[2], TestScalar::from(3u64));
    }
}
