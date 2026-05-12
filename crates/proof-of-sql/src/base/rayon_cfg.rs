macro_rules! if_rayon {
    ($rayon_value: expr, $else_value: expr) => {{
        #[cfg(feature = "rayon")]
        {
            ($rayon_value)
        }
        #[cfg(not(feature = "rayon"))]
        {
            ($else_value)
        }
    }};
}
pub(crate) use if_rayon;

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "rayon"))]
    #[test]
    fn if_rayon_selects_fallback_value_without_rayon_feature() {
        assert_eq!(if_rayon!("rayon", "fallback"), "fallback");
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn if_rayon_selects_rayon_value_with_rayon_feature() {
        assert_eq!(if_rayon!("rayon", "fallback"), "rayon");
    }
}
