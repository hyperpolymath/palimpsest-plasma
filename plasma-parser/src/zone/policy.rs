// SPDX-License-Identifier: PMPL-2.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// Zone policy validation — checks that zone definitions are consistent
// and non-overlapping.

use crate::zone::LicenseZone;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Errors in zone policy configuration.
#[derive(Debug, Clone)]
pub enum ZonePolicyError {
    /// A file matches multiple zones.
    Overlap { file: PathBuf, zones: Vec<String> },
    /// A zone has no matching files in the repository.
    EmptyZone { zone_name: String },
    /// Duplicate zone names.
    DuplicateName { name: String },
}

/// Determine which zone a file belongs to.
///
/// Returns the first matching zone name, or None if the file falls under
/// the default license.
pub fn assign_zone<'a>(file: &Path, root: &Path, zones: &'a [LicenseZone]) -> Option<&'a str> {
    let relative = file.strip_prefix(root).unwrap_or(file);
    let rel_str = relative.to_string_lossy();

    for zone in zones {
        for pattern in &zone.paths {
            if let Ok(glob_pattern) = glob::Pattern::new(pattern) {
                if glob_pattern.matches(&rel_str) {
                    return Some(&zone.name);
                }
            }
        }
    }
    None
}

/// Find all files that match multiple zones (overlap detection).
pub fn detect_overlaps(
    files: &[PathBuf],
    root: &Path,
    zones: &[LicenseZone],
) -> Vec<(PathBuf, Vec<String>)> {
    let mut overlaps = Vec::new();

    for file in files {
        let relative = file.strip_prefix(root).unwrap_or(file);
        let rel_str = relative.to_string_lossy();
        let mut matching_zones = Vec::new();

        for zone in zones {
            for pattern in &zone.paths {
                if let Ok(glob_pattern) = glob::Pattern::new(pattern) {
                    if glob_pattern.matches(&rel_str) {
                        matching_zones.push(zone.name.clone());
                        break;
                    }
                }
            }
        }

        if matching_zones.len() > 1 {
            overlaps.push((file.clone(), matching_zones));
        }
    }

    overlaps
}

/// Validate zone definitions for policy consistency.
pub fn validate_zones(zones: &[LicenseZone]) -> Vec<ZonePolicyError> {
    let mut errors = Vec::new();
    let mut seen: HashMap<&str, usize> = HashMap::new();

    for zone in zones {
        let count = seen.entry(&zone.name).or_insert(0);
        *count += 1;
        if *count == 2 {
            errors.push(ZonePolicyError::DuplicateName {
                name: zone.name.clone(),
            });
        }
    }

    errors
}
