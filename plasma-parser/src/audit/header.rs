// SPDX-License-Identifier: PMPL-2.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// SPDX header extraction — reads the first N lines of a source file and
// extracts the SPDX-License-Identifier value.

use crate::spdx::{parse_spdx_expr, SpdxExpr};
use std::fs;
use std::path::Path;

/// Maximum number of lines to scan for an SPDX header.
const MAX_HEADER_LINES: usize = 15;

/// File extensions that should have SPDX headers, grouped by comment style.
pub const AUDITABLE_EXTENSIONS: &[&str] = &[
    "rs", "zig", "idr", "js", "ts", "jsx", "tsx", "res", "resi", "ex", "exs",
    "gleam", "jl", "hs", "ml", "mli", "nim", "pony", "py", "rb", "go", "c",
    "h", "cpp", "hpp", "java", "kt", "scala", "sh", "bash", "zsh", "toml",
    "yaml", "yml", "ncl", "nix", "scm", "v", "ada", "adb", "ads",
];

/// Returns the comment prefix for a given file extension.
pub fn comment_prefix(ext: &str) -> &'static str {
    match ext {
        "rs" | "zig" | "js" | "ts" | "jsx" | "tsx" | "res" | "resi" | "go"
        | "c" | "h" | "cpp" | "hpp" | "java" | "kt" | "scala" | "pony"
        | "v" => "//",
        "idr" | "hs" | "ml" | "mli" => "--",
        "ada" | "adb" | "ads" => "--",
        "ex" | "exs" | "gleam" | "nim" | "py" | "rb" | "sh" | "bash"
        | "zsh" | "toml" | "yaml" | "yml" | "ncl" | "nix" | "jl" => "#",
        "scm" => ";;",
        _ => "//",
    }
}

/// Check whether a file extension is auditable.
pub fn is_auditable(ext: &str) -> bool {
    AUDITABLE_EXTENSIONS.contains(&ext)
}

/// Extract the SPDX-License-Identifier from a file's header.
///
/// Returns `None` if the file has no SPDX header within the first
/// `MAX_HEADER_LINES` lines.
pub fn extract_spdx_header(path: &Path) -> Option<SpdxExpr> {
    let content = fs::read_to_string(path).ok()?;
    extract_spdx_from_content(&content)
}

/// Extract SPDX expression from file content string.
pub fn extract_spdx_from_content(content: &str) -> Option<SpdxExpr> {
    for line in content.lines().take(MAX_HEADER_LINES) {
        let trimmed = line.trim();
        // Strip comment prefixes.
        let stripped = trimmed
            .strip_prefix("//")
            .or_else(|| trimmed.strip_prefix('#'))
            .or_else(|| trimmed.strip_prefix("--"))
            .or_else(|| trimmed.strip_prefix(";;"))
            .unwrap_or(trimmed)
            .trim();

        if let Some(rest) = stripped.strip_prefix("SPDX-License-Identifier:") {
            let spdx_str = rest.trim();
            return parse_spdx_expr(spdx_str).ok();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_rust_header() {
        let content = "// SPDX-License-Identifier: PMPL-2.0-or-later\n// Copyright (c) 2026\n";
        let expr = extract_spdx_from_content(content);
        assert!(expr.is_some());
    }

    #[test]
    fn test_extract_python_header() {
        let content = "#!/usr/bin/env python3\n# SPDX-License-Identifier: MIT\n";
        let expr = extract_spdx_from_content(content);
        assert!(expr.is_some());
    }

    #[test]
    fn test_no_header() {
        let content = "fn main() {\n    println!(\"hello\");\n}\n";
        let expr = extract_spdx_from_content(content);
        assert!(expr.is_none());
    }
}
