# SPDX-License-Identifier: PMPL-1.0-or-later
# Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
#
# Contractile CLI delegation recipes for palimpsest-plasma.
# These recipes delegate to the canonical contractile CLI from reposystem/contractiles/
# rather than reimplementing contractile logic locally.
#
# Usage: import this file from the root Justfile via:
#   import "contractiles/contractile.just"

# Run must checks via the contractile CLI
must-check *args:
    @echo "Delegating to contractile CLI: must check {{args}}"
    must check {{args}}

# Run trust verification via the contractile CLI
trust-verify *args:
    @echo "Delegating to contractile CLI: trust verify {{args}}"
    trust verify {{args}}

# Run dust status via the contractile CLI
dust-status *args:
    @echo "Delegating to contractile CLI: dust status {{args}}"
    dust status {{args}}

# Run intend list via the contractile CLI
intend-list *args:
    @echo "Delegating to contractile CLI: intend list {{args}}"
    intend list {{args}}

# Run K9 evaluation via the contractile CLI
k9-eval *args:
    @echo "Delegating to contractile CLI: k9 eval {{args}}"
    k9 eval {{args}}

# Run all contractile checks (must + trust + k9)
contractile-all:
    @echo "Running full contractile suite..."
    -must check
    -trust verify
    -k9 eval
    @echo "Contractile suite complete."
