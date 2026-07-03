// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// plasma init — add a license and SPDX headers to a project in one command.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// The full MPL-2.0 license text (embedded for offline use).
const MPL_2_0_TEXT: &str = include_str!("../LICENSES/MPL-2.0.txt");

/// Run the init command: add LICENSE and SPDX metadata for the chosen license.
pub fn run(path: &str, license: &str, author: Option<&str>) -> Result<()> {
    let root = Path::new(path);

    if !root.exists() {
        anyhow::bail!("Path does not exist: {}", path);
    }

    let author_name = author.unwrap_or("CHANGE-ME <your@email.com>");
    let year = "2026"; // TODO: get current year dynamically

    // 1. Write LICENSE file
    let license_path = root.join("LICENSE");
    if license_path.exists() {
        println!("  SKIP: LICENSE already exists (use `plasma migrate` to convert)");
    } else if license == "MPL-2.0" {
        fs::write(&license_path, MPL_2_0_TEXT).context("Failed to write LICENSE")?;
        println!("  CREATED: LICENSE (MPL-2.0)");
    } else {
        fs::write(
            &license_path,
            format!(
                "SPDX-License-Identifier: {license}\n\nTODO: replace this stub with the full {license} license text.\n"
            ),
        )
        .context("Failed to write LICENSE stub")?;
        println!("  CREATED: LICENSE stub for {license} — add the full license text manually");
    }

    // 2. Add SPDX header to common config files if they exist
    let spdx_header = format!(
        "# SPDX-License-Identifier: {license}\n# SPDX-FileCopyrightText: {year} {author_name}\n"
    );

    // Check for Cargo.toml and update license field
    let cargo_path = root.join("Cargo.toml");
    if cargo_path.exists() {
        let content = fs::read_to_string(&cargo_path)?;
        if !content.contains("SPDX-License-Identifier") {
            let updated = format!("{spdx_header}{content}");
            fs::write(&cargo_path, updated)?;
            println!("  UPDATED: Cargo.toml (added SPDX header)");
        }

        // Update license field if empty or a common default
        let content = fs::read_to_string(&cargo_path)?;
        if content.contains("license = \"\"") {
            let updated = content.replace("license = \"\"", &format!("license = \"{license}\""));
            fs::write(&cargo_path, updated)?;
            println!("  UPDATED: Cargo.toml license field → {license}");
        }
    }

    // Check for package.json and note the license field
    let pkg_path = root.join("package.json");
    if pkg_path.exists() {
        let content = fs::read_to_string(&pkg_path)?;
        if !content.contains(&format!("\"license\": \"{license}\"")) {
            println!("  NOTE: package.json found — set \"license\": \"{license}\" manually");
        }
    }

    // Check for mix.exs
    let mix_path = root.join("mix.exs");
    if mix_path.exists() {
        let content = fs::read_to_string(&mix_path)?;
        if !content.contains(license) {
            println!("  NOTE: mix.exs found — manually update licenses field to [\"{license}\"]");
        }
    }

    println!();
    println!("  {license} applied to {path}");
    println!("  Next steps:");
    println!("    1. Run `plasma audit` to check SPDX headers on source files");
    println!("    2. Add badge to README: `plasma badge`");

    Ok(())
}
