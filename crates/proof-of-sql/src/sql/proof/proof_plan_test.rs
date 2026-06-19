//! Tests for the [`HonestProver`] unit type and its associated marker trait
//! [`ProverHonestyMarker`] declared in [`super::proof_plan`].
//!
//! These tests cover the previously-untested executable lines of
//! `sql/proof/proof_plan.rs`:
//! - `HonestProver` derives `Debug`, `PartialEq`, `Clone`
//! - `HonestProver` impls `ProverHonestyMarker`
//! - The trait bounds `Debug + Send + Sync + PartialEq + 'static` hold

use super::{HonestProver, ProverHonestyMarker};

#[test]
fn we_can_construct_an_honest_prover() {
    // HonestProver is a unit struct; construction is just `HonestProver`.
    let _prover = HonestProver;
}

#[test]
fn we_can_clone_an_honest_prover() {
    let prover = HonestProver;
    let cloned = prover.clone();
    // Unit type: every HonestProver compares equal.
    assert_eq!(prover, cloned);
}

#[test]
fn honest_provers_compare_equal() {
    let a = HonestProver;
    let b = HonestProver;
    // PartialEq + Eq reflexivity + symmetry.
    assert_eq!(a, b);
    assert_eq!(b, a);
    assert_eq!(a, a);
}

#[test]
fn honest_prover_debug_contains_its_name() {
    let prover = HonestProver;
    let rendered = format!("{prover:?}");
    // The derived Debug impl should mention the type name.
    assert!(
        rendered.contains("HonestProver"),
        "Debug output was {rendered:?}, expected to contain `HonestProver`"
    );
}

#[test]
fn honest_prover_implements_prover_honesty_marker() {
    // Compile-time assertion: HonestProver satisfies the marker trait.
    fn accept_marker<T: ProverHonestyMarker>(t: &T) -> &T {
        t
    }
    let prover = HonestProver;
    let _ref: &HonestProver = accept_marker(&prover);
}

#[test]
fn honest_prover_satisfies_marker_trait_bounds() {
    // The marker requires Debug + Send + Sync + PartialEq + 'static.
    // The following compile-time asserts verify each bound independently.
    fn assert_debug<T: Debug>(_: &T) {}
    fn assert_send<T: Send>(_: &T) {}
    fn assert_sync<T: Sync>(_: &T) {}
    fn assert_partial_eq<T: PartialEq>(_: &T) {}
    fn assert_static<T: 'static>(_: &T) {}

    let prover = HonestProver;
    assert_debug(&prover);
    assert_send(&prover);
    assert_sync(&prover);
    assert_partial_eq(&prover);
    assert_static(&prover);
}

#[test]
fn honest_prover_works_in_a_generic_prover_honesty_marker_context() {
    // Exercise the trait dispatch by passing through a generic function
    // that requires `ProverHonestyMarker`. This exercises the empty impl
    // `impl ProverHonestyMarker for HonestProver {}`.
    fn roundtrip<T: ProverHonestyMarker + Clone>(t: T) -> T {
        t.clone()
    }
    let prover = HonestProver;
    let returned = roundtrip(prover.clone());
    assert_eq!(returned, prover);
}
