## DELIVERY

This update introduces caching for Dory setup objects (`PublicParameters`, `ProverSetup`, `VerifierSetup`) used in tests and CI, dramatically reducing test suite runtime. The cache is stored in `.dory_cache/` and is automatically populated on first run.

### How to run

1. Ensure Rust and Cargo are installed.
2. Run tests:

    make verify

or

    cargo test --all-features

The cache will be used for subsequent test runs, speeding up execution.

### Notes
- The cache is local and auto-managed; delete `.dory_cache/` to force regeneration.
- No tests are removed or weakened.
