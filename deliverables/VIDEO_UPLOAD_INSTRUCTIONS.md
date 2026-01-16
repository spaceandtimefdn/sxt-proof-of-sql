# Video Recording & Upload Instructions

## Recording Setup

**Duration:** 60-120 seconds
**Output:** `deliverables/demo_nullable_columns_183.mp4`

### macOS Recording (QuickTime)
1. Open QuickTime Player
2. File → New Screen Recording
3. Select recording area (terminal window)
4. Click Record

### Terminal Setup
```bash
cd /Users/nicholastoledo/b5
# Increase font size for readability (Cmd+Plus in most terminals)
```

---

## Recording Script (Execute These Commands)

### 1. Introduction (5 sec)
Show terminal prompt in the repo directory.

### 2. Run PoC Proof Test (25 sec)
```bash
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- test_nullable_column_to_committable --nocapture
```
Wait for: `test result: ok. 1 passed`

### 3. Run Issue #183 Requirement Test (25 sec)
```bash
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- test_nullable_plus_nonnullable_bigint_requirement --nocapture
```
Wait for: `test result: ok. 1 passed`

### 4. Run Full Test Suite (25 sec)
```bash
cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- nullable
```
Wait for: `test result: ok. 21 passed`

### 5. End (5 sec)
Show final output with all tests passing.

---

## Post-Recording

### Option A: Direct GitHub Upload (if small enough)
1. Go to PR #1120
2. Add a comment
3. Drag and drop the video file into the comment box
4. Submit comment

### Option B: External Hosting
If video is too large for GitHub:

1. Upload to Google Drive / Dropbox
2. Set sharing to "Anyone with link can view"
3. Post comment on PR:

```markdown
## Demo Video

Recorded demonstration of nullable column support:
- PoC proof test with nulls
- Issue #183 requirement test (nullable + non-nullable bigint)
- Full 21-test suite passing

**Video link:** [INSERT LINK HERE]

---
Nicholas Toledo / Toledo Technologies LLC
```

---

## PR Comment Template (After Upload)

```markdown
## Demo Video Attached

Demonstration of nullable column support for Issue #183:

1. **PoC Proof Test** - `test_nullable_column_to_committable` creates committable columns from nullable BigInt data
2. **Issue Requirement** - `test_nullable_plus_nonnullable_bigint_requirement` verifies nullable + non-nullable add
3. **Full Suite** - All 21 nullable tests pass

[Video attached above or linked]

---
Nicholas Toledo / Toledo Technologies LLC
```

---

## Quick Commands (Copy-Paste Ready)

```bash
# All commands for recording
cd /Users/nicholastoledo/b5

cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- test_nullable_column_to_committable --nocapture

cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- test_nullable_plus_nonnullable_bigint_requirement --nocapture

cargo test -p proof-of-sql --no-default-features --features="arrow cpu-perf test" -- nullable
```
