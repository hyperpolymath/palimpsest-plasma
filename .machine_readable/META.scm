;; SPDX-License-Identifier: PMPL-1.0-or-later
;; META.scm - Project metadata and architectural decisions

(define project-meta
  `((version . "2.0.0")
    (architecture-decisions
      ((adr-001 . ((title . "OCaml as core engine language")
                   (status . "accepted")
                   (date . "2026-01-13")
                   (context . "Need a language for the governance engine that provides type safety, correctness guarantees, and long-term maintainability")
                   (decision . "Use OCaml for the PLASMA core engine. Typed AST for obligations, rules, facts, and findings. Pure core with IO at boundaries.")
                   (consequences . ("Strong type safety for policy evaluation"
                                   "Pattern matching for rule evaluation"
                                   "Dune build system"
                                   "opam for package management"))))
       (adr-002 . ((title . "Deontic operators as first-class AST values")
                   (status . "accepted")
                   (date . "2026-01-13")
                   (context . "Palimpsest-MPL governance expresses obligations, prohibitions, and permissions. These deontic concepts must be mechanically evaluable.")
                   (decision . "Model Obligation, Prohibition, and Permission as a modality sum type in the Policy AST. Every rule carries a modality.")
                   (consequences . ("Rules can be mechanically evaluated against facts"
                                   "Findings carry deontic context"
                                   "Exhibits can target specific modalities"))))
       (adr-003 . ((title . "Exhibit-based policy extensions")
                   (status . "accepted")
                   (date . "2026-01-13")
                   (context . "PMPL uses exhibits to extend or modify the base licence. The policy engine must model this without mutating base rules.")
                   (decision . "Exhibit policies can AddRules, ModifyRules, or OverrideRules. Base policy remains immutable during evaluation.")
                   (consequences . ("Full audit trail: base vs exhibit rules"
                                   "Exhibits compose cleanly"
                                   "Rule provenance is always traceable"))))
       (adr-004 . ((title . "Versioned policy schemas with migration")
                   (status . "accepted")
                   (date . "2026-01-13")
                   (context . "Policy schemas will evolve. Prior audits must remain interpretable under new schemas.")
                   (decision . "Every policy carries a schema_version. Migration framework chains from_version -> to_version transforms. migrate_to_latest applies all steps.")
                   (consequences . ("Backward-compatible schema evolution"
                                   "Old audit logs remain valid"
                                   "Migration framework tested from day one"))))
       (adr-005 . ((title . "Contractiles as governance primitives")
                   (status . "proposed")
                   (date . "2026-02-02")
                   (context . "Need a primitive that represents reversible, tension-bearing governance bindings. Sits between a static rule and a dynamic finding.")
                   (decision . "Introduce contractile type: rule + state (Tightening/Holding/Relaxing/Released) + tension + observed conditions + exemptions.")
                   (consequences . ("Narrative compliance reporting"
                                   "Tension-based prioritisation"
                                   "Reversible governance bindings"
                                   "Constraint propagation across rules"))))
       (adr-006 . ((title . "Rust for union-policy-parser sub-project")
                   (status . "accepted")
                   (date . "2026-01-17")
                   (context . "A2ML parsing and contract validation needs performance, CLI capability, and cross-compilation.")
                   (decision . "Use Rust with nom for parsing, clap for CLI. Idris2 ABI + Zig FFI for formal interface definitions.")
                   (consequences . ("Fast A2ML parsing"
                                   "Type-safe contract validation"
                                   "Formal ABI proofs via Idris2"))))))
    (development-practices
      ((code-style . "ocamlformat")
       (security . "openssf-scorecard")
       (versioning . "semver")
       (documentation . "asciidoc")
       (branching . "trunk-based")
       (build-system . "dune")
       (package-manager . "opam")))
    (design-rationale
      ((plasma-meaning . "Programmable substrate for licence governance")
       (three-layer-arch . "core (pure) / integration (IO) / adapters (environment)")
       (narrative-coherence . "Findings and actions carry human-readable rationale tied to governance intent")))))
