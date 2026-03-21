// SPDX-License-Identifier: PMPL-2.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// plasma migrate — convert from MIT/Apache/GPL to PMPL-2.0.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Known license identifiers we can migrate from.
const KNOWN_LICENSES: &[&str] = &[
    "MIT",
    "Apache-2.0",
    "GPL-2.0",
    "GPL-3.0",
    "LGPL-2.1",
    "LGPL-3.0",
    "AGPL-3.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "MPL-2.0",
    "Unlicense",
    "0BSD",
];

/// Run the migration: detect current license, replace with PMPL-2.0.
pub fn run(path: &str, from: Option<&str>) -> Result<()> {
    let root = Path::new(path);

    // Detect current license
    let current = if let Some(f) = from {
        f.to_string()
    } else {
        detect_license(root)?
    };

    println!("  Migrating from {} → PMPL-2.0-or-later", current);
    println!();

    // Replace LICENSE file
    let license_path = root.join("LICENSE");
    if license_path.exists() {
        // Back up old license
        let backup = root.join(format!("LICENSE.{}.bak", current.replace('/', "-")));
        fs::copy(&license_path, &backup)
            .context("Failed to back up LICENSE")?;
        println!("  BACKED UP: LICENSE → {}", backup.display());
    }

    // Write new license
    let pmpl_text = include_str!("../LICENSE-PMPL-2.0.txt");
    fs::write(&license_path, pmpl_text)
        .context("Failed to write LICENSE")?;
    println!("  REPLACED: LICENSE (PMPL-2.0)");

    // Update SPDX headers in source files
    let old_spdx = format!("SPDX-License-Identifier: {current}");
    let new_spdx = "SPDX-License-Identifier: PMPL-2.0-or-later".to_string();
    let mut updated_count = 0u32;

    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_str().unwrap_or("");
            !name.starts_with('.') && name != "target" && name != "node_modules"
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let file_path = entry.path();
        if let Ok(content) = fs::read_to_string(file_path) {
            if content.contains(&old_spdx) {
                let updated = content.replace(&old_spdx, &new_spdx);
                fs::write(file_path, updated)?;
                updated_count += 1;
            }
        }
    }

    if updated_count > 0 {
        println!(
            "  UPDATED: {} file(s) — SPDX headers changed to PMPL-2.0-or-later",
            updated_count
        );
    }

    // Update Cargo.toml license field
    let cargo_path = root.join("Cargo.toml");
    if cargo_path.exists() {
        let content = fs::read_to_string(&cargo_path)?;
        let old_field = format!("license = \"{current}\"");
        if content.contains(&old_field) {
            let updated = content.replace(&old_field, "license = \"PMPL-2.0-or-later\"");
            fs::write(&cargo_path, updated)?;
            println!("  UPDATED: Cargo.toml license field");
        }
    }

    // Update package.json license field
    let pkg_path = root.join("package.json");
    if pkg_path.exists() {
        let content = fs::read_to_string(&pkg_path)?;
        let old_field = format!("\"license\": \"{current}\"");
        if content.contains(&old_field) {
            let updated = content.replace(&old_field, "\"license\": \"PMPL-2.0-or-later\"");
            fs::write(&pkg_path, updated)?;
            println!("  UPDATED: package.json license field");
        }
    }

    println!();
    println!("  Migration complete. Run `plasma audit` to verify all headers.");

    Ok(())
}

/// Detect the current license by reading the LICENSE file.
fn detect_license(root: &Path) -> Result<String> {
    let license_path = root.join("LICENSE");
    if !license_path.exists() {
        // Try LICENSE.md, LICENSE.txt
        for name in &["LICENSE.md", "LICENSE.txt", "LICENCE", "LICENCE.md"] {
            let p = root.join(name);
            if p.exists() {
                return detect_from_content(&fs::read_to_string(p)?);
            }
        }
        anyhow::bail!(
            "No LICENSE file found. Use --from to specify the current license."
        );
    }

    let content = fs::read_to_string(&license_path)?;
    detect_from_content(&content)
}

/// Detect license from file content by looking for known patterns.
fn detect_from_content(content: &str) -> Result<String> {
    let lower = content.to_lowercase();

    if lower.contains("mit license") || lower.contains("permission is hereby granted, free of charge") {
        return Ok("MIT".to_string());
    }
    if lower.contains("apache license") && lower.contains("version 2.0") {
        return Ok("Apache-2.0".to_string());
    }
    if lower.contains("gnu general public license") && lower.contains("version 3") {
        return Ok("GPL-3.0".to_string());
    }
    if lower.contains("gnu general public license") && lower.contains("version 2") {
        return Ok("GPL-2.0".to_string());
    }
    if lower.contains("gnu lesser general public license") {
        return Ok("LGPL-2.1".to_string());
    }
    if lower.contains("gnu affero general public license") {
        return Ok("AGPL-3.0".to_string());
    }
    if lower.contains("mozilla public license") && lower.contains("2.0") {
        return Ok("MPL-2.0".to_string());
    }
    if lower.contains("bsd 2-clause") || (lower.contains("redistribution") && !lower.contains("3.")) {
        return Ok("BSD-2-Clause".to_string());
    }
    if lower.contains("bsd 3-clause") {
        return Ok("BSD-3-Clause".to_string());
    }
    if lower.contains("isc license") {
        return Ok("ISC".to_string());
    }
    if lower.contains("unlicense") || lower.contains("this is free and unencumbered") {
        return Ok("Unlicense".to_string());
    }
    if lower.contains("palimpsest") {
        return Ok("PMPL-1.0".to_string());
    }

    anyhow::bail!(
        "Could not auto-detect license. Use --from to specify. Known licenses: {}",
        KNOWN_LICENSES.join(", ")
    )
}
