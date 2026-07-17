# SPDX-License-Identifier: MPL-2.0
# Justfile — task runner for palimpsest-plasma (Rust workspace).
# https://just.systems/man/en/
#
# Run `just` to list recipes.

set shell := ["bash", "-uc"]
set dotenv-load := true
set positional-arguments := true

# Optional generated contractile recipes (estate tooling).
import? "contractile.just"

project := "palimpsest-plasma"
version := "0.6.0"
tier := "infrastructure"
state := ".machine_readable/6a2/STATE.a2ml"

# ═══════════════════════════════════════════════════════════════════════════
# DEFAULT & HELP
# ═══════════════════════════════════════════════════════════════════════════

# Show all available recipes
default:
    @just --list --unsorted

# Show detailed help for a specific recipe
help recipe="":
    #!/usr/bin/env bash
    if [ -z "{{recipe}}" ]; then
        just --list --unsorted
    else
        just --show "{{recipe}}" 2>/dev/null || echo "Recipe '{{recipe}}' not found"
    fi

# Show this project's info
info:
    @echo "Project:  {{project}}"
    @echo "Version:  {{version}}"
    @echo "RSR Tier: {{tier}}"
    @[ -f {{state}} ] && grep -E '^phase' {{state}} | head -1 || true

# ═══════════════════════════════════════════════════════════════════════════
# BUILD & TEST
# ═══════════════════════════════════════════════════════════════════════════

# Build the whole workspace (debug)
build *args:
    cargo build --workspace {{args}}

# Build the whole workspace (release)
build-release *args:
    cargo build --workspace --release {{args}}

# Run the test suite
test *args:
    cargo test --workspace {{args}}

# Format all code
fmt:
    cargo fmt --all

# Check formatting without modifying
fmt-check:
    cargo fmt --all -- --check

# Lint with clippy, warnings as errors
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Remove build artifacts
clean:
    cargo clean

# Run the plasma binary with arbitrary arguments
run *args:
    cargo run -- {{args}}

# ═══════════════════════════════════════════════════════════════════════════
# DOGFOOD — plasma checks itself
# ═══════════════════════════════════════════════════════════════════════════

# Evaluate this repository against the bundled repo-hygiene policy
check:
    cargo run -- check . --format human

# Dump the deterministic fact snapshot for this repository
facts:
    cargo run -- facts .

# Audit SPDX headers across this repository
audit:
    cargo run -- audit .

# Show the remediation plan for this repository (dry-run; add --apply to fix)
fix *args:
    cargo run -- fix . {{args}}

# Validate the bundled policy file
policy-check:
    cargo run -- policy validate plasma-engine/policies/repo-hygiene.plasma.toml

# ═══════════════════════════════════════════════════════════════════════════
# CI & DIAGNOSTICS
# ═══════════════════════════════════════════════════════════════════════════

# Everything CI runs: format check, lint, tests, and self-check
ci: fmt-check lint test check

# Diagnose the local toolchain
doctor:
    #!/usr/bin/env bash
    set -e
    echo "== toolchain =="
    command -v cargo >/dev/null && cargo --version || { echo "cargo: MISSING (install rustup)"; exit 1; }
    command -v just  >/dev/null && just --version  || echo "just: MISSING"
    echo "== workspace check =="
    cargo check --workspace

# Annotated tour of the repository layout
tour:
    #!/usr/bin/env bash
    echo "src/            — the plasma CLI (check/facts/policy/audit/init/badge/migrate)"
    echo "plasma-engine/  — policy AST, schema, facts, pure evaluator, reports"
    echo "plasma-parser/  — SPDX parsing, license families, zones, audit scan"
    echo "docs/           — engine-v0-design.adoc is the normative spec"
    echo "site/           — Elixir NimblePublisher guidance site"

# Pre-fill an issue report with local context
help-me:
    #!/usr/bin/env bash
    echo "## Environment"
    echo "- plasma: {{version}}"
    cargo --version 2>/dev/null || echo "- cargo: not found"
    uname -a

# Set up a development environment (fetch deps and build)
setup:
    cargo fetch
    cargo build --workspace
    @echo "Setup complete. Try: just check"
