;; SPDX-License-Identifier: PMPL-1.0-or-later
;; ECOSYSTEM.scm - Project relationship mapping

(ecosystem
  (version "1.0")
  (name "palimpsest-plasma")
  (type "project")
  (purpose "Palimpsest ecosystem component")

  (position-in-ecosystem
    (role "component")
    (layer "application")
    (description "Palimpsest ecosystem component"))

  (related-projects
    ("palimpsest-license"))

  (badges
    ("image:assets/badges/svg/badge-standard.svg[Palimpsest License v1.0]"))

  (what-this-is
    "Palimpsest Plasma repository for ongoing development.")

  (what-this-is-not
    "Not the reference license repository (see palimpsest-license)."))
