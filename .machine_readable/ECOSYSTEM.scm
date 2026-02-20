;; SPDX-License-Identifier: PMPL-1.0-or-later
;; ECOSYSTEM.scm - Project relationship mapping

(ecosystem
  (version "2.0")
  (name "palimpsest-plasma")
  (type "engine")
  (purpose "Governance automation engine for the Palimpsest-MPL ecosystem. Interprets licence, exhibits, and governance rules as typed policies; evaluates repository state against obligations; produces findings, actions, and audit reports.")

  (position-in-ecosystem
    (role "governance-engine")
    (layer "infrastructure")
    (description "The operational, programmable substrate (PLASMA) that keeps the Palimpsest-MPL ecosystem correct, compliant, and narratively coherent over time."))

  (related-projects
    ((palimpsest-license
      (relationship . "upstream")
      (description . "Canonical PMPL-1.0 licence text and versioning. PLASMA enforces the rules defined here."))
     (palimpsest-governance
      (relationship . "sibling")
      (description . "Stewardship Council processes and artefacts. PLASMA automates enforcement of Council decisions."))
     (union-policy-parser
      (relationship . "sub-project")
      (description . "A2ML-based contract validation for union standards. Embedded in this repo."))
     (gitbot-fleet
      (relationship . "downstream-consumer")
      (description . "Bot orchestration. Will consume PLASMA findings for automated remediation."))
     (hypatia
      (relationship . "downstream-consumer")
      (description . "Neurosymbolic CI/CD intelligence. Consumes PLASMA audit logs."))))

  (technology-stack
    ((core . "OCaml (dune, opam)")
     (sub-project . "Rust (cargo, nom, clap)")
     (abi . "Idris2 (dependent types)")
     (ffi . "Zig (C ABI)")
     (site . "Elixir (NimblePublisher)")
     (metadata . "Guile Scheme (SCM files)")))

  (badges
    ("image:assets/badges/svg/badge-standard.svg[Palimpsest License v1.0]"))

  (what-this-is
    "PLASMA — the governance automation engine for Palimpsest-MPL. OCaml-based policy engine with typed AST, deontic operators, exhibit extensions, schema migration, contractiles, and multi-surface integration (CLI, git hooks, CI, daemon).")

  (what-this-is-not
    "Not the reference license repository (see palimpsest-license). Not the Council governance processes (see palimpsest-governance). Not a general-purpose policy engine — specifically designed for PMPL ecosystem governance."))
