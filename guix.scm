; SPDX-License-Identifier: PMPL-1.0-or-later
;; guix.scm — GNU Guix package definition for palimpsest-plasma
;; Usage: guix shell -f guix.scm

(use-modules (guix packages)
             (guix build-system gnu)
             (guix licenses))

(package
  (name "palimpsest-plasma")
  (version "0.1.0")
  (source #f)
  (build-system gnu-build-system)
  (synopsis "palimpsest-plasma")
  (description "palimpsest-plasma — part of the hyperpolymath ecosystem.")
  (home-page "https://github.com/hyperpolymath/palimpsest-plasma")
  (license ((@@ (guix licenses) license) "PMPL-1.0-or-later"
             "https://github.com/hyperpolymath/palimpsest-license")))
