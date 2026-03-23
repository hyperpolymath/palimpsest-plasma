// SPDX-License-Identifier: PMPL-2.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// License compatibility matrix — determines whether two licenses can
// coexist in the same repository, binary, or project.
//
// The matrix encodes well-known compatibility rules from the SPDX and
// FSF compatibility databases, plus palimpsest-specific rules from the
// family parser spec.

use crate::compat::Compatibility;
use crate::family::{BaseLicense, License, PalimpsestVariant};

/// Check compatibility between two licenses.
///
/// Applies the palimpsest family compatibility rules first, then falls
/// back to standard SPDX compatibility heuristics.
///
/// # Key rules (from the spec):
///
/// - PAGPL + AGPL-3.0 → Compatible (PAGPL is a superset)
/// - PAGPL + GPL-3.0 → Compatible (AGPL is GPL-compatible)
/// - PAGPL + PMPL → Conditional (file-level separation)
/// - PAGPL + Proprietary → ProprietaryDepends (AGPL prohibits)
/// - PMPL + Proprietary → Compatible (MPL allows per-file mixing)
/// - PAPL + Anything → Compatible (Apache is permissive)
/// - PGPL + AGPL-3.0 → Compatible (GPL→AGPL is allowed)
/// - PGPL + MIT → Compatible (MIT→GPL is allowed)
pub fn check_compatibility(a: &License, b: &License) -> Compatibility {
    // Same license is always compatible.
    if a == b {
        return Compatibility::Compatible;
    }

    // Extract base licenses for comparison.
    let base_a = extract_base(a);
    let base_b = extract_base(b);

    // Handle proprietary on either side.
    if base_a == BaseLicense::Proprietary || base_b == BaseLicense::Proprietary {
        return check_proprietary_compat(a, b);
    }

    // Handle palimpsest-specific rules.
    if let Some(compat) = check_palimpsest_compat(a, b) {
        return compat;
    }

    // Fall back to base license compatibility.
    check_base_compat(&base_a, &base_b)
}

/// Extract the effective base license from a License.
///
/// For palimpsest licenses, this returns the base that the palimpsest
/// extends. For base licenses, it returns the license itself.
fn extract_base(license: &License) -> BaseLicense {
    match license {
        License::Base(base) => base.clone(),
        License::Palimpsest(p) => p.base.clone(),
    }
}

/// Check compatibility when one side is proprietary.
fn check_proprietary_compat(a: &License, b: &License) -> Compatibility {
    let (open, _proprietary) = if matches!(extract_base(a), BaseLicense::Proprietary) {
        (b, a)
    } else {
        (a, b)
    };

    let open_base = extract_base(open);
    match open_base {
        // Strong copyleft prohibits proprietary combination.
        BaseLicense::AGPL3 => Compatibility::ProprietaryDepends(
            "AGPL-3.0 prohibits proprietary combination in linked binaries".to_string(),
        ),
        BaseLicense::GPL3 | BaseLicense::GPL2 => Compatibility::ProprietaryDepends(
            "GPL prohibits proprietary combination in linked binaries".to_string(),
        ),
        // Weak copyleft allows per-file separation.
        BaseLicense::MPL2 => Compatibility::Compatible,
        BaseLicense::LGPL21 | BaseLicense::LGPL3 => Compatibility::Conditional(
            "LGPL allows proprietary linking but modified LGPL files must remain open".to_string(),
        ),
        // Permissive licenses are fine with proprietary.
        BaseLicense::MIT
        | BaseLicense::Apache2
        | BaseLicense::BSD2
        | BaseLicense::BSD3
        | BaseLicense::ISC
        | BaseLicense::Unlicense
        | BaseLicense::ZeroBSD => Compatibility::Compatible,
        // EUPL has some compatibility provisions.
        BaseLicense::EUPL12 => Compatibility::Conditional(
            "EUPL-1.2 has a compatibility list; check if proprietary use is covered".to_string(),
        ),
        _ => Compatibility::Unknown,
    }
}

/// Check compatibility rules specific to palimpsest variants.
///
/// Returns `Some(Compatibility)` if a palimpsest-specific rule applies,
/// or `None` to fall through to base license comparison.
fn check_palimpsest_compat(a: &License, b: &License) -> Option<Compatibility> {
    let (palimpsest, other) = match (a, b) {
        (License::Palimpsest(p), other) => (p, other),
        (other, License::Palimpsest(p)) => (p, other),
        _ => return None,
    };

    let other_base = extract_base(other);

    match &palimpsest.variant {
        PalimpsestVariant::PAGPL => match &other_base {
            BaseLicense::AGPL3 => {
                Some(Compatibility::Compatible)
            }
            BaseLicense::GPL3 => {
                Some(Compatibility::Compatible)
            }
            BaseLicense::MPL2 => Some(Compatibility::Conditional(
                "file-level separation required (AGPL copyleft vs MPL per-file)".to_string(),
            )),
            _ => None,
        },
        PalimpsestVariant::PMPL => match &other_base {
            BaseLicense::MPL2 => {
                Some(Compatibility::Compatible)
            }
            _ => None,
        },
        PalimpsestVariant::PGPL => match &other_base {
            BaseLicense::AGPL3 => {
                Some(Compatibility::Compatible)
            }
            BaseLicense::GPL3 => {
                Some(Compatibility::Compatible)
            }
            BaseLicense::MIT | BaseLicense::BSD3 | BaseLicense::BSD2 => {
                Some(Compatibility::Compatible)
            }
            _ => None,
        },
        PalimpsestVariant::PAPL => {
            // Apache-based is permissive — compatible with nearly everything.
            Some(Compatibility::Compatible)
        }
        PalimpsestVariant::PBSD => {
            // BSD-based is permissive.
            Some(Compatibility::Compatible)
        }
        PalimpsestVariant::PEUPL => match &other_base {
            BaseLicense::GPL3 | BaseLicense::AGPL3 | BaseLicense::LGPL3 => {
                Some(Compatibility::Compatible)
            }
            _ => None,
        },
        PalimpsestVariant::Other(_) => None,
    }
}

/// Check compatibility between two base licenses using standard rules.
fn check_base_compat(a: &BaseLicense, b: &BaseLicense) -> Compatibility {
    // Permissive licenses are compatible with everything open-source.
    let permissive = [
        BaseLicense::MIT,
        BaseLicense::Apache2,
        BaseLicense::BSD2,
        BaseLicense::BSD3,
        BaseLicense::ISC,
        BaseLicense::Unlicense,
        BaseLicense::ZeroBSD,
    ];

    let a_permissive = permissive.contains(a);
    let b_permissive = permissive.contains(b);

    if a_permissive && b_permissive {
        return Compatibility::Compatible;
    }

    if a_permissive || b_permissive {
        // Permissive + anything open-source is generally fine
        // (permissive code can be incorporated into copyleft projects).
        return Compatibility::Compatible;
    }

    // Copyleft vs copyleft.
    match (a, b) {
        (BaseLicense::GPL3, BaseLicense::AGPL3)
        | (BaseLicense::AGPL3, BaseLicense::GPL3) => Compatibility::Compatible,

        (BaseLicense::GPL3, BaseLicense::LGPL3)
        | (BaseLicense::LGPL3, BaseLicense::GPL3) => Compatibility::Compatible,

        (BaseLicense::GPL3, BaseLicense::MPL2)
        | (BaseLicense::MPL2, BaseLicense::GPL3) => Compatibility::Conditional(
            "MPL-2.0 Section 3.3 allows relicensing under GPL".to_string(),
        ),

        (BaseLicense::AGPL3, BaseLicense::MPL2)
        | (BaseLicense::MPL2, BaseLicense::AGPL3) => Compatibility::Conditional(
            "file-level separation required (AGPL copyleft vs MPL per-file)".to_string(),
        ),

        (BaseLicense::GPL2, BaseLicense::GPL3)
        | (BaseLicense::GPL3, BaseLicense::GPL2) => Compatibility::Incompatible(
            "GPL-2.0-only and GPL-3.0 are incompatible without 'or later' clause".to_string(),
        ),

        _ => Compatibility::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_license_compatible() {
        let mit = License::Base(BaseLicense::MIT);
        assert_eq!(check_compatibility(&mit, &mit), Compatibility::Compatible);
    }

    #[test]
    fn test_mit_apache_compatible() {
        let mit = License::Base(BaseLicense::MIT);
        let apache = License::Base(BaseLicense::Apache2);
        assert_eq!(
            check_compatibility(&mit, &apache),
            Compatibility::Compatible
        );
    }

    #[test]
    fn test_agpl_proprietary() {
        let agpl = License::Base(BaseLicense::AGPL3);
        let prop = License::Base(BaseLicense::Proprietary);
        assert!(matches!(
            check_compatibility(&agpl, &prop),
            Compatibility::ProprietaryDepends(_)
        ));
    }

    #[test]
    fn test_mpl_proprietary_compatible() {
        let mpl = License::Base(BaseLicense::MPL2);
        let prop = License::Base(BaseLicense::Proprietary);
        assert_eq!(
            check_compatibility(&mpl, &prop),
            Compatibility::Compatible
        );
    }

    #[test]
    fn test_papl_anything_compatible() {
        use crate::family::{PalimpsestLayer, PalimpsestLicense, Version};

        let version = Version { major: 1, minor: 0, patch: 0 };
        let papl = License::Palimpsest(PalimpsestLicense {
            variant: PalimpsestVariant::PAPL,
            version: version.clone(),
            base: BaseLicense::Apache2,
            layer: PalimpsestLayer::full(version),
            fallback: BaseLicense::Apache2,
            or_later: true,
        });
        let agpl = License::Base(BaseLicense::AGPL3);
        assert_eq!(
            check_compatibility(&papl, &agpl),
            Compatibility::Compatible
        );
    }
}
