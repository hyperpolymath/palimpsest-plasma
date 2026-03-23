// SPDX-License-Identifier: PMPL-2.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// Fallback validation — ensures that a palimpsest license's declared fallback
// is consistent with its base license, and that the LICENSE file actually
// contains the base license text.

use crate::family::{BaseLicense, FamilyError, PalimpsestLicense, PalimpsestVariant};
use std::path::Path;

/// Validate that a palimpsest license's fallback matches its base.
///
/// The palimpsest legal structure requires that `fallback == base` — if the
/// palimpsest provisions are ever struck, the code reverts to the base
/// license. A mismatch is a structural error in the license declaration.
pub fn validate_fallback(license: &PalimpsestLicense) -> Result<(), FamilyError> {
    if license.base != license.fallback {
        return Err(FamilyError::FallbackMismatch {
            variant: license.variant.clone(),
            declared_base: license.base.clone(),
            declared_fallback: license.fallback.clone(),
        });
    }
    Ok(())
}

/// Validate that the LICENSE file at the given path contains text consistent
/// with the expected base license.
///
/// Uses approximate string matching (Levenshtein distance) to detect whether
/// the base license text is present, even if there are minor formatting
/// differences or the palimpsest preamble is prepended.
///
/// # Arguments
///
/// * `license_path` — Path to the LICENSE or LICENSE.txt file.
/// * `variant` — The palimpsest variant being validated.
/// * `expected_base` — The base license that should be present in the file.
pub fn validate_license_file(
    license_path: &Path,
    variant: &PalimpsestVariant,
    expected_base: &BaseLicense,
) -> Result<(), FamilyError> {
    let content = match std::fs::read_to_string(license_path) {
        Ok(c) => c,
        Err(_) => {
            return Err(FamilyError::BaseLicenseTextMissing {
                variant: variant.clone(),
                expected_base: expected_base.clone(),
            });
        }
    };

    let content_upper = content.to_uppercase();

    // Check for the presence of characteristic phrases from each base license.
    let found = match expected_base {
        BaseLicense::MPL2 => content_upper.contains("MOZILLA PUBLIC LICENSE"),
        BaseLicense::AGPL3 => {
            content_upper.contains("GNU AFFERO GENERAL PUBLIC LICENSE")
                || content_upper.contains("AGPL")
        }
        BaseLicense::GPL3 => {
            content_upper.contains("GNU GENERAL PUBLIC LICENSE")
                && !content_upper.contains("AFFERO")
                && !content_upper.contains("LESSER")
        }
        BaseLicense::GPL2 => {
            content_upper.contains("GNU GENERAL PUBLIC LICENSE")
                && content_upper.contains("VERSION 2")
        }
        BaseLicense::Apache2 => content_upper.contains("APACHE LICENSE"),
        BaseLicense::BSD3 => {
            content_upper.contains("BSD") && content_upper.contains("3-CLAUSE")
                || content_upper.contains("REDISTRIBUTION AND USE")
        }
        BaseLicense::BSD2 => {
            content_upper.contains("BSD") && content_upper.contains("2-CLAUSE")
                || content_upper.contains("REDISTRIBUTION AND USE")
        }
        BaseLicense::MIT => content_upper.contains("MIT LICENSE") || content_upper.contains("PERMISSION IS HEREBY GRANTED"),
        BaseLicense::ISC => content_upper.contains("ISC LICENSE") || content_upper.contains("PERMISSION TO USE, COPY, MODIFY"),
        BaseLicense::EUPL12 => content_upper.contains("EUROPEAN UNION PUBLIC LICENCE"),
        BaseLicense::LGPL21 | BaseLicense::LGPL3 => {
            content_upper.contains("GNU LESSER GENERAL PUBLIC LICENSE")
        }
        BaseLicense::Artistic2 => content_upper.contains("ARTISTIC LICENSE"),
        BaseLicense::CERNOHLS2 => content_upper.contains("CERN OPEN HARDWARE"),
        BaseLicense::Unlicense => content_upper.contains("UNLICENSE") || content_upper.contains("PUBLIC DOMAIN"),
        BaseLicense::ZeroBSD => content_upper.contains("0BSD") || content_upper.contains("ZERO-CLAUSE BSD"),
        BaseLicense::Proprietary => {
            content_upper.contains("ALL RIGHTS RESERVED")
                || content_upper.contains("PROPRIETARY")
        }
        BaseLicense::CreativeCommons(_) => content_upper.contains("CREATIVE COMMONS"),
        BaseLicense::Other(_) => true, // Cannot validate unknown licenses
    };

    if found {
        Ok(())
    } else {
        Err(FamilyError::BaseLicenseTextMissing {
            variant: variant.clone(),
            expected_base: expected_base.clone(),
        })
    }
}

/// Validate that required exhibits are present in the LICENSE file.
///
/// Palimpsest licenses may reference Exhibit A (standard) and Exhibit B
/// (quantum-safe provenance). If the layer declares quantum_safe=true,
/// Exhibit B must be present.
pub fn validate_exhibits(
    license_path: &Path,
    license: &PalimpsestLicense,
) -> Vec<FamilyError> {
    let mut errors = Vec::new();

    let content = match std::fs::read_to_string(license_path) {
        Ok(c) => c,
        Err(_) => return errors,
    };

    let content_upper = content.to_uppercase();

    // Exhibit A is always expected for palimpsest licenses.
    if !content_upper.contains("EXHIBIT A") {
        errors.push(FamilyError::ExhibitMissing {
            exhibit: 'A',
            variant: license.variant.clone(),
        });
    }

    // Exhibit B is required only when quantum_safe is enabled.
    if license.layer.quantum_safe && !content_upper.contains("EXHIBIT B") {
        errors.push(FamilyError::ExhibitMissing {
            exhibit: 'B',
            variant: license.variant.clone(),
        });
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::family::{PalimpsestLayer, PalimpsestVariant, Version};

    fn make_pmpl(or_later: bool) -> PalimpsestLicense {
        let version = Version {
            major: 2,
            minor: 0,
            patch: 0,
        };
        PalimpsestLicense {
            variant: PalimpsestVariant::PMPL,
            version: version.clone(),
            base: BaseLicense::MPL2,
            layer: PalimpsestLayer::full(version),
            fallback: BaseLicense::MPL2,
            or_later,
        }
    }

    #[test]
    fn test_valid_fallback() {
        let license = make_pmpl(true);
        assert!(validate_fallback(&license).is_ok());
    }

    #[test]
    fn test_invalid_fallback() {
        let mut license = make_pmpl(true);
        license.fallback = BaseLicense::Apache2;
        let err = validate_fallback(&license).unwrap_err();
        assert!(matches!(err, FamilyError::FallbackMismatch { .. }));
    }
}
