// SPDX-License-Identifier: PMPL-2.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// License family type system — defines the core types for the palimpsest
// license family: base licenses, palimpsest layers, variants, and the
// composite PalimpsestLicense type.

pub mod fallback;
pub mod registry;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// A semantic version with major.minor.patch components.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.patch == 0 {
            write!(f, "{}.{}", self.major, self.minor)
        } else {
            write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
        }
    }
}

/// A base license that exists independently in the legal landscape.
///
/// These are the well-known licenses with court precedent and established
/// community understanding. A palimpsest license always extends one of these.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BaseLicense {
    MIT,
    Apache2,
    GPL2,
    GPL3,
    LGPL21,
    LGPL3,
    AGPL3,
    MPL2,
    BSD2,
    BSD3,
    ISC,
    EUPL12,
    Artistic2,
    CERNOHLS2,
    Unlicense,
    ZeroBSD,
    /// Proprietary / All Rights Reserved.
    Proprietary,
    /// Creative Commons variant.
    CreativeCommons(CcVariant),
    /// Unknown base license with raw SPDX identifier.
    Other(String),
}

impl std::fmt::Display for BaseLicense {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BaseLicense::MIT => write!(f, "MIT"),
            BaseLicense::Apache2 => write!(f, "Apache-2.0"),
            BaseLicense::GPL2 => write!(f, "GPL-2.0"),
            BaseLicense::GPL3 => write!(f, "GPL-3.0"),
            BaseLicense::LGPL21 => write!(f, "LGPL-2.1"),
            BaseLicense::LGPL3 => write!(f, "LGPL-3.0"),
            BaseLicense::AGPL3 => write!(f, "AGPL-3.0"),
            BaseLicense::MPL2 => write!(f, "MPL-2.0"),
            BaseLicense::BSD2 => write!(f, "BSD-2-Clause"),
            BaseLicense::BSD3 => write!(f, "BSD-3-Clause"),
            BaseLicense::ISC => write!(f, "ISC"),
            BaseLicense::EUPL12 => write!(f, "EUPL-1.2"),
            BaseLicense::Artistic2 => write!(f, "Artistic-2.0"),
            BaseLicense::CERNOHLS2 => write!(f, "CERN-OHL-S-2.0"),
            BaseLicense::Unlicense => write!(f, "Unlicense"),
            BaseLicense::ZeroBSD => write!(f, "0BSD"),
            BaseLicense::Proprietary => write!(f, "LicenseRef-Proprietary"),
            BaseLicense::CreativeCommons(cc) => write!(f, "{cc}"),
            BaseLicense::Other(id) => write!(f, "{id}"),
        }
    }
}

/// Creative Commons sub-variants, capturing the modular CC license elements.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CcVariant {
    /// BY — Attribution required.
    pub attribution: bool,
    /// SA — Share Alike (derivative works must use same license).
    pub share_alike: bool,
    /// NC — Non-Commercial use only.
    pub non_commercial: bool,
    /// ND — No Derivatives allowed.
    pub no_derivatives: bool,
    /// Version string, e.g. "4.0".
    pub version: String,
}

impl std::fmt::Display for CcVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CC")?;
        if self.attribution {
            write!(f, "-BY")?;
        }
        if self.non_commercial {
            write!(f, "-NC")?;
        }
        if self.no_derivatives {
            write!(f, "-ND")?;
        }
        if self.share_alike {
            write!(f, "-SA")?;
        }
        write!(f, "-{}", self.version)
    }
}

/// The palimpsest layer — the provisions added on top of a base license.
///
/// All palimpsest variants share the same set of provisions; only the base
/// license differs. Each provision maps to a numbered article in the
/// palimpsest license text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PalimpsestLayer {
    /// The palimpsest version these provisions belong to.
    pub version: Version,
    /// P1: Emotional lineage — acknowledgement of creative/emotional investment.
    pub emotional_lineage: bool,
    /// P2: AI notice — disclosure when AI tools contributed to the work.
    pub ai_notice: bool,
    /// P3: Ethical use — restrictions on harmful applications.
    pub ethical_use: bool,
    /// P4: Provenance metadata — structured origin and chain-of-custody data.
    pub provenance_metadata: bool,
    /// P5: Quantum-safe — post-quantum cryptography for provenance chains.
    pub quantum_safe: bool,
    /// P6: Lineage chain — verifiable history of modifications.
    pub lineage_chain: bool,
    /// P7: Governance — community governance integration.
    pub governance: bool,
}

impl PalimpsestLayer {
    /// Create a full palimpsest layer with all provisions enabled.
    ///
    /// This is the default for new palimpsest licenses — all seven provisions
    /// are active unless explicitly disabled in `.plasma.toml`.
    pub fn full(version: Version) -> Self {
        Self {
            version,
            emotional_lineage: true,
            ai_notice: true,
            ethical_use: true,
            provenance_metadata: true,
            quantum_safe: true,
            lineage_chain: true,
            governance: true,
        }
    }

    /// Check whether all provisions are enabled.
    pub fn is_complete(&self) -> bool {
        self.emotional_lineage
            && self.ai_notice
            && self.ethical_use
            && self.provenance_metadata
            && self.quantum_safe
            && self.lineage_chain
            && self.governance
    }
}

/// A complete palimpsest license = base + layer + fallback.
///
/// The fallback must always match the base license — this is enforced by
/// the family validation stage. If the palimpsest provisions are ever
/// struck down legally, the code reverts to the base/fallback license.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PalimpsestLicense {
    /// The palimpsest variant identifier (e.g., PMPL, PAGPL, PGPL).
    pub variant: PalimpsestVariant,
    /// The version of this palimpsest license.
    pub version: Version,
    /// The base license this palimpsest extends.
    pub base: BaseLicense,
    /// The palimpsest-specific provisions.
    pub layer: PalimpsestLayer,
    /// The license to fall back to if palimpsest provisions are struck.
    /// Must be identical to `base` (enforced by validation).
    pub fallback: BaseLicense,
    /// Whether "or later" applies.
    pub or_later: bool,
}

impl std::fmt::Display for PalimpsestLicense {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.variant, self.version)?;
        if self.or_later {
            write!(f, "-or-later")?;
        }
        Ok(())
    }
}

/// Known palimpsest variants.
///
/// Each variant maps to a specific base license. The variant name is the
/// SPDX-style prefix used in headers (e.g., "PMPL" in "PMPL-2.0-or-later").
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PalimpsestVariant {
    /// Palimpsest-MPL (base: MPL-2.0). Published.
    PMPL,
    /// Palimpsest-AGPL (base: AGPL-3.0). Proposed.
    PAGPL,
    /// Palimpsest-GPL (base: GPL-3.0). Proposed.
    PGPL,
    /// Palimpsest-Apache (base: Apache-2.0). Proposed.
    PAPL,
    /// Palimpsest-BSD (base: BSD-3-Clause). Proposed.
    PBSD,
    /// Palimpsest-EUPL (base: EUPL-1.2). Proposed.
    PEUPL,
    /// Other/future variant.
    Other(String),
}

impl std::fmt::Display for PalimpsestVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PalimpsestVariant::PMPL => write!(f, "PMPL"),
            PalimpsestVariant::PAGPL => write!(f, "PAGPL"),
            PalimpsestVariant::PGPL => write!(f, "PGPL"),
            PalimpsestVariant::PAPL => write!(f, "PAPL"),
            PalimpsestVariant::PBSD => write!(f, "PBSD"),
            PalimpsestVariant::PEUPL => write!(f, "PEUPL"),
            PalimpsestVariant::Other(name) => write!(f, "{name}"),
        }
    }
}

/// Any license — either a plain base license or a palimpsest variant.
///
/// This is the top-level license type used throughout the parser. Every
/// SPDX identifier resolves to one of these.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum License {
    /// A well-known base license (MIT, Apache-2.0, GPL-3.0, etc.).
    Base(BaseLicense),
    /// A palimpsest family license (PMPL, PAGPL, PGPL, etc.).
    Palimpsest(PalimpsestLicense),
}

impl std::fmt::Display for License {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            License::Base(base) => write!(f, "{base}"),
            License::Palimpsest(p) => write!(f, "{p}"),
        }
    }
}

/// Validation errors specific to palimpsest licenses.
///
/// These errors indicate structural problems with a palimpsest license
/// declaration — not SPDX parse errors, but semantic inconsistencies.
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum FamilyError {
    /// Fallback license does not match the declared base.
    #[error("fallback mismatch for {variant}: base is {declared_base}, fallback is {declared_fallback}")]
    FallbackMismatch {
        variant: PalimpsestVariant,
        declared_base: BaseLicense,
        declared_fallback: BaseLicense,
    },
    /// Base license text not found in the LICENSE file.
    #[error("base license text missing for {variant}: expected {expected_base}")]
    BaseLicenseTextMissing {
        variant: PalimpsestVariant,
        expected_base: BaseLicense,
    },
    /// Palimpsest layer claims a provision (e.g., quantum-safe) but the
    /// required exhibit is missing from the license file.
    #[error("exhibit '{exhibit}' missing for {variant}")]
    ExhibitMissing {
        exhibit: char,
        variant: PalimpsestVariant,
    },
    /// SPDX header in a file says one license but the LICENSE file says another.
    #[error("header/license mismatch in {}: header says {header_says}, LICENSE says {license_file_says}", file.display())]
    HeaderLicenseMismatch {
        file: PathBuf,
        header_says: License,
        license_file_says: License,
    },
}
