# Demo Video Script: Nullable Column Support

## Overview
- **Duration:** 60-120 seconds
- **Purpose:** Demonstrate nullable column support for Proof of SQL (Issue #183)
- **Output:** `deliverables/demo_nullable_columns_183.mp4`

---

## Recording Instructions

### Setup
1. Open terminal in `/Users/nicholastoledo/b5`
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
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- test_nullable_column_to_committable --nocapture
```
**Wait for:** Test passes
**Say:** "This PoC test creates a nullable BigInt column and commits both data and validity mask."

### Scene 3: Run Issue Requirement Test (25 sec)
**Type:**
```bash
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- test_nullable_plus_nonnullable_bigint_requirement --nocapture
```
**Wait for:** Test passes
**Say:** "This test verifies the explicit Issue 183 requirement: adding a nullable bigint to a non-nullable bigint."

### Scene 4: Run Full Test Suite (20 sec)
**Type:**
```bash
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- nullable
```
**Wait for:** "21 passed" output
**Say:** "All 21 nullable column tests pass, covering validity masks, null propagation, and Arrow conversion."

### Scene 5: Conclusion (10 sec)
**Show:** Terminal with successful output
**Say:** "PR 1120 is ready for review. Nicholas Toledo, Toledo Technologies."

---

## Quick Record Commands (copy-paste ready)

```bash
# Change to repo directory
cd /Users/nicholastoledo/b5

# Test 1: PoC proof test with nulls
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- test_nullable_column_to_committable --nocapture

# Test 2: Issue #183 requirement (nullable + non-nullable)
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- test_nullable_plus_nonnullable_bigint_requirement --nocapture

# Test 3: Full nullable test suite
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- nullable
```

---

## Post-Recording

1. Save as: `deliverables/demo_nullable_columns_183.mp4`
2. Verify video shows all tests passing
3. Upload to PR or provide download link
