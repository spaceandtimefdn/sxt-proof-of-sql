# Maintainer Response Templates

Ready-to-use responses for common reviewer questions.

---

## Template 1: "Can you extend this to more types?"

**Question:** Can this be extended to support other types like Int, SmallInt, Decimal75, VarChar?

**Response:**

Yes, the design explicitly supports this. The `NullableOwnedColumn<S>` wrapper is type-agnostic—it wraps any `OwnedColumn` variant with a validity mask. The `validity` module's `canonicalize_nulls_numeric()` already handles all numeric types via the `Default + Copy` trait bounds.

Extension path:
1. Add operation functions (e.g., `add_nullable_int()`) following the `add_nullable_bigint()` pattern
2. Extend Arrow conversion to handle other array types
3. Add tests for each type

This PoC establishes the pattern; full type coverage is straightforward follow-up work.

---
Nicholas Toledo / Toledo Technologies LLC

---

## Template 2: "How does this handle SQL three-valued logic?"

**Question:** Does this implement proper SQL three-valued logic for comparisons?

**Response:**

The current PoC focuses on arithmetic null propagation (NULL + X = NULL), which follows SQL semantics. For comparisons, SQL three-valued logic (TRUE/FALSE/UNKNOWN) would require:

1. Returning a `NullableOwnedColumn<Boolean>` for comparison results
2. UNKNOWN maps to validity[i] = false with value = false (canonical)
3. WHERE clause filtering treats UNKNOWN as FALSE

The validity mask pattern supports this—UNKNOWN is represented as validity=false. The PoC establishes the foundation; comparison operators can follow the same pattern.

---
Nicholas Toledo / Toledo Technologies LLC

---

## Template 3: "Why canonical null values? Proof soundness explanation."

**Question:** Why do null positions need canonical values (0)? Can't we just ignore them?

**Response:**

The canonical null invariant is critical for **proof soundness**. Without it:

1. A malicious prover could hide arbitrary values under NULL entries
2. These hidden values would be committed but never constrained
3. Verifier wouldn't detect the discrepancy since "NULL" masks the value

By enforcing canonical values (0 for numeric):
- Committed data is deterministic for any validity mask
- Verifier can recompute expected commitments
- No information can be hidden in NULL positions

This is why `new_with_canonical_nulls()` and all operations enforce this invariant.

---
Nicholas Toledo / Toledo Technologies LLC

---

## Template 4: "Performance: Vec<bool> vs bitmap?"

**Question:** Why use `Vec<bool>` instead of a packed bitmap? Isn't that inefficient?

**Response:**

`Vec<bool>` was chosen for PoC clarity and compatibility with existing code. Trade-offs:

| Approach | Pros | Cons |
|----------|------|------|
| `Vec<bool>` | Simple, matches existing patterns, easy debugging | 8x memory overhead |
| Packed bitmap | Memory efficient, matches Arrow format | More complex bit manipulation |

For production, a bitmap representation (possibly reusing Arrow's `BooleanBuffer`) would be better. The current implementation can be migrated without changing the API—`validity()` returns `Option<&[bool]>` which could be backed by either representation.

---
Nicholas Toledo / Toledo Technologies LLC

---

## Template 5: "What about NOT NULL constraints?"

**Question:** How do we enforce NOT NULL column constraints?

**Response:**

NOT NULL enforcement happens at schema/query planning level, not in the column storage. This implementation supports both:

1. **Nullable columns**: `validity = Some(mask)`
2. **Non-nullable columns**: `validity = None` (all values valid by definition)

Schema enforcement would:
1. Track nullability in `ColumnType` or table metadata
2. Reject INSERT/UPDATE of NULL into NOT NULL columns at query execution
3. Optimizer can skip null checks when column is known NOT NULL

---
Nicholas Toledo / Toledo Technologies LLC

---

## Template 6: "Tests don't show actual proof generation/verification"

**Question:** The tests only show commitment creation, not actual prove→verify.

**Response:**

You're correct that the current PoC tests demonstrate commitment compatibility rather than full QueryProof generation. This is intentional for the PoC scope:

1. `test_nullable_column_to_committable` proves nullable data can be committed
2. The canonical null invariant ensures commitment soundness

Full prove→verify integration requires:
1. Modifying `ProofExpr` evaluators to handle nullable columns
2. Adding validity constraints to the proof system
3. This is substantial work beyond PoC scope

The PoC establishes that nullable columns are **commitment-compatible** and **sound**, which is the foundation for full integration.

---
Nicholas Toledo / Toledo Technologies LLC
