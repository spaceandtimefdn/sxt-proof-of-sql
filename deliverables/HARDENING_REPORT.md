# Hardening Report: Nullable Column Support PR #1120

**Date:** 2026-01-16
**Author:** Nicholas Toledo / Toledo Technologies LLC

---

## PR Information

| Field | Value |
|-------|-------|
| **PR Number** | #1120 |
| **PR URL** | https://github.com/spaceandtimefdn/sxt-proof-of-sql/pull/1120 |
| **Issue** | #183 |
| **Base Branch** | `main` ✅ |
| **State** | OPEN |
| **Mergeable** | MERGEABLE ✅ |
| **Review Decision** | REVIEW_REQUIRED |

---

## PR Body Verification

| Requirement | Status |
|-------------|--------|
| Contains "Addresses #183" | ✅ Yes |
| PoC section present | ✅ Yes |
| How to Test section | ✅ Yes |
| Acceptance Criteria Checklist | ✅ Yes |
| Author attribution | ✅ Yes |

---

## CI Status

| Check | Status |
|-------|--------|
| Orca Security - Infrastructure as Code | ✅ Passed |
| Orca Security - Secrets | ✅ Passed |
| Orca Security - Vulnerabilities | ✅ Passed |
| **Overall** | ✅ All checks passed |

---

## Local Verification

| Command | Result |
|---------|--------|
| `cargo fmt --all --check` | ✅ Pass |
| `cargo test ... -- nullable` | ✅ 21 tests pass |
| `cargo test ... -- validity` | ✅ 8 tests pass |
| `cargo clippy ...` | ✅ No errors (warnings only) |

---

## Bidi/Hidden Unicode Scan

| Scan Type | Result |
|-----------|--------|
| Ripgrep bidi characters | ✅ None found |
| Python Cf category scan | ✅ None found |
| **GitHub warning risk** | ✅ Eliminated |

See `deliverables/BIDI_SCAN_RESULTS.md` for full details.

---

## PoC Proof Test

**Test that proves commitment compatibility with nulls:**
`test_nullable_column_to_committable` in `crates/proof-of-sql/src/base/database/nullable_column_proof_test.rs`

**What it does:**
1. Creates nullable BigInt column with values `[10, 20, 30, 40, 50]` and validity `[true, false, true, false, true]`
2. Canonicalizes null positions to 0: `[10, 0, 30, 0, 50]`
3. Creates `CommittableColumn::BigInt` for data
4. Creates `CommittableColumn::Boolean` for validity mask
5. Verifies both are commitment-ready

**Run command:**
```bash
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- test_nullable_column_to_committable --nocapture
```

---

## Issue #183 Requirement Test

**Test for nullable + non-nullable bigint:**
`test_nullable_plus_nonnullable_bigint_requirement`

**Run command:**
```bash
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- test_nullable_plus_nonnullable_bigint_requirement --nocapture
```

---

## Video Status

| Item | Status |
|------|--------|
| Video file exists | ❌ Not yet recorded |
| Recording instructions | ✅ `deliverables/VIDEO_UPLOAD_INSTRUCTIONS.md` |
| Upload method | GitHub PR comment or external link |

---

## Deliverables Created

| File | Purpose |
|------|---------|
| `HARDENING_REPORT.md` | This report |
| `PR_BODY_FINAL.md` | Final PR body content |
| `PR_COMMENT_FINAL.md` | PR comment for reviewers |
| `ISSUE_COMMENT_FINAL.md` | Issue comment for reviewers |
| `BIDI_SCAN_RESULTS.md` | Hidden unicode scan results |
| `VIDEO_UPLOAD_INSTRUCTIONS.md` | Video recording/upload guide |
| `POST_MERGE_CLAIM.md` | Claim comment (post-merge only) |
| `MAINTAINER_RESPONSE_TEMPLATES.md` | Response templates |

---

## Commands to Reproduce

```bash
# Clone and checkout
git clone https://github.com/ntoledo319/sxt-proof-of-sql.git
cd sxt-proof-of-sql
git checkout fix/nullable-columns-183

# Run PoC test
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- test_nullable_column_to_committable --nocapture

# Run all nullable tests
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- nullable

# Verify formatting
cargo fmt --all --check

# Run clippy
cargo clippy -p proof-of-sql --no-default-features --features="arrow cpu-perf test"
```

---

## Post-Merge Claim

**DO NOT POST UNTIL PR IS MERGED**

See `deliverables/POST_MERGE_CLAIM.md` for exact claim comment.

Summary:
```
/claim #183

Implemented nullable column support for Proof of SQL as specified in this issue.
[Full details in POST_MERGE_CLAIM.md]

Nicholas Toledo / Toledo Technologies LLC
```

---

## Summary

| Item | Status |
|------|--------|
| PR targets main | ✅ |
| PR references #183 | ✅ |
| CI checks green | ✅ |
| Bidi warning eliminated | ✅ |
| PoC test exists | ✅ |
| Issue requirement test exists | ✅ |
| Local tests pass | ✅ |
| Claim ready (post-merge) | ✅ |
| Video instructions ready | ✅ |
| Maintainer responses ready | ✅ |

**Status: MERGE-READY pending maintainer review**
