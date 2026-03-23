// SPDX-License-Identifier: PMPL-2.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// Palimpsest family registry — maps all palimpsest variant names to their
// base licenses and creates fully-populated PalimpsestLicense structs.
//
// Each variant in the palimpsest family extends a different well-known base
// license while sharing the same palimpsest provisions layer.

use crate::family::{
    BaseLicense, PalimpsestLayer, PalimpsestLicense, PalimpsestVariant, Version,
};

/// A registry entry describing one palimpsest variant.
///
/// Contains all the static metadata needed to construct a
/// [`PalimpsestLicense`] for this variant.
#[derive(Debug, Clone)]
pub struct RegistryEntry {
    /// The variant enum value.
    pub variant: PalimpsestVariant,
    /// Human-readable full name.
    pub full_name: &'static str,
    /// Short description of the variant.
    pub description: &'static str,
    /// The base license this variant extends.
    pub base: BaseLicense,
    /// Whether this variant has been published (vs. proposed).
    pub published: bool,
}

/// All known palimpsest family variants and their base license mappings.
///
/// This is the canonical source of truth for the palimpsest family. When
/// a new variant is proposed or published, add it here.
pub const REGISTRY: &[RegistryEntry] = &[
    RegistryEntry {
        variant: PalimpsestVariant::PMPL,
        full_name: "Palimpsest Mozilla Public License",
        description: "Palimpsest layer on MPL-2.0 — file-level copyleft with provenance",
        base: BaseLicense::MPL2,
        published: true,
    },
    RegistryEntry {
        variant: PalimpsestVariant::PAGPL,
        full_name: "Palimpsest Affero General Public License",
        description: "Palimpsest layer on AGPL-3.0 — strongest copyleft with network clause",
        base: BaseLicense::AGPL3,
        published: false,
    },
    RegistryEntry {
        variant: PalimpsestVariant::PGPL,
        full_name: "Palimpsest General Public License",
        description: "Palimpsest layer on GPL-3.0 — standard copyleft with provenance",
        base: BaseLicense::GPL3,
        published: false,
    },
    RegistryEntry {
        variant: PalimpsestVariant::PAPL,
        full_name: "Palimpsest Apache License",
        description: "Palimpsest layer on Apache-2.0 — permissive with patent grant and provenance",
        base: BaseLicense::Apache2,
        published: false,
    },
    RegistryEntry {
        variant: PalimpsestVariant::PBSD,
        full_name: "Palimpsest BSD License",
        description: "Palimpsest layer on BSD-3-Clause — minimal copyleft with provenance",
        base: BaseLicense::BSD3,
        published: false,
    },
    RegistryEntry {
        variant: PalimpsestVariant::PEUPL,
        full_name: "Palimpsest European Union Public License",
        description: "Palimpsest layer on EUPL-1.2 — EU-compatible copyleft with provenance",
        base: BaseLicense::EUPL12,
        published: false,
    },
];

/// Look up a registry entry by variant.
///
/// Returns `None` if the variant is `PalimpsestVariant::Other(_)` or
/// otherwise not in the static registry.
pub fn lookup_variant(variant: &PalimpsestVariant) -> Option<&'static RegistryEntry> {
    REGISTRY.iter().find(|e| e.variant == *variant)
}

/// Create a [`PalimpsestLicense`] from a variant and version.
///
/// Uses the registry to determine the correct base and fallback licenses,
/// and enables all palimpsest layer provisions by default.
///
/// # Arguments
///
/// * `variant` — Which palimpsest variant to create.
/// * `version` — The version of the palimpsest license.
/// * `or_later` — Whether the "or later" suffix applies.
///
/// # Returns
///
/// A fully-populated `PalimpsestLicense`, or `None` if the variant is not
/// in the registry.
pub fn create_license(
    variant: &PalimpsestVariant,
    version: Version,
    or_later: bool,
) -> Option<PalimpsestLicense> {
    let entry = lookup_variant(variant)?;
    Some(PalimpsestLicense {
        variant: entry.variant.clone(),
        version: version.clone(),
        base: entry.base.clone(),
        layer: PalimpsestLayer::full(version),
        fallback: entry.base.clone(),
        or_later,
    })
}

/// Return all known palimpsest variants as a vector of registry entries.
///
/// Useful for CLI display (`plasma family`) and report generation.
pub fn all_variants() -> Vec<&'static RegistryEntry> {
    REGISTRY.iter().collect()
}

/// Return only the published (non-proposed) variants.
pub fn published_variants() -> Vec<&'static RegistryEntry> {
    REGISTRY.iter().filter(|e| e.published).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_pmpl() {
        let entry = lookup_variant(&PalimpsestVariant::PMPL).unwrap();
        assert_eq!(entry.base, BaseLicense::MPL2);
        assert!(entry.published);
    }

    #[test]
    fn test_lookup_pagpl() {
        let entry = lookup_variant(&PalimpsestVariant::PAGPL).unwrap();
        assert_eq!(entry.base, BaseLicense::AGPL3);
        assert!(!entry.published);
    }

    #[test]
    fn test_create_license() {
        let version = Version {
            major: 2,
            minor: 0,
            patch: 0,
        };
        let license = create_license(&PalimpsestVariant::PMPL, version, true).unwrap();
        assert_eq!(license.base, BaseLicense::MPL2);
        assert_eq!(license.fallback, BaseLicense::MPL2);
        assert!(license.or_later);
        assert!(license.layer.is_complete());
    }

    #[test]
    fn test_all_variants_count() {
        assert_eq!(all_variants().len(), 6);
    }

    #[test]
    fn test_published_count() {
        assert_eq!(published_variants().len(), 1);
    }

    #[test]
    fn test_lookup_other_returns_none() {
        assert!(lookup_variant(&PalimpsestVariant::Other("PFOO".to_string())).is_none());
    }
}
