# Demo Video Script: Nullable Column Support

## Overview
- **Duration:** 60-120 seconds
- **Purpose:** Demonstrate nullable column support for Proof of SQL (Issue #183)
- **Output:** `deliverables/demo_nullable_columns_183.mp4`

---

## Recording Instructions

### Setup
1. Open terminal in `/Users/nicholastoledo/CascadeProjects/b4/sxt-proof-of-sql`
2. Use large font (14pt+) for readability
3. Use a screen recorder (QuickTime, OBS, etc.)
4. Record at 1920x1080 or higher

---

## Script (60-120 seconds)

### Scene 1: Introduction (10 sec)
**Show:** Terminal with repo directory
**Say:** "Demonstrating nullable column support for Proof of SQL, Issue #183."

### Scene 2: Run PoC Proof Test (25 sec)
**Type:**
```bash
cargo test -p proof-of-sql --no-default-features --features="std,arrow" -- nullable_column_proof_test --nocapture
```
**Wait for:** Test passes
**Say:** "These PoC tests cover nullable BigInt creation, validity masks, and conversions."

### Scene 3: Run Issue Requirement Test (25 sec)
**Type:**
```bash
cargo test -p proof-of-sql --no-default-features --features="std,arrow" -- test_nullable_plus_nonnullable_bigint_requirement --nocapture
```
**Wait for:** Test passes
**Say:** "This test verifies the explicit Issue 183 requirement: adding a nullable bigint to a non-nullable bigint."

### Scene 4: (Optional, Linux/CI) Run end-to-end proof (20 sec)
**Type:**
```bash
cargo test -p proof-of-sql --features="perf,arrow" test_nullable_bigint_filter_proves_with_validity --nocapture
```
**Say:** "On Linux with blitzar, this runs the full proof generation + verification for nullable BigInt."

### Scene 5: Run Full Nullable Suite (20 sec)
**Type:**
```bash
cargo test -p proof-of-sql --no-default-features --features="std,arrow" -- nullable
```
**Wait for:** Summary line
**Say:** "All nullable column tests pass, covering validity masks, null propagation, Arrow conversion, and the new NullableBigInt type."

### Scene 6: Conclusion (10 sec)
**Show:** Terminal with successful output
**Say:** "PR 1120 is ready for review. Nicholas Toledo, Toledo Technologies."

---

## Quick Record Commands (copy-paste ready)

```bash
# Change to repo directory
cd /Users/nicholastoledo/CascadeProjects/b4/sxt-proof-of-sql

# Test 1: PoC nullable suite (std + arrow)
cargo test -p proof-of-sql --no-default-features --features="std,arrow" -- nullable_column_proof_test --nocapture

# Test 2: Issue #183 requirement (nullable + non-nullable)
cargo test -p proof-of-sql --no-default-features --features="std,arrow" -- test_nullable_plus_nonnullable_bigint_requirement --nocapture

# Test 3: Full nullable test suite
cargo test -p proof-of-sql --no-default-features --features="std,arrow" -- nullable

# Optional (Linux/CI): end-to-end proof with blitzar
cargo test -p proof-of-sql --features="perf,arrow" test_nullable_bigint_filter_proves_with_validity --nocapture
```

---

## Post-Recording

1. Save as: `deliverables/demo_nullable_columns_183.mp4`
2. Verify video shows all tests passing
3. Upload to PR or provide download link
