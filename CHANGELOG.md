<!--
SPDX-License-Identifier: MPL-2.0
Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->
# Changelog

All notable changes to `palimpsest-plasma` will be documented in this file.

This file is generated from conventional commits by the
[`changelog-reusable.yml`](https://github.com/hyperpolymath/standards/blob/main/.github/workflows/changelog-reusable.yml)
workflow (`hyperpolymath/standards#206`). Adopt the workflow in this repo's CI to keep this file in sync automatically — see
[`templates/cliff.toml`](https://github.com/hyperpolymath/standards/blob/main/templates/cliff.toml)
for the canonical config.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
this project aims to follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2026-07-16

### Added

- feat(engine): `diff(before, after) -> FactsDiff` — pure, deterministic
  comparison of two fact snapshots: files added/removed, SPDX header
  transitions, metadata changes, git state (`plasma-engine/src/diff.rs`)
- feat(cli): `plasma diff before.json after.json` with diff(1) exit
  semantics (0 identical, 1 differ, 2 error) — the before/after
  verification protocol for agent claims
- docs: `docs/interchange-contracts.adoc` — pins the four wire contracts
  (facts snapshot, facts diff, findings JSON, SARIF namespace) that
  sibling tools build against; additive-only evolution within a major

## [0.3.0] - 2026-07-16

### Added

- feat(engine): action planner — pure `plan(policy, evaluation, ctx)` maps
  violations to corrective actions or a manual list with reasons; IO
  boundary `apply(plan, root, opts)` executes them with `.bak` backups and
  idempotent skips (`plasma-engine/src/action.rs`, `apply.rs`)
- feat(cli): `plasma fix` — dry-run by default, `--apply` to make changes,
  `--license`/`--author`/`--no-backup`/`--format human|json`. First
  auto-action: AddSpdxHeader; missing files / unparsable headers → manual
- docs: "Action planner" section in `docs/engine-v0-design.adoc`;
  `plasma fix` documented in `docs/cli-design.adoc`
- test: `tests/fix_selftest.rs` (dry-run, apply+backup, idempotency, json)

## [0.2.0] - 2026-07-02

Identity pivot: palimpsest-plasma is a deterministic, typed policy engine
for the agentic era. The Palimpsest license (PMPL) becomes a separate
future project.

### Added

- feat(engine): new `plasma-engine` crate — typed policy AST (deontic
  modalities, composable conditions, overlays), versioned TOML/JSON schema
  loading with load-time rejection of reserved constructs, deterministic
  fact collection, a pure/total evaluator, human/JSON/SARIF rendering
- feat(cli): `plasma check` (policy evaluation, exit-code gated),
  `plasma facts` (deterministic fact snapshot), `plasma policy validate`
- feat(cli): bundled repo-hygiene policy; the repo checks itself in tests
- feat(parser): `scan_repo` — deterministic zone-aware repository scanner
- feat(parser): raw SPDX header extraction (`extract_spdx_raw_from_content`)
- docs: `docs/engine-v0-design.adoc` — normative semantics and
  Catala-readiness guarantees

### Changed

- **BREAKING**: binary renamed `palimpsest-plasma` → `plasma`
- **BREAKING**: licensing unified — code MPL-2.0, documentation
  CC-BY-SA-4.0; all SPDX headers swept (PMPL/PPMPL identifiers survive only
  as parseable inputs to plasma-parser)
- `plasma audit` rewritten on plasma-parser (real SPDX expression parsing,
  `.plasma.toml` zone awareness) instead of substring matching
- `plasma init`/`migrate`/`badge` genericized to any SPDX license
  (MPL-2.0 default); Covenant embedding removed
- Root crate moved to edition 2021; workspace versions unified at 0.2.0
- README/EXPLAINME/TOPOLOGY/ROADMAP/architecture/cli-design rewritten to
  describe what exists (the OCaml engine described previously was never
  built; the design lives on in docs/policy-ast-v0.1.adoc as lineage)

### Fixed

- Workspace compiles again (init/migrate embedded a nonexistent license
  file)
- SPDX catalog PMPL/PPMPL identifier drift (2 failing tests now pass)
- codemeta.json/CITATION.cff described an unrelated boilerplate project

### Removed

- union-policy-parser sub-project (mostly non-compiling scaffolding;
  preserved in git history, belongs in its own repository)
- Corrupted `LICENSE-PMPL-2.0.txt`, stale MVP release tarballs, badge
  pack, signature files of removed artifacts, `PALIMPSEST-COVENANT.md`,
  `docs/mvp-v1.adoc`

## [0.1.0] - 2026-05/06 (previously listed as Unreleased)

### Added

- feat(crg): add crg-grade and crg-badge justfile recipes (#5)
- feat(crg): add crg-grade and crg-badge justfile recipes
- feat: add stapeln.toml container definition (#3)
- feat: deploy UX Manifesto infrastructure
- feat: add plasma-parser library crate (40 tests passing)
- feat: plasma CLI — one-command PMPL-2.0 adoption toolkit
- feat: add CLADE.a2ml — clade taxonomy declaration
- feat: integrate PLASMA engine architecture, Policy AST, and contractiles design
- feat: add CITATION.cff, codemeta.json, and .well-known/ files
- feat(site): replace jekyll with nimblepublisher pages deployment

### Fixed

- fix(ci): bump a2ml/k9-validate-action pins to canonical (standards#85) (#14)
- fix(ci): sync hypatia-scan.yml to canonical (kill cd-scanner build drift) (#13)
- fix(ci): build Hypatia escript from repo root (estate dogfood drift)
- fix(ci): rsr-antipattern.yml duplicate heredoc (#11)
- fix(scorecard): enforce granular permissions and add fuzzing placeholder
- fix(ci): Resolve workflow-linter self-matching and metadata issues
- fix: remove duplicate SCM files from root

### Changed

- refactor: consume contractile CLI instead of reimplementing
- refactor: migrate 6SCM → 6A2 (.scm → .a2ml format)

### Documentation

- docs: MPL-2.0 baseline with Palimpsest overlay wording
- docs: add RSR compliance anchors and archive plasma summary
- docs: add TEST-NEEDS.md and/or PROOF-NEEDS.md from audit
- docs: add family parser specification for palimpsest license variants
- docs: add EXPLAINME.adoc — prove-it file backing README claims
- docs: add CITATION.cff and codemeta.json
- docs: add CONTRIBUTING.md
- docs: refresh audit summaries

### CI

- ci(rust): convert rust-ci.yml to thin wrapper (standards#174) (#20)
- ci: redistribute concurrency-cancel guard to read-only check workflows (#16)
- ci: fix nonexistent actions/upload-artifact SHA pin (#12)
- ci: bump actions/upload-artifact SHA to current v4 (#10)
- ci(antipattern): fix top-level dir matching + benchmarks/lsp/bench filename allowlists (#9)

## Pre-history

Prior commits to this file's introduction are recorded in git history but not formally classified into Keep-a-Changelog sections. To backfill, run `git cliff -o CHANGELOG.md` locally using the canonical [`cliff.toml`](https://github.com/hyperpolymath/standards/blob/main/templates/cliff.toml) — this is one-shot mechanical work.

---

<!-- This file was seeded by the 2026-05-26 estate tech-debt audit follow-up (Row-2 Phase 3); see [`hyperpolymath/standards/docs/audits/2026-05-26-estate-documentation-debt.md`](https://github.com/hyperpolymath/standards/blob/main/docs/audits/2026-05-26-estate-documentation-debt.md). -->
