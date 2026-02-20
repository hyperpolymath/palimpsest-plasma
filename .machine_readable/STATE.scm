;; SPDX-License-Identifier: PMPL-1.0-or-later
;; STATE.scm - Current project state

(define project-state
  `((metadata
      ((version . "2.0.0-dev")
       (schema-version . "1")
       (created . "2026-01-10T13:50:48+00:00")
       (updated . "2026-02-20T00:00:00+00:00")
       (project . "palimpsest-plasma")
       (repo . "palimpsest-plasma")))

    (current-position
      ((phase . "PLASMA Engine Design-to-Implementation")
       (overall-completion . 40)
       (working-features . ("canonical-license" "badge-pack" "compliance-docs"
                            "audit-ready" "citation-cff" "codemeta-json"
                            "well-known-security" "well-known-ai"
                            "a2ml-parser" "union-schemas" "contract-validator"
                            "grievance-generator" "architecture-docs"
                            "policy-ast-spec" "cli-design-spec"
                            "contractiles-concept"))))

    (route-to-mvp
      ((milestones
        ((v1.0.0 . ((items . ("Finalize PMPL-1.0 license text and README"
                            "Ship badge + embed documentation from docs/mvp-v1.adoc"
                            "Update ECOSYSTEM/STATE metadata and release log"))
                    (status . "complete")))
         (v1.1.0 . ((items . ("Add CITATION.cff for academic citation support"
                              "Add codemeta.json for software metadata interoperability"
                              "Add .well-known/security.txt and .well-known/ai.txt"
                              "Update STATE.scm to reflect additions"))
                    (status . "complete")))
         (union-policy-parser-v0.1 . ((items . ("A2ML parser with nom"
                                                "Union schema definitions (NUJ, IWW, UCU, BECTU, Equity, GMB)"
                                                "Contract validator (Lax, Checked, Attested)"
                                                "Grievance generator (JSON, Markdown, HTML)"
                                                "Idris2 ABI + Zig FFI layer"))
                                      (status . "complete")))
         (v2.0.0 . ((items . ("OCaml project layout (dune-project, opam, src/core)"
                              "Policy AST v0.1 implementation"
                              "Schema versioning and migration framework"
                              "Fact collection types and adapters"
                              "Finding types with severity and kind"))
                    (status . "in-progress")))
         (v2.1.0 . ((items . ("Condition compiler"
                              "Rule evaluator"
                              "Policy evaluator"
                              "Integration tests"))
                    (status . "pending")))
         (v2.2.0 . ((items . ("Action types"
                              "Suggest vs Apply modes"
                              "Planner: findings to planned actions"))
                    (status . "pending")))
         (v2.3.0 . ((items . ("plasma check CLI"
                              "plasma fix CLI"
                              "plasma audit CLI"
                              "plasma governance CLI"
                              "plasma migrate CLI"))
                    (status . "pending")))
         (v2.4.0 . ((items . ("Git hooks integration"
                              "GitHub Actions reference workflow"
                              "Daemon/cron mode"))
                    (status . "pending")))))))

    (blockers-and-issues
      ((critical . ())
       (high . ())
       (medium . ("OCaml dune-project needs initial setup"
                  "opam package needs definition"))
       (low . ())))

    (critical-next-actions
      ((immediate . ("Set up dune-project and plasma.opam"
                     "Implement Policy AST v0.1 types in OCaml"
                     "Write migration framework skeleton"))
       (this-week . ("Implement fact collection types"
                     "Implement finding types"
                     "Write mock migration tests"))
       (this-month . ("Build evaluation engine"
                      "Build action planner"
                      "Prototype CLI entry point"))))

    (session-history . ())))
