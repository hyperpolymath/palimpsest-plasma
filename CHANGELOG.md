<!--
SPDX-License-Identifier: MPL-2.0
SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell (hyperpolymath)
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
