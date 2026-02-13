;; SPDX-License-Identifier: PMPL-1.0-or-later
;; STATE.scm - Current project state

(define project-state
  `((metadata
      ((version . "1.1.0")
       (schema-version . "1")
       (created . "2026-01-10T13:50:48+00:00")
       (updated . "2026-02-13T00:00:00+00:00")
       (project . "palimpsest-plasma")
       (repo . "palimpsest-plasma")))

    (current-position
      ((phase . "Post-Release Metadata")
       (overall-completion . 85)
       (working-features . ("canonical-license" "badge-pack" "compliance-docs"
                            "audit-ready" "citation-cff" "codemeta-json"
                            "well-known-security" "well-known-ai"))))

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
         (audit-log . ((items . ("Run pmpl-verify --recursive --existence-only"
                                "Run pmpl-audit CLI to capture provenance status"))
                       (status . "in-progress")))
         (release . ((items . ("Bundle docs/assets for MVP v1 release"
                               "Publish audit summary and compliance checklist"
                               "Announce PMPL-1.0-or-later readiness"))
                     (status . "pending")))))))

    (blockers-and-issues
      ((critical . ())
       (high . ())
       (medium . ())
       (low . ())))

    (critical-next-actions
      ((immediate . ("Document pmpl-verify output in docs/mvp-v1.adoc"
                     "Capture pmpl-audit observations for release notes"))
       (this-week . ("Publish MVP v1 release assets" "Confirm badge CDN links"))
       (this-month . ("Gather feedback from Palimpsest Council" "Plan v1.2 follow-up"))))

    (session-history . ())))
