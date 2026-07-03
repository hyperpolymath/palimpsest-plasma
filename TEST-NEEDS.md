<!--
SPDX-License-Identifier: CC-BY-SA-4.0
Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->
# TEST-NEEDS.md — palimpsest-plasma

> Updated 2026-07-02 (v0.2.0). The 2026-03-29 audit ("zero unit tests")
> described the pre-engine repo and is superseded.

## Current State

| Category     | Count | Notes |
|--------------|-------|-------|
| Unit tests   | 61    | Inline `#[test]` across plasma-parser (42) and plasma-engine (19) |
| Integration  | 4     | `tests/check_selftest.rs` — the repo checks itself with its own bundled policy |
| E2E          | 1     | `check_self_passes` runs the built binary end-to-end |
| Determinism  | 2     | Byte-identical JSON asserted at unit and binary level |
| Benchmarks   | 0     | None yet |

Run everything with `cargo test --workspace`.

## What's Covered

- SPDX expression parsing (lexer, parser, catalog incl. Palimpsest-family ids)
- Zone assignment and `.plasma.toml` parsing
- Repository scanning (`scan_repo`): counts, score, deterministic ordering
- Policy schema loading: version gating, duplicate rule ids, reserved-construct
  rejection, TOML↔JSON round-trip
- Evaluation semantics: deontic matrix rows, condition gating, subject
  expansion, overlay provenance, permission never violating
- SARIF shape: rule namespace, levels, location omission for repo subjects
- Fact collection: skip list, header extraction, Cargo version metadata

## What's Missing

### Property-Based Tests
- [ ] Arbitrary policy documents: load → serialize → reload identity
- [ ] Arbitrary fact sets: evaluation totality (no panics) under fuzzing
- [ ] SPDX parser: arbitrary expression round-trips

### E2E Tests
- [ ] `plasma init`/`migrate` against fixture repos (tempdir round-trips)
- [ ] `plasma audit --fix` idempotence (fixing twice changes nothing)
- [ ] `plasma check --policy <json>` (JSON policy path is unit-tested only)

### Aspect Tests
- **Security:** path traversal attempts in audit/fact walking; SVG/badge
  output injection
- **Performance:** scan/evaluate time vs repository size benchmarks
- **Error handling:** malformed policy files produce the intended
  `SchemaError` variants (partially covered)

## FAKE-FUZZ ALERT (still open)

- `tests/fuzz/placeholder.txt` is a scorecard placeholder inherited from
  rsr-template-repo — it does NOT provide real fuzz testing
- Replace with an actual fuzz harness (`cargo-fuzz` over `load_policy_str`
  and `parse_spdx_expr` are the natural first targets) or remove the file
- Priority: P2 — creates false impression of fuzz coverage
