// SPDX-License-Identifier: PMPL-2.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// License zone module — defines zone types and re-exports the boundary
// parser and policy validator.

pub mod boundary;
pub mod policy;

use crate::spdx::SpdxExpr;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A license zone — a region of the repository under one license.
///
/// Zones partition a repository into areas with different licensing terms.
/// For example, a game might have an AGPL engine zone, a proprietary
/// content zone, and an Apache specs zone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseZone {
    /// Human-readable zone name (e.g., "engine", "game-content", "tools").
    pub name: String,
    /// Glob patterns for directories/files in this zone.
    pub paths: Vec<String>,
    /// The license governing this zone.
    pub license: SpdxExpr,
    /// Whether this zone is proprietary (All Rights Reserved).
    pub proprietary: bool,
    /// Optional copyright holder (primarily for proprietary zones).
    pub copyright: Option<String>,
}

/// A repository's complete license map.
///
/// Combines the repository root, a default license for uncovered files,
/// and zero or more explicit zones with their own licenses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseMap {
    /// The repository root path.
    pub root: PathBuf,
    /// Default license (applies to files not in any explicit zone).
    pub default_license: SpdxExpr,
    /// Explicit zones with their own licenses.
    pub zones: Vec<LicenseZone>,
}
