<!--
SPDX-License-Identifier: CC-BY-SA-4.0
Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->
<!-- TOPOLOGY.md — Project architecture map and completion dashboard -->
<!-- Last updated: 2026-07-02 -->

# Palimpsest Plasma — Project Topology

## System Architecture

```
                 ┌─────────────────────────────────────────────────────┐
                 │                  plasma (CLI)                       │
                 │  check · facts · policy · audit · init · badge ·    │
                 │  migrate                                            │
                 └────────────┬───────────────────────┬────────────────┘
                              │                       │
                 ┌────────────▼────────────┐ ┌────────▼────────────────┐
                 │      plasma-engine      │ │      plasma-parser      │
                 │                         │ │                         │
                 │  Policy AST (deontic)   │ │  SPDX lexer + parser    │
                 │  Schema load/validate   │ │  Identifier catalog     │
                 │  Fact collection (IO)   │ │  License family types   │
                 │  Pure evaluator         │ │  Zones (.plasma.toml)   │
                 │  Findings → human/      │ │  Repo scanner           │
                 │  JSON/SARIF             │ │  Compat matrix          │
                 └────────────┬────────────┘ └─────────────────────────┘
                              │        (engine depends on parser)
                              ▼
        ┌──────────────────────────────────────────────────────────────┐
        │              AGENT-HONESTY TRIO (planned siblings)           │
        │                                                              │
        │  palimpsest-plasma      somethings-fishy    did-you-actually-│
        │  (policy definition +   (forensic bot-      do-that (claim   │
        │  deterministic          damage              verification)    │
        │  evaluation)            investigation)                       │
        │                                                              │
        │  Contract: JSON findings · SARIF plasma/<rule-id> ·          │
        │            plasma facts snapshots (diffable)                 │
        └──────────────────────────────────────────────────────────────┘
```

## Completion Dashboard

```
COMPONENT                          STATUS              NOTES
─────────────────────────────────  ──────────────────  ─────────────────────────────────
PLASMA-PARSER (Rust)
  SPDX Lexer/Parser/Catalog         █████████░  90%    42 unit tests; PMPL/PPMPL drift fixed
  License Family Types              ████████░░  80%    Registry + fallback chains
  Zone System (.plasma.toml)        ████████░░  80%    Boundary parsing, overlap detection
  Repo Scanner (scan_repo)          ████████░░  80%    Deterministic walk, zone-aware
  Compat Matrix                     ███████░░░  70%    Pairwise checks; no report surface yet
  JSON/SARIF Reports                ████████░░  80%    RepoAudit renderers

PLASMA-ENGINE (Rust)
  Policy AST v0.1                   █████████░  90%    Typed, serde TOML/JSON
  Schema Load + Validation          █████████░  90%    Load-time rejection of reserved constructs
  Fact Collection                   ████████░░  80%    Files, headers, Cargo version, git HEAD
  Pure Evaluator                    █████████░  90%    Total; deterministic; 19 unit tests
  Human/SARIF Rendering             ████████░░  80%    Stable plasma/<rule-id> namespace
  Overlays (add-rules)              ████████░░  80%    modify/override reserved for v0.3

CLI (plasma)
  check / facts / policy validate   █████████░  90%    Self-tested against this repo
  audit (parser-backed)             ████████░░  80%    Zone-aware; --fix
  init / migrate / badge            ████████░░  80%    License-agnostic, MPL-2.0 default
  fix (action planner)              ░░░░░░░░░░   0%    Roadmap v0.3
  Git hooks / CI action / daemon    ░░░░░░░░░░   0%    Roadmap v0.4

INFRASTRUCTURE
  Elixir Site (NimblePublisher)     ███████░░░  70%    Render helpers restored; CI-verified
  Justfile (cargo)                  █████████░  90%    Rewritten from stale OCaml recipes
  .machine_readable/                ███████░░░  70%    STATE/ECOSYSTEM truth-updated
  CI (GitHub reusables + GitLab)    ██████░░░░  60%    Reusables externally pinned/unverifiable

─────────────────────────────────────────────────────────────────────────────
OVERALL:                            █████░░░░░  ~50%   Engine v0 shipped; planner + surfaces ahead
```

## Key Dependencies

```
Policy AST ──► Schema Loader ──► Pure Evaluator ──► Findings ──► human/JSON/SARIF
                                      ▲
Fact Collector (walk, SPDX headers, ──┘
metadata, git) ── uses plasma-parser header extraction

plasma audit ──► plasma-parser scan_repo ──► zone map (.plasma.toml)
```

## Update Protocol

This file is maintained by both humans and AI agents. When updating:

1. **After completing a component**: Change its bar and percentage
2. **After adding a component**: Add a new row in the appropriate section
3. **After architectural changes**: Update the ASCII diagram
4. **Date**: Update the `Last updated` comment at the top of this file

Progress bars use: `█` (filled) and `░` (empty), 10 characters wide.
Percentages: 0%, 10%, 20%, ... 100% (in 10% increments).
Honesty rule: a component that does not compile or run is 0–20%,
whatever its design documents say.
