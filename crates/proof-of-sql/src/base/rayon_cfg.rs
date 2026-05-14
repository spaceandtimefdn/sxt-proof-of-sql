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
    fn if_rayon_selects_the_active_branch() {
        let value = super::if_rayon!(1, 2);

        #[cfg(feature = "rayon")]
        assert_eq!(value, 1);

        #[cfg(not(feature = "rayon"))]
        assert_eq!(value, 2);
    }
}
