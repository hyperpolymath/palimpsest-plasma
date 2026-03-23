// SPDX-License-Identifier: PMPL-2.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// SPDX identifier catalog — maps known SPDX identifiers (including all
// palimpsest family variants) to structured License types.

use crate::family::{
    BaseLicense, License, PalimpsestLayer, PalimpsestLicense, PalimpsestVariant, Version,
};
use thiserror::Error;

/// Errors produced during identifier resolution.
#[derive(Debug, Clone, Error)]
pub enum CatalogError {
    #[error("unknown SPDX identifier: {0}")]
    UnknownIdentifier(String),
}

/// Resolve an SPDX identifier string (with optional or-later flag) to a
/// structured [`License`].
///
/// This function recognises all standard SPDX identifiers as well as the
/// palimpsest family extensions (PMPL, PAGPL, PGPL, PAPL, PBSD, PEUPL).
///
/// # Arguments
///
/// * `id` — The identifier string, e.g. "MIT", "PMPL-2.0", "Apache-2.0".
/// * `or_later` — Whether the "-or-later" or "+" suffix was present.
///
/// # Returns
///
/// A [`License`] variant, or an error if the identifier is unrecognised
/// (in practice, unrecognised identifiers fall through to `BaseLicense::Other`).
pub fn resolve_identifier(id: &str, or_later: bool) -> Result<License, CatalogError> {
    // Extract the name portion (before any version) for palimpsest matching.
    let name_part = id
        .split('-')
        .next()
        .unwrap_or(id)
        .to_uppercase();

    // Try palimpsest variants first.
    if let Some(license) = try_palimpsest(&name_part, id, or_later) {
        return Ok(license);
    }

    // Try well-known base licenses.
    let base = match id {
        "MIT" => BaseLicense::MIT,
        "Apache-2.0" => BaseLicense::Apache2,
        "GPL-2.0" | "GPL-2.0-only" => BaseLicense::GPL2,
        "GPL-3.0" | "GPL-3.0-only" => BaseLicense::GPL3,
        "LGPL-2.1" | "LGPL-2.1-only" => BaseLicense::LGPL21,
        "LGPL-3.0" | "LGPL-3.0-only" => BaseLicense::LGPL3,
        "AGPL-3.0" | "AGPL-3.0-only" => BaseLicense::AGPL3,
        "MPL-2.0" => BaseLicense::MPL2,
        "BSD-2-Clause" => BaseLicense::BSD2,
        "BSD-3-Clause" => BaseLicense::BSD3,
        "ISC" => BaseLicense::ISC,
        "EUPL-1.2" => BaseLicense::EUPL12,
        "Artistic-2.0" => BaseLicense::Artistic2,
        "CERN-OHL-S-2.0" => BaseLicense::CERNOHLS2,
        "Unlicense" => BaseLicense::Unlicense,
        "0BSD" => BaseLicense::ZeroBSD,
        "LicenseRef-Proprietary" | "NONE" => BaseLicense::Proprietary,
        _ if id.starts_with("CC-") => {
            return Ok(License::Base(parse_cc_identifier(id)));
        }
        _ => BaseLicense::Other(id.to_string()),
    };

    Ok(License::Base(base))
}

/// Attempt to resolve the identifier as a palimpsest family variant.
///
/// Returns `Some(License::Palimpsest(...))` if the name matches a known
/// palimpsest prefix, or `None` otherwise.
fn try_palimpsest(name_upper: &str, full_id: &str, or_later: bool) -> Option<License> {
    let (variant, base) = match name_upper {
        "PMPL" => (PalimpsestVariant::PMPL, BaseLicense::MPL2),
        "PAGPL" => (PalimpsestVariant::PAGPL, BaseLicense::AGPL3),
        "PGPL" => (PalimpsestVariant::PGPL, BaseLicense::GPL3),
        "PAPL" => (PalimpsestVariant::PAPL, BaseLicense::Apache2),
        "PBSD" => (PalimpsestVariant::PBSD, BaseLicense::BSD3),
        "PEUPL" => (PalimpsestVariant::PEUPL, BaseLicense::EUPL12),
        _ => return None,
    };

    let version = extract_version(full_id).unwrap_or(Version {
        major: 1,
        minor: 0,
        patch: 0,
    });

    Some(License::Palimpsest(PalimpsestLicense {
        variant,
        version: version.clone(),
        base: base.clone(),
        layer: PalimpsestLayer::full(version),
        fallback: base,
        or_later,
    }))
}

/// Extract a semantic version from an SPDX identifier string.
///
/// Looks for patterns like "PMPL-2.0" or "PAGPL-1.0" and extracts the
/// major.minor version numbers.
fn extract_version(id: &str) -> Option<Version> {
    // Find the first segment that starts with a digit after a dash.
    let parts: Vec<&str> = id.split('-').collect();
    for part in &parts {
        if let Some(first_char) = part.chars().next() {
            if first_char.is_ascii_digit() {
                let version_parts: Vec<&str> = part.split('.').collect();
                let major = version_parts.first()?.parse().ok()?;
                let minor = version_parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                let patch = version_parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
                return Some(Version {
                    major,
                    minor,
                    patch,
                });
            }
        }
    }
    None
}

/// Parse a Creative Commons SPDX identifier into a [`BaseLicense::CreativeCommons`].
///
/// Handles identifiers like "CC-BY-SA-4.0", "CC-BY-NC-ND-4.0", "CC0-1.0".
fn parse_cc_identifier(id: &str) -> BaseLicense {
    use crate::family::CcVariant;

    let upper = id.to_uppercase();
    let parts: Vec<&str> = upper.split('-').collect();

    let attribution = parts.contains(&"BY");
    let share_alike = parts.contains(&"SA");
    let non_commercial = parts.contains(&"NC");
    let no_derivatives = parts.contains(&"ND");

    // Extract version (last part that looks like a version number).
    let version = parts
        .iter()
        .rev()
        .find(|p| p.contains('.') || p.chars().all(|c| c.is_ascii_digit()))
        .map(|v| v.to_string())
        .unwrap_or_else(|| "4.0".to_string());

    BaseLicense::CreativeCommons(CcVariant {
        attribution,
        share_alike,
        non_commercial,
        no_derivatives,
        version,
    })
}

/// All known palimpsest variant SPDX identifiers.
///
/// This list is used by the audit system to recognise palimpsest licenses
/// in SPDX headers.
pub const PALIMPSEST_IDENTIFIERS: &[&str] = &[
    "PMPL-1.0",
    "PMPL-1.0-or-later",
    "PMPL-2.0",
    "PMPL-2.0-or-later",
    "PAGPL-1.0",
    "PAGPL-1.0-or-later",
    "PGPL-1.0",
    "PGPL-1.0-or-later",
    "PAPL-1.0",
    "PAPL-1.0-or-later",
    "PBSD-1.0",
    "PBSD-1.0-or-later",
    "PEUPL-1.0",
    "PEUPL-1.0-or-later",
];

/// All well-known SPDX identifiers recognised by the catalog.
pub const KNOWN_SPDX_IDENTIFIERS: &[&str] = &[
    "MIT",
    "Apache-2.0",
    "GPL-2.0",
    "GPL-2.0-only",
    "GPL-3.0",
    "GPL-3.0-only",
    "LGPL-2.1",
    "LGPL-2.1-only",
    "LGPL-3.0",
    "LGPL-3.0-only",
    "AGPL-3.0",
    "AGPL-3.0-only",
    "MPL-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "EUPL-1.2",
    "Artistic-2.0",
    "CERN-OHL-S-2.0",
    "Unlicense",
    "0BSD",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_mit() {
        let license = resolve_identifier("MIT", false).unwrap();
        assert!(matches!(license, License::Base(BaseLicense::MIT)));
    }

    #[test]
    fn test_resolve_pmpl() {
        let license = resolve_identifier("PMPL-2.0", true).unwrap();
        if let License::Palimpsest(p) = license {
            assert_eq!(p.variant, PalimpsestVariant::PMPL);
            assert!(p.or_later);
            assert_eq!(p.base, BaseLicense::MPL2);
        } else {
            panic!("expected palimpsest license");
        }
    }

    #[test]
    fn test_resolve_pagpl() {
        let license = resolve_identifier("PAGPL-1.0", false).unwrap();
        if let License::Palimpsest(p) = license {
            assert_eq!(p.variant, PalimpsestVariant::PAGPL);
            assert_eq!(p.base, BaseLicense::AGPL3);
        } else {
            panic!("expected palimpsest license");
        }
    }

    #[test]
    fn test_resolve_unknown() {
        let license = resolve_identifier("SomeFutureLicense-4.0", false).unwrap();
        assert!(matches!(license, License::Base(BaseLicense::Other(_))));
    }

    #[test]
    fn test_resolve_cc() {
        let license = resolve_identifier("CC-BY-SA-4.0", false).unwrap();
        if let License::Base(BaseLicense::CreativeCommons(cc)) = license {
            assert!(cc.attribution);
            assert!(cc.share_alike);
            assert!(!cc.non_commercial);
        } else {
            panic!("expected CC license");
        }
    }
}
