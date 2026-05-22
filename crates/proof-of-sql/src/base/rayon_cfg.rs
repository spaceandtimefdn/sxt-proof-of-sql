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
    use super::if_rayon;

    #[cfg(feature = "rayon")]
    #[test]
    fn if_rayon_uses_rayon_branch_when_enabled() {
        let value = if_rayon!(17, fallback_value());

        assert_eq!(value, 17);
    }

    #[cfg(not(feature = "rayon"))]
    #[test]
    fn if_rayon_uses_fallback_branch_when_disabled() {
        let value = if_rayon!(rayon_value(), 23);

        assert_eq!(value, 23);
    }
}
