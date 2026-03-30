# PROOF-NEEDS.md — palimpsest-plasma

## Current State

- **src/abi/*.idr**: NO
- **Dangerous patterns**: 18 `unwrap()` in Rust code
- **LOC**: ~6,900 (Rust)
- **ABI layer**: Missing

## What Needs Proving

| Component | What | Why |
|-----------|------|-----|
| SPDX parser correctness | Parser accepts all valid SPDX expressions and rejects invalid ones | Wrong SPDX parsing causes license misidentification |
| License family registry | Family classification is complete and consistent | Misclassified licenses create compliance risk |
| Union policy parser | Policy composition preserves individual license requirements | Composed policies must not lose restrictions |
| Content audit | Audit detection is sound (no false negatives) | Missed license violations create legal exposure |

## Recommended Prover

**Idris2** — Create `src/abi/` with SPDX expression grammar proofs (completeness and soundness). License family classification could use algebraic data types with exhaustive matching.

## Priority

**LOW** — License parsing and auditing tool. The 18 unwrap() calls should be addressed, but the overall risk is limited to incorrect license reports.
