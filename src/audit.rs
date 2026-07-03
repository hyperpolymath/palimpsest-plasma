// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// plasma audit — check all source files for correct SPDX headers, using
// the plasma-parser SPDX machinery (real expression parsing, zone-aware
// expectations from .plasma.toml when present).

use anyhow::{Context, Result};
use plasma_parser::audit::header::comment_prefix;
use plasma_parser::audit::{scan_repo, AuditStatus};
use plasma_parser::spdx::parse_spdx_expr;
use plasma_parser::zone::boundary::parse_plasma_toml;
use plasma_parser::zone::LicenseMap;
use std::fs;
use std::path::Path;

/// Run the audit: walk all files, check SPDX headers against the license map.
pub fn run(path: &str, license: &str, fix: bool) -> Result<()> {
    let root = Path::new(path);
    if !root.exists() {
        anyhow::bail!("Path does not exist: {}", path);
    }

    // Zone-aware expectations from .plasma.toml when present; otherwise a
    // flat map with the --license expression as the default.
    let license_map = match parse_plasma_toml(root) {
        Ok(map) => map,
        Err(_) => LicenseMap {
            root: root.to_path_buf(),
            default_license: parse_spdx_expr(license)
                .map_err(|e| anyhow::anyhow!("invalid --license expression {license:?}: {e}"))?,
            zones: Vec::new(),
        },
    };

    let audit = scan_repo(root, &license_map);

    let mut fixed = 0u32;
    for file in &audit.files {
        match file.status {
            AuditStatus::Compliant | AuditStatus::Unreadable => {}
            AuditStatus::MissingHeader if fix => {
                let full_path = root.join(&file.path);
                let ext = file.path.extension().and_then(|e| e.to_str()).unwrap_or("");
                let prefix = comment_prefix(ext);
                let header = format!(
                    "{prefix} SPDX-License-Identifier: {}\n{prefix} Copyright (c) 2026 CHANGE-ME\n\n",
                    file.expected_license
                );
                let content = fs::read_to_string(&full_path)
                    .with_context(|| format!("cannot read {}", full_path.display()))?;
                fs::write(&full_path, format!("{header}{content}"))?;
                println!("  FIXED: {}", file.path.display());
                fixed += 1;
            }
            AuditStatus::MissingHeader => {
                println!("  MISSING: {}", file.path.display());
            }
            AuditStatus::WrongLicense => {
                println!(
                    "  WRONG LICENSE: {} (expected {}, found {})",
                    file.path.display(),
                    file.expected_license,
                    file.actual_header
                        .as_ref()
                        .map(|h| h.to_string())
                        .unwrap_or_else(|| "an unparsable header".to_string())
                );
            }
            AuditStatus::ProprietaryViolation => {
                println!(
                    "  PROPRIETARY ZONE VIOLATION: {} carries {}",
                    file.path.display(),
                    file.actual_header
                        .as_ref()
                        .map(|h| h.to_string())
                        .unwrap_or_default()
                );
            }
            AuditStatus::OpenSourceViolation => {
                println!("  OPEN-SOURCE ZONE VIOLATION: {}", file.path.display());
            }
        }
    }

    let summary = &audit.summary;
    let noncompliant = summary.wrong_license
        + summary.missing_header
        + summary.proprietary_violation
        + summary.open_source_violation;

    println!();
    println!("  Audit results:");
    println!("    Total files:   {}", summary.total_files);
    println!("    Compliant:     {}", summary.compliant);
    println!("    Missing/Wrong: {noncompliant}");
    if fix {
        println!("    Fixed:         {fixed}");
    }
    if summary.unreadable > 0 {
        println!(
            "    Skipped:       {} (binary/unreadable)",
            summary.unreadable
        );
    }
    println!("    Score:         {:.0}%", summary.score * 100.0);

    if noncompliant > 0 && !fix {
        println!();
        println!("  Run `plasma audit --fix` to add missing headers automatically.");
        // Exit with error code so CI fails.
        std::process::exit(1);
    }

    Ok(())
}
