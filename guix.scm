;; SPDX-License-Identifier: MPL-2.0
;; guix.scm — GNU Guix package definition for palimpsest-plasma
;; Usage: guix shell -f guix.scm   (development shell)
;;
;; Note: cargo inputs are not vendored here — this definition provides a
;; development shell (rust + cargo), not a fully reproducible build. Vendor
;; the crate inputs if a reproducible `guix build` is needed.

(use-modules (guix packages)
             (guix build-system cargo)
             (guix gexp)
             ((guix licenses) #:prefix license:)
             (gnu packages rust))

(package
  (name "palimpsest-plasma")
  (version "0.2.0")
  (source (local-file "." "palimpsest-plasma-checkout"
                      #:recursive? #t))
  (build-system cargo-build-system)
  (arguments '(#:tests? #f))
  (synopsis "Deterministic, typed policy engine")
  (description
   "palimpsest-plasma is a deterministic, typed policy engine: define
machine-readable deontic policies and evaluate repositories against them with
reproducible results.  Includes an SPDX expression parser and zone-aware
license auditing.")
  (home-page "https://github.com/hyperpolymath/palimpsest-plasma")
  (license license:mpl2.0))
