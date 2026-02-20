<!-- SPDX-License-Identifier: PMPL-1.0-or-later -->
<!-- TOPOLOGY.md — Project architecture map and completion dashboard -->
<!-- Last updated: 2026-02-20 -->

# Palimpsest Plasma — Project Topology

## System Architecture

```
                   ┌──────────────────────────────────────────────────────────┐
                   │                   PLASMA ENGINE                          │
                   │                                                          │
                   │  ┌────────────────────────────────────────────────────┐  │
                   │  │                    CORE (OCaml)                    │  │
                   │  │                                                    │  │
                   │  │  Policy AST ──► Eval Engine ──► Action Planner    │  │
                   │  │       │              │               │             │  │
                   │  │  Schema &        Findings &      Suggest /        │  │
                   │  │  Migration       Severity        Apply            │  │
                   │  │       │              │               │             │  │
                   │  │  Governance ◄── Contractiles ──► Audit Logs       │  │
                   │  │  Runtime                                          │  │
                   │  └────────────────────────────────────────────────────┘  │
                   │                          │                               │
                   │  ┌───────────────┐  ┌────┴──────┐  ┌──────────────┐    │
                   │  │  ADAPTERS     │  │INTEGRATION│  │ UNION POLICY │    │
                   │  │               │  │           │  │  PARSER      │    │
                   │  │ Repo Scanner  │  │ CLI       │  │              │    │
                   │  │ SPDX Parser   │  │ Git Hooks │  │ A2ML Parser  │    │
                   │  │ Exhibit Parse │  │ CI/CD     │  │ Validator    │    │
                   │  │ Metadata I/O  │  │ Daemon    │  │ Grievance Gen│    │
                   │  └───────────────┘  └───────────┘  └──────────────┘    │
                   └──────────────────────────────────────────────────────────┘
                                          │
                   ┌──────────────────────┼──────────────────────────────────┐
                   │            PALIMPSEST-MPL ECOSYSTEM                     │
                   │                      │                                  │
                   │  palimpsest-license   palimpsest-governance             │
                   │  (PMPL-1.0 text)     (Council processes)               │
                   └──────────────────────────────────────────────────────────┘
```

## Completion Dashboard

```
COMPONENT                          STATUS              NOTES
─────────────────────────────────  ──────────────────  ─────────────────────────────────
MVP v1 BUNDLE (COMPLETE)
  Canonical License Tooling         ██████████ 100%    PMPL-1.0 alignment stable
  Badge & Documentation Assets      ██████████ 100%    v1.0 brand pack shipped
  Compliance Readiness              ██████████ 100%    SPDX metadata verified
  Audit/Playback Support            ████████░░  80%    pmpl-verify examples active
  Citation & Metadata               ██████████ 100%    CITATION.cff + codemeta.json

UNION POLICY PARSER (Rust/A2ML)
  A2ML Parser                       ██████████ 100%    nom-based, full test suite
  Union Schemas (NUJ/IWW/UCU/+3)   ██████████ 100%    7 schemas with attestations
  Contract Validator                ██████████ 100%    3 modes: Lax/Checked/Attested
  Grievance Generator               ██████████ 100%    JSON/Markdown/HTML output
  Idris2 ABI + Zig FFI             ████████░░  80%    Template stubs, needs customisation

PLASMA ENGINE (OCaml — IN DESIGN)
  Policy AST v0.1                   ██░░░░░░░░  20%    Types specified, not implemented
  Schema Migration Framework        ░░░░░░░░░░   0%    Design complete
  Evaluation Engine                 ░░░░░░░░░░   0%    Design complete
  Action Planner                    ░░░░░░░░░░   0%    Design complete
  Governance Runtime                ░░░░░░░░░░   0%    Design complete
  Contractiles                      █░░░░░░░░░  10%    Concept documented

INTEGRATION SURFACES
  CLI (plasma check/fix/audit)      ░░░░░░░░░░   0%    Design complete (cli-design.adoc)
  Git Hooks                         ░░░░░░░░░░   0%    Planned
  GitHub Actions                    ░░░░░░░░░░   0%    Planned
  Daemon/Cron Mode                  ░░░░░░░░░░   0%    Planned

STATIC SITE & INFRASTRUCTURE
  Elixir Site (NimblePublisher)     ██████████ 100%    mix site.build verified
  Justfile Automation               ██████████ 100%    Standard build tasks
  .machine_readable/                ██████████ 100%    STATE tracking active
  Multi-Forge Synchronization       ██████████ 100%    GH/GL sync stable

─────────────────────────────────────────────────────────────────────────────
OVERALL:                            ████░░░░░░  40%    MVP done; engine in design phase
```

## Key Dependencies

```
Policy AST ───► Eval Engine ────► Action Planner ──► CLI / Hooks / CI
     │               │                │
     ▼               ▼                ▼
Migration ───► Fact Adapters ──► Governance Runtime ──► Audit Logs
     │               │                │
     ▼               ▼                ▼
Contractiles  Union Policy      Exhibit Lifecycle
              Parser (A2ML)     (Council Decisions)
```

## Update Protocol

This file is maintained by both humans and AI agents. When updating:

1. **After completing a component**: Change its bar and percentage
2. **After adding a component**: Add a new row in the appropriate section
3. **After architectural changes**: Update the ASCII diagram
4. **Date**: Update the `Last updated` comment at the top of this file

Progress bars use: `█` (filled) and `░` (empty), 10 characters wide.
Percentages: 0%, 10%, 20%, ... 100% (in 10% increments).
