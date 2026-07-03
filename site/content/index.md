# SPDX-License-Identifier: CC-BY-SA-4.0
# Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
%{
  title: "Palimpsest Plasma",
  slug: "index",
  description: "A deterministic, typed policy engine for the agentic era.",
  date: ~D[2026-07-02]
}
---

## Overview

Palimpsest Plasma is a deterministic, typed **policy engine**. You write
policies as machine-readable deontic rules — obligations, prohibitions, and
permissions over repository state — and `plasma check` evaluates a repository
against them with byte-for-byte reproducible results.

- **Engine:** typed policy AST, versioned TOML/JSON schema, a pure and total
  evaluator (`plasma-engine/`)
- **License tooling:** SPDX header auditing over a zone-aware license map
  (`plasma-parser/`)
- **Agent-honesty contract:** JSON findings, SARIF results with stable
  `plasma/<rule-id>` identifiers, and diffable `plasma facts` snapshots

## Quickstart

```
cargo build --release
./target/release/plasma check .
./target/release/plasma facts .
```

## Building this site locally

Install Elixir (1.15+), then from the `site/` directory run `mix site.build`.
The command regenerates `site/_site`, which GitHub Actions uploads to Pages.
