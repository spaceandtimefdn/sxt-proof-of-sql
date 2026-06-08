/// Macro to simplify using cached test setups in Dory tests.
///
/// # Usage
///
/// ```ignore
/// use crate::proof_primitive::dory::test_macro::cached_dory_setup;
///
/// #[test]
/// fn my_test() {
///     cached_dory_setup!(nu = 4, prover_setup, verifier_setup);
///     // prover_setup and verifier_setup are now available as &'static references
///     // ... rest of test
/// }
/// ```
#[macro_export]
macro_rules! cached_dory_setup {
    (nu = $nu:expr, $prover:ident, $verifier:ident) => {
        let __setup = $crate::proof_primitive::dory::test_setup_accessor::get_test_setup($nu);
        let $prover = __setup.prover_setup;
        let $verifier = __setup.verifier_setup;
    };
    (nu = $nu:expr, $params:ident, $prover:ident, $verifier:ident) => {
        let __setup = $crate::proof_primitive::dory::test_setup_accessor::get_test_setup($nu);
        let $params = __setup.public_parameters;
        let $prover = __setup.prover_setup;
        let $verifier = __setup.verifier_setup;
    };
}

pub use cached_dory_setup;
