// SPDX-License-Identifier: PMPL-2.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// Zone boundary parser — reads .plasma.toml and builds a LicenseMap that
// assigns files to license zones based on glob patterns.

use crate::spdx::{parse_spdx_expr, SpdxExpr};
use crate::zone::{LicenseMap, LicenseZone};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors produced when parsing .plasma.toml zone configuration.
#[derive(Debug, Error)]
pub enum ZoneParseError {
    #[error("failed to read .plasma.toml: {0}")]
    IoError(#[from] std::io::Error),
    #[error("failed to parse .plasma.toml: {0}")]
    TomlError(#[from] toml::de::Error),
    #[error("missing [project] section in .plasma.toml")]
    MissingProject,
    #[error("missing default-license in [project]")]
    MissingDefaultLicense,
    #[error("invalid SPDX expression in zone '{zone}': {source}")]
    InvalidSpdx {
        zone: String,
        source: crate::spdx::SpdxParseError,
    },
}

/// Raw TOML structure for .plasma.toml (deserialization target).
#[derive(Debug, serde::Deserialize)]
struct PlasmaToml {
    project: Option<ProjectSection>,
    zone: Option<Vec<ZoneSection>>,
    #[allow(dead_code)]
    family: Option<FamilySection>,
}

/// The [project] section of .plasma.toml.
#[derive(Debug, serde::Deserialize)]
struct ProjectSection {
    #[allow(dead_code)]
    name: Option<String>,
    #[serde(rename = "default-license")]
    default_license: Option<String>,
}

/// A [[zone]] entry in .plasma.toml.
#[derive(Debug, serde::Deserialize)]
struct ZoneSection {
    name: String,
    license: String,
    paths: Vec<String>,
    #[serde(default)]
    proprietary: bool,
    copyright: Option<String>,
}

/// The optional [family] section for palimpsest provision overrides.
#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct FamilySection {
    #[serde(rename = "emotional-lineage")]
    emotional_lineage: Option<bool>,
    #[serde(rename = "ai-notice")]
    ai_notice: Option<bool>,
    #[serde(rename = "ethical-use")]
    ethical_use: Option<bool>,
    #[serde(rename = "provenance-metadata")]
    provenance_metadata: Option<bool>,
    #[serde(rename = "quantum-safe")]
    quantum_safe: Option<bool>,
    #[serde(rename = "lineage-chain")]
    lineage_chain: Option<bool>,
    governance: Option<bool>,
}

/// Parse a `.plasma.toml` file and build a [`LicenseMap`].
///
/// Looks for `.plasma.toml` in the repository root first, then falls back
/// to `.machine_readable/6a2/.plasma.toml`.
///
/// # Arguments
///
/// * `repo_root` — Path to the repository root directory.
///
/// # Returns
///
/// A populated `LicenseMap` with default license and all declared zones,
/// or an error if the configuration is missing or invalid.
pub fn parse_plasma_toml(repo_root: &Path) -> Result<LicenseMap, ZoneParseError> {
    let toml_path = find_plasma_toml(repo_root)?;
    let content = std::fs::read_to_string(&toml_path)?;
    let raw: PlasmaToml = toml::from_str(&content)?;

    let project = raw.project.ok_or(ZoneParseError::MissingProject)?;
    let default_license_str = project
        .default_license
        .ok_or(ZoneParseError::MissingDefaultLicense)?;

    let default_license =
        parse_spdx_expr(&default_license_str).map_err(|e| ZoneParseError::InvalidSpdx {
            zone: "default".to_string(),
            source: e,
        })?;

    let mut zones = Vec::new();
    if let Some(zone_sections) = raw.zone {
        for section in zone_sections {
            let license =
                parse_spdx_expr(&section.license).map_err(|e| ZoneParseError::InvalidSpdx {
                    zone: section.name.clone(),
                    source: e,
                })?;

            zones.push(LicenseZone {
                name: section.name,
                paths: section.paths,
                license,
                proprietary: section.proprietary,
                copyright: section.copyright,
            });
        }
    }

    Ok(LicenseMap {
        root: repo_root.to_path_buf(),
        default_license,
        zones,
    })
}

/// Locate the .plasma.toml file within a repository.
///
/// Checks the repository root first, then the .machine_readable/6a2/
/// subdirectory (the standard location for machine-readable artefacts).
fn find_plasma_toml(repo_root: &Path) -> Result<PathBuf, std::io::Error> {
    let root_path = repo_root.join(".plasma.toml");
    if root_path.exists() {
        return Ok(root_path);
    }

    let mr_path = repo_root.join(".machine_readable/6a2/.plasma.toml");
    if mr_path.exists() {
        return Ok(mr_path);
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!(
            ".plasma.toml not found in {} or .machine_readable/6a2/",
            repo_root.display()
        ),
    ))
}

/// Determine which zone a file belongs to, given a license map.
///
/// Returns the zone name if the file matches any zone's glob patterns,
/// or `None` if the file falls under the default license.
///
/// # Arguments
///
/// * `file_path` — Absolute or relative path to the file.
/// * `license_map` — The repository's license map.
pub fn assign_file_to_zone(file_path: &Path, license_map: &LicenseMap) -> Option<String> {
    // Make the path relative to the repository root for glob matching.
    let relative = file_path
        .strip_prefix(&license_map.root)
        .unwrap_or(file_path);
    let relative_str = relative.to_string_lossy();

    for zone in &license_map.zones {
        for pattern in &zone.paths {
            if let Ok(glob_pattern) = glob::Pattern::new(pattern) {
                if glob_pattern.matches(&relative_str) {
                    return Some(zone.name.clone());
                }
            }
        }
    }

    None
}

/// Get the expected license for a file based on its zone assignment.
///
/// If the file is in an explicit zone, returns that zone's license.
/// Otherwise, returns the default license.
pub fn expected_license_for_file(file_path: &Path, license_map: &LicenseMap) -> SpdxExpr {
    if let Some(zone_name) = assign_file_to_zone(file_path, license_map) {
        for zone in &license_map.zones {
            if zone.name == zone_name {
                return zone.license.clone();
            }
        }
    }
    license_map.default_license.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_plasma_toml_missing() {
        let result = find_plasma_toml(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }
}
