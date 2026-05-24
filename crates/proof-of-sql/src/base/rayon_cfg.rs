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

    #[test]
    fn if_rayon_selects_else_value_when_rayon_is_disabled() {
        let selected = if_rayon!(1usize, 2usize);
        assert_eq!(selected, 2);
    }

    #[test]
    fn if_rayon_keeps_selected_branch_side_effects_local() {
        let (selected, from_else_branch) = if_rayon!(
            {
                (10usize, false)
            },
            {
                (20usize, true)
            }
        );

        assert_eq!(selected, 20);
        assert!(from_else_branch);
    }
}
