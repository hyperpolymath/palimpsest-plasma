// SPDX-License-Identifier: PMPL-2.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// Audit module — SPDX header extraction, zone-aware file auditing,
// and LICENSE file content analysis with approximate matching.

pub mod content;
pub mod header;

use crate::family::FamilyError;
use crate::spdx::SpdxExpr;
use crate::zone::LicenseMap;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Audit result for a single file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAudit {
    /// Path to the audited file (relative to repo root).
    pub path: PathBuf,
    /// Zone this file belongs to (None if using default license).
    pub zone: Option<String>,
    /// The license expected based on zone assignment.
    pub expected_license: SpdxExpr,
    /// The SPDX expression actually found in the file header.
    pub actual_header: Option<SpdxExpr>,
    /// Compliance status.
    pub status: AuditStatus,
}

/// Audit status for a file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditStatus {
    /// SPDX header matches zone license.
    Compliant,
    /// SPDX header present but wrong license.
    WrongLicense,
    /// No SPDX header found.
    MissingHeader,
    /// File is in a proprietary zone but has an open-source header.
    ProprietaryViolation,
    /// File is in an open-source zone but has a proprietary header.
    OpenSourceViolation,
    /// File could not be read (binary, permissions, encoding).
    Unreadable,
}

/// Full repository audit result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoAudit {
    /// Repository root path.
    pub root: PathBuf,
    /// The license map used for this audit.
    pub license_map: LicenseMap,
    /// Per-file audit results.
    pub files: Vec<FileAudit>,
    /// Family-level validation errors.
    pub family_errors: Vec<FamilyError>,
    /// Files that appear in multiple zones (zone overlap).
    pub zone_overlaps: Vec<(PathBuf, Vec<String>)>,
    /// Compatibility warnings between zones.
    pub compatibility_warnings: Vec<crate::compat::CompatWarning>,
    /// Summary statistics.
    pub summary: AuditSummary,
}

/// Summary statistics for a repository audit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSummary {
    pub total_files: u32,
    pub compliant: u32,
    pub wrong_license: u32,
    pub missing_header: u32,
    pub proprietary_violation: u32,
    pub open_source_violation: u32,
    pub unreadable: u32,
    /// Compliance score (0.0 to 1.0).
    pub score: f64,
}
