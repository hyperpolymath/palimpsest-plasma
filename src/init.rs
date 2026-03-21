// SPDX-License-Identifier: PMPL-2.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// plasma init — add PMPL-2.0 to any project in one command.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// The full PMPL-2.0 license text (embedded for offline use).
const PMPL_2_0_TEXT: &str = include_str!("../LICENSE-PMPL-2.0.txt");

/// The Palimpsest Community Covenant text.
const COVENANT_TEXT: &str = include_str!("../PALIMPSEST-COVENANT.md");

/// Run the init command: add LICENSE, SPDX headers, and optional Covenant.
pub fn run(path: &str, author: Option<&str>, covenant: bool) -> Result<()> {
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
    } else {
        fs::write(&license_path, PMPL_2_0_TEXT)
            .context("Failed to write LICENSE")?;
        println!("  CREATED: LICENSE (PMPL-2.0)");
    }

    // 2. Write PALIMPSEST-COVENANT.md if requested
    if covenant {
        let covenant_path = root.join("PALIMPSEST-COVENANT.md");
        if covenant_path.exists() {
            println!("  SKIP: PALIMPSEST-COVENANT.md already exists");
        } else {
            fs::write(&covenant_path, COVENANT_TEXT)
                .context("Failed to write PALIMPSEST-COVENANT.md")?;
            println!("  CREATED: PALIMPSEST-COVENANT.md");
        }
    }

    // 3. Add SPDX header to common config files if they exist
    let spdx_header = format!(
        "# SPDX-License-Identifier: PMPL-2.0-or-later\n# SPDX-FileCopyrightText: {} {}\n",
        year, author_name
    );

    // Check for Cargo.toml and update license field
    let cargo_path = root.join("Cargo.toml");
    if cargo_path.exists() {
        let content = fs::read_to_string(&cargo_path)?;
        if !content.contains("PMPL") && !content.contains("SPDX-License-Identifier") {
            // Add SPDX header at top
            let updated = format!("{spdx_header}{content}");
            fs::write(&cargo_path, updated)?;
            println!("  UPDATED: Cargo.toml (added SPDX header)");
        }

        // Update license field if present
        if content.contains("license = \"MIT\"")
            || content.contains("license = \"Apache-2.0\"")
            || content.contains("license = \"\"")
        {
            let updated = content
                .replace("license = \"MIT\"", "license = \"PMPL-2.0-or-later\"")
                .replace(
                    "license = \"Apache-2.0\"",
                    "license = \"PMPL-2.0-or-later\"",
                )
                .replace("license = \"\"", "license = \"PMPL-2.0-or-later\"");
            fs::write(&cargo_path, updated)?;
            println!("  UPDATED: Cargo.toml license field → PMPL-2.0-or-later");
        }
    }

    // Check for package.json and update license field
    let pkg_path = root.join("package.json");
    if pkg_path.exists() {
        let content = fs::read_to_string(&pkg_path)?;
        if content.contains("\"license\": \"MIT\"")
            || content.contains("\"license\": \"Apache-2.0\"")
            || content.contains("\"license\": \"ISC\"")
        {
            let updated = content
                .replace("\"license\": \"MIT\"", "\"license\": \"PMPL-2.0-or-later\"")
                .replace(
                    "\"license\": \"Apache-2.0\"",
                    "\"license\": \"PMPL-2.0-or-later\"",
                )
                .replace(
                    "\"license\": \"ISC\"",
                    "\"license\": \"PMPL-2.0-or-later\"",
                );
            fs::write(&pkg_path, updated)?;
            println!("  UPDATED: package.json license → PMPL-2.0-or-later");
        }
    }

    // Check for mix.exs
    let mix_path = root.join("mix.exs");
    if mix_path.exists() {
        let content = fs::read_to_string(&mix_path)?;
        if !content.contains("PMPL") {
            println!("  NOTE: mix.exs found — manually update licenses field to [\"PMPL-2.0-or-later\"]");
        }
    }

    println!();
    println!("  Palimpsest-MPL 2.0 applied to {}", path);
    println!("  Next steps:");
    println!("    1. Run `plasma audit` to check SPDX headers on source files");
    println!("    2. Add badge to README: `plasma badge`");
    if covenant {
        println!("    3. Reference the Covenant in your CONTRIBUTING.md");
    }

    Ok(())
}
