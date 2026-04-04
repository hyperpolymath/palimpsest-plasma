# TEST-NEEDS.md — palimpsest-plasma
## CRG Grade: C — ACHIEVED 2026-04-04

> Generated 2026-03-29 by punishing audit.

## Current State

| Category     | Count | Notes |
|-------------|-------|-------|
| Unit tests   | 0     | No inline `#[test]` in any Rust source |
| Integration  | 1     | union-policy-parser/ffi/zig/test/integration_test.zig |
| E2E          | 0     | None |
| Benchmarks   | 0     | None |

**Source modules:** ~35 Rust files across: main.rs, init.rs, audit.rs, badge.rs, migrate.rs + union-policy-parser (Idris2 ABI + Zig FFI). Also: 3 Idris2 ABI, 2 Elixir, 2 Julia scripts.

## What's Missing

### P2P (Property-Based) Tests
- [ ] Badge generation: property tests for SVG/image output validity
- [ ] Audit: property tests for license detection accuracy
- [ ] Migration: property tests for license text transformation correctness
- [ ] Union policy parser: arbitrary policy document fuzzing

### E2E Tests
- [ ] Full audit: scan project -> detect licenses -> generate report
- [ ] Full migration: detect old license -> generate new -> apply -> verify
- [ ] Badge: generate badge for all license types
- [ ] Init: new project initialization with license setup

### Aspect Tests
- **Security:** No tests for license text injection, badge SVG injection, path traversal in audit scanning
- **Performance:** No scanning speed benchmarks
- **Concurrency:** No tests for parallel file scanning
- **Error handling:** No tests for malformed license files, unsupported formats, corrupted badge templates

### Build & Execution
- [ ] `cargo test`
- [ ] Zig FFI test execution
- [ ] CLI smoke tests

### Benchmarks Needed
- [ ] License detection speed per file
- [ ] Full repository audit time vs repo size
- [ ] Badge generation time
- [ ] Migration transformation throughput

### Self-Tests
- [ ] Audit its own license compliance
- [ ] Badge generation for its own license
- [ ] ABI version agreement

## Priority

**CRITICAL.** 35 Rust source files with ZERO unit tests. A license compliance tool that has never been unit-tested. The single Zig FFI integration test only covers the union-policy-parser seam. The 5 main Rust modules (main, init, audit, badge, migrate) are completely untested.

## FAKE-FUZZ ALERT

- `tests/fuzz/placeholder.txt` is a scorecard placeholder inherited from rsr-template-repo — it does NOT provide real fuzz testing
- Replace with an actual fuzz harness (see rsr-template-repo/tests/fuzz/README.adoc) or remove the file
- Priority: P2 — creates false impression of fuzz coverage
