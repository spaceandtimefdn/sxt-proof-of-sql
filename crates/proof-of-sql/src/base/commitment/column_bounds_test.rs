//! Tests for Bounds and BoundsInner types.

#[cfg(test)]
mod bounds_tests {
    use crate::base::commitment::{Bounds, BoundsInner, NegativeBounds};
    use crate::base::scalar::Scalar;
    use alloc::string::ToString;

    #[test]
    fn test_negative_bounds_display() {
        let err = NegativeBounds;
        assert_eq!(
            format!("{}", err),
            "cannot construct bounds where min is greater than max"
        );
    }

    #[test]
    fn test_negative_bounds_debug() {
        let err = NegativeBounds;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("NegativeBounds"));
    }

    #[test]
    fn test_negative_bounds_unit_struct() {
        let err1 = NegativeBounds;
        let err2 = NegativeBounds;
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_bounds_inner_try_new_valid() {
        let result = BoundsInner::<i32>::try_new(1, 10);
        assert!(result.is_ok());
        let inner = result.unwrap();
        assert_eq!(*inner.min(), 1);
        assert_eq!(*inner.max(), 10);
    }

    #[test]
    fn test_bounds_inner_try_new_invalid() {
        let result = BoundsInner::<i32>::try_new(10, 1);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NegativeBounds));
    }

    #[test]
    fn test_bounds_inner_union() {
        let inner1 = BoundsInner::try_new(1, 5).unwrap();
        let inner2 = BoundsInner::try_new(3, 8).unwrap();
        let union = inner1.union(inner2);
        assert_eq!(*union.min(), 1);
        assert_eq!(*union.max(), 8);
    }

    #[test]
    fn test_bounds_inner_surrounds() {
        let inner = BoundsInner::try_new(1, 10).unwrap();
        assert!(inner.surrounds(&5));
        assert!(inner.surrounds(&1));
        assert!(inner.surrounds(&10));
        assert!(!inner.surrounds(&0));
        assert!(!inner.surrounds(&11));
    }

    #[test]
    fn test_bounds_sharp() {
        let result = Bounds::<i32>::sharp(1, 100);
        assert!(result.is_ok());
    }

    #[test]
    fn test_bounds_bounded() {
        let result = Bounds::<i32>::bounded(0, 50);
        assert!(result.is_ok());
    }

    #[test]
    fn test_bounds_empty() {
        let bounds = Bounds::<i32>::Empty;
        assert!(matches!(bounds, Bounds::Empty));
    }

    #[test]
    fn test_bounds_clone() {
        let bounds = Bounds::sharp(1, 10).unwrap();
        let cloned = bounds;
        assert_eq!(bounds, cloned);
    }

    #[test]
    fn test_bounds_default() {
        let default_bounds: Bounds<i32> = Bounds::default();
        assert!(matches!(default_bounds, Bounds::Empty));
    }

    #[test]
    fn test_bounds_debug() {
        let bounds = Bounds::sharp(1, 10).unwrap();
        let debug_str = format!("{:?}", bounds);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_bounds_inner_debug() {
        let inner = BoundsInner::try_new(1, 10).unwrap();
        let debug_str = format!("{:?}", inner);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_bounds_inner_partial_eq() {
        let inner1 = BoundsInner::try_new(1, 10).unwrap();
        let inner2 = BoundsInner::try_new(1, 10).unwrap();
        let inner3 = BoundsInner::try_new(2, 10).unwrap();
        assert_eq!(inner1, inner2);
        assert_ne!(inner1, inner3);
    }

    #[test]
    fn test_bounds_sharp_bounded() {
        let sharp = Bounds::sharp(1, 10).unwrap();
        let bounded = Bounds::bounded(1, 10).unwrap();
        assert!(matches!(sharp, Bounds::Sharp(_)));
        assert!(matches!(bounded, Bounds::Bounded(_)));
    }

    #[test]
    fn test_bounds_with_scalar() {
        let scalar_min = Scalar::from(0i64);
        let scalar_max = Scalar::from(100i64);
        let result = Bounds::<Scalar>::sharp(scalar_min, scalar_max);
        assert!(result.is_ok());
    }
}