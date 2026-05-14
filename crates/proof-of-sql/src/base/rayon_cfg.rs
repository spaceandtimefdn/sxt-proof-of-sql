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

    #[cfg(not(feature = "rayon"))]
    #[test]
    fn if_rayon_uses_else_value_without_rayon_feature() {
        assert_eq!(if_rayon!(panic!("rayon branch should not run"), 7), 7);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn if_rayon_uses_rayon_value_with_rayon_feature() {
        assert_eq!(if_rayon!(11, panic!("else branch should not run")), 11);
    }
}
