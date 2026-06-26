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
    #[test]
    #[cfg(not(feature = "rayon"))]
    fn if_rayon_selects_fallback_without_rayon_feature() {
        assert_eq!(super::if_rayon!("rayon", "fallback"), "fallback");
    }
}
