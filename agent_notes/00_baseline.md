# Baseline Info

## Environment
- **rustc**: 1.88.0 (6b00bc388 2025-06-23) (Homebrew)
- **cargo**: 1.88.0 (Homebrew)
- **OS**: macOS Darwin 22.6.0 x86_64

## Repository
- **Repo**: spaceandtimefdn/sxt-proof-of-sql
- **Branch**: fix/nullable-columns-183
- **Issue**: #183 - Add nullable column support

## Test Commands (from README)
- CPU-only (no GPU): `cargo test --no-default-features --features="arrow cpu-perf"`
- With Blitzar CPU backend: `export BLITZAR_BACKEND=cpu && cargo test --all-features --all-targets`
- Example run: `cargo run --example hello_world --no-default-features --features="rayon test"`

## Notes
- Project requires lld and clang on Linux
- GPU acceleration via NVIDIA recommended but CPU workaround available
- Must use `--no-default-features` for non-GPU machines
