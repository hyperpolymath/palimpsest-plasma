// SPDX-License-Identifier: PMPL-2.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// plasma audit — check all source files for correct SPDX headers.

use anyhow::Result;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// File extensions that should have SPDX headers.
const AUDITABLE_EXTENSIONS: &[&str] = &[
    "rs", "zig", "idr", "js", "ts", "jsx", "tsx", "res", "resi", "ex", "exs", "gleam", "jl",
    "hs", "ml", "mli", "nim", "pony", "py", "rb", "go", "c", "h", "cpp", "hpp", "java", "kt",
    "scala", "sh", "bash", "zsh", "toml", "yaml", "yml", "ncl", "nix", "scm",
];

/// Comment prefixes for SPDX headers by file type.
fn comment_prefix(ext: &str) -> &'static str {
    match ext {
        "rs" | "zig" | "js" | "ts" | "jsx" | "tsx" | "res" | "resi" | "go" | "c" | "h"
        | "cpp" | "hpp" | "java" | "kt" | "scala" | "pony" => "//",
        "idr" | "hs" | "ml" | "mli" => "--",
        "ex" | "exs" | "gleam" | "nim" | "py" | "rb" | "sh" | "bash" | "zsh" | "toml"
        | "yaml" | "yml" | "ncl" | "nix" | "jl" => "#",
        "scm" => ";;",
        _ => "//",
    }
}

/// Run the audit: walk all files, check for SPDX headers.
pub fn run(path: &str, fix: bool) -> Result<()> {
    let root = Path::new(path);
    let mut total = 0u32;
    let mut compliant = 0u32;
    let mut missing = 0u32;
    let mut fixed = 0u32;
    let mut skipped = 0u32;

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_str().unwrap_or("");
            !name.starts_with('.')
                && name != "target"
                && name != "node_modules"
                && name != "vendor"
                && name != "_build"
                && name != "deps"
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        if !AUDITABLE_EXTENSIONS.contains(&ext) {
            continue;
        }

        total += 1;

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => {
                skipped += 1;
                continue;
            }
        };

        if content.contains("SPDX-License-Identifier") {
            // Check it says PMPL
            if content.contains("PMPL") {
                compliant += 1;
            } else {
                // Has SPDX but wrong license
                println!(
                    "  WRONG LICENSE: {} (has SPDX but not PMPL)",
                    path.display()
                );
                missing += 1;
            }
        } else {
            // No SPDX header at all
            if fix {
                let prefix = comment_prefix(ext);
                let header = format!(
                    "{prefix} SPDX-License-Identifier: PMPL-2.0-or-later\n{prefix} Copyright (c) 2026 CHANGE-ME\n\n"
                );
                let updated = format!("{header}{content}");
                fs::write(path, updated)?;
                println!("  FIXED: {}", path.display());
                fixed += 1;
            } else {
                println!("  MISSING: {}", path.display());
                missing += 1;
            }
        }
    }

    println!();
    println!("  Audit results:");
    println!("    Total files:  {total}");
    println!("    Compliant:    {compliant}");
    println!("    Missing/Wrong: {missing}");
    if fix {
        println!("    Fixed:        {fixed}");
    }
    if skipped > 0 {
        println!("    Skipped:      {skipped} (binary/unreadable)");
    }

    if missing > 0 && !fix {
        println!();
        println!("  Run `plasma audit --fix` to add missing headers automatically.");
    }

    if missing > 0 && !fix {
        // Exit with error code so CI fails
        std::process::exit(1);
    }

    Ok(())
}
