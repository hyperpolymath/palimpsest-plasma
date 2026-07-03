// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Repository scanning — walks a repository tree and builds a RepoAudit
// against a LicenseMap. This is the bridge between the zone/header
// machinery and CLI consumers.

use crate::audit::header::{extract_spdx_raw_from_content, is_auditable};
use crate::audit::{AuditStatus, AuditSummary, FileAudit, RepoAudit};
use crate::spdx::parse_spdx_expr;
use crate::zone::boundary::{assign_file_to_zone, expected_license_for_file};
use crate::zone::LicenseMap;
use std::fs;
use std::path::Path;

/// Directory names skipped during the walk (mirrors the fact-collection
/// skip list documented in docs/engine-v0-design.adoc).
fn skip_entry(name: &str) -> bool {
    name.starts_with('.')
        || name == "target"
        || name == "node_modules"
        || name == "vendor"
        || name == "_build"
        || name == "deps"
}

/// Walk the repository and audit every auditable file against the license
/// map. Files iterate in sorted order, so results are deterministic.
pub fn scan_repo(root: &Path, license_map: &LicenseMap) -> RepoAudit {
    let mut files: Vec<FileAudit> = Vec::new();

    let mut paths: Vec<_> = walkdir::WalkDir::new(root)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|e| e.depth() == 0 || !skip_entry(e.file_name().to_str().unwrap_or("")))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();
    paths.sort();

    for path in paths {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !is_auditable(ext) {
            continue;
        }

        let rel = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
        let zone = assign_file_to_zone(&rel, license_map);
        let expected = expected_license_for_file(&rel, license_map);
        let proprietary_zone = zone
            .as_deref()
            .and_then(|name| license_map.zones.iter().find(|z| z.name == name))
            .map(|z| z.proprietary)
            .unwrap_or(false);

        let (actual_header, status) = match fs::read_to_string(&path) {
            Err(_) => (None, AuditStatus::Unreadable),
            Ok(content) => match extract_spdx_raw_from_content(&content) {
                None => (None, AuditStatus::MissingHeader),
                Some(raw) => match parse_spdx_expr(&raw) {
                    Err(_) => (None, AuditStatus::WrongLicense),
                    Ok(actual) => {
                        let status = if actual == expected {
                            AuditStatus::Compliant
                        } else if proprietary_zone {
                            AuditStatus::ProprietaryViolation
                        } else {
                            AuditStatus::WrongLicense
                        };
                        (Some(actual), status)
                    }
                },
            },
        };

        files.push(FileAudit {
            path: rel,
            zone,
            expected_license: expected,
            actual_header,
            status,
        });
    }

    let summary = summarize(&files);

    RepoAudit {
        root: root.to_path_buf(),
        license_map: license_map.clone(),
        files,
        family_errors: Vec::new(),
        zone_overlaps: Vec::new(),
        compatibility_warnings: Vec::new(),
        summary,
    }
}

fn summarize(files: &[FileAudit]) -> AuditSummary {
    let mut summary = AuditSummary {
        total_files: files.len() as u32,
        compliant: 0,
        wrong_license: 0,
        missing_header: 0,
        proprietary_violation: 0,
        open_source_violation: 0,
        unreadable: 0,
        score: 1.0,
    };

    for file in files {
        match file.status {
            AuditStatus::Compliant => summary.compliant += 1,
            AuditStatus::WrongLicense => summary.wrong_license += 1,
            AuditStatus::MissingHeader => summary.missing_header += 1,
            AuditStatus::ProprietaryViolation => summary.proprietary_violation += 1,
            AuditStatus::OpenSourceViolation => summary.open_source_violation += 1,
            AuditStatus::Unreadable => summary.unreadable += 1,
        }
    }

    if summary.total_files > 0 {
        summary.score = f64::from(summary.compliant) / f64::from(summary.total_files);
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map_with_default(root: &Path, expr: &str) -> LicenseMap {
        LicenseMap {
            root: root.to_path_buf(),
            default_license: parse_spdx_expr(expr).unwrap(),
            zones: Vec::new(),
        }
    }

    #[test]
    fn test_scan_counts_and_score() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("good.rs"),
            "// SPDX-License-Identifier: MPL-2.0\nfn main() {}\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("wrong.rs"),
            "// SPDX-License-Identifier: MIT\nfn main() {}\n",
        )
        .unwrap();
        fs::write(dir.path().join("missing.rs"), "fn main() {}\n").unwrap();
        fs::write(dir.path().join("notes.md"), "# not auditable\n").unwrap();

        let map = map_with_default(dir.path(), "MPL-2.0");
        let audit = scan_repo(dir.path(), &map);

        assert_eq!(audit.summary.total_files, 3);
        assert_eq!(audit.summary.compliant, 1);
        assert_eq!(audit.summary.wrong_license, 1);
        assert_eq!(audit.summary.missing_header, 1);
        assert!((audit.summary.score - 1.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_scan_is_deterministic() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["b.rs", "a.rs", "c.rs"] {
            fs::write(
                dir.path().join(name),
                "// SPDX-License-Identifier: MPL-2.0\n",
            )
            .unwrap();
        }
        let map = map_with_default(dir.path(), "MPL-2.0");
        let first = scan_repo(dir.path(), &map);
        let second = scan_repo(dir.path(), &map);
        let paths: Vec<_> = first.files.iter().map(|f| f.path.clone()).collect();
        assert_eq!(
            paths,
            second
                .files
                .iter()
                .map(|f| f.path.clone())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            paths,
            vec!["a.rs".into(), "b.rs".into(), "c.rs".into()] as Vec<std::path::PathBuf>
        );
    }
}
