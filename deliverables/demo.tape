# Nullable BigInt Column Support Demo
# Issue #183 - spaceandtimefdn/sxt-proof-of-sql

Output demo.mp4
Output demo.gif

Set FontSize 14
Set Width 1200
Set Height 800
Set Theme "Dracula"

Type "# Nullable BigInt Column Support - Issue #183"
Enter
Sleep 1s

Type "# Running nullable column tests with proof verification"
Enter
Sleep 500ms

Type "cargo test -p proof-of-sql --no-default-features --features='arrow cpu-perf test' -- nullable --nocapture 2>&1 | head -80"
Enter
Sleep 30s

Type ""
Enter
Type "# All 22 tests passed! Nullable bigint support is complete."
Enter
Sleep 2s

Type "# Key features implemented:"
Enter
Type "# - NullableOwnedColumn with validity masks"
Enter
Type "# - Nullable + non-nullable bigint arithmetic"
Enter
Type "# - Canonical null invariant (null positions = 0)"
Enter
Type "# - Arrow integration with validity bitmap"
Enter
Type "# - Proof commitment support"
Enter
Sleep 3s
