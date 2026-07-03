// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Fact collection — the only impure module in this crate.
//
// Everything the evaluator may know about a repository flows through the
// FactSet collected here. Collection rules (walk order, skip list, header
// window) are pinned in docs/engine-v0-design.adoc so an independent
// implementation can reproduce identical facts. All collections are
// BTree-ordered so downstream evaluation is deterministic.

use plasma_parser::audit::header::{extract_spdx_raw_from_content, is_auditable};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Errors produced during fact collection.
#[derive(Debug, Error)]
pub enum FactError {
    #[error("path does not exist: {0}")]
    MissingRoot(String),
    #[error("io error while collecting facts: {0}")]
    Io(#[from] std::io::Error),
}

/// Git facts read directly from `.git/HEAD` (no subprocess).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitFacts {
    pub is_repo: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_ref: Option<String>,
}

/// A deterministic snapshot of repository state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactSet {
    /// Repository root as given by the caller.
    pub root: String,
    /// All regular files, relative to root, '/'-separated, sorted.
    pub files: BTreeSet<String>,
    /// Raw SPDX header expression per auditable file (None = no header).
    pub spdx_headers: BTreeMap<String, Option<String>>,
    /// Governance metadata: v0 collects `version` from Cargo.toml when present.
    pub metadata: BTreeMap<String, String>,
    pub git: GitFacts,
}

/// Directory/file names skipped during the walk. Identical to the audit
/// CLI's skip list; pinned in the design doc.
fn skip_entry(name: &str) -> bool {
    name.starts_with('.')
        || name == "target"
        || name == "node_modules"
        || name == "vendor"
        || name == "_build"
        || name == "deps"
}

/// Collect a [`FactSet`] from a repository tree.
pub fn collect(root: &Path) -> Result<FactSet, FactError> {
    if !root.exists() {
        return Err(FactError::MissingRoot(root.display().to_string()));
    }

    let mut files = BTreeSet::new();
    let mut spdx_headers = BTreeMap::new();

    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            // Never skip the root itself, whatever it is named.
            e.depth() == 0 || !skip_entry(e.file_name().to_str().unwrap_or(""))
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let rel = match entry.path().strip_prefix(root) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let rel_str = rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        if rel_str.is_empty() {
            continue;
        }

        let ext = rel.extension().and_then(|e| e.to_str()).unwrap_or("");
        if is_auditable(ext) {
            let header = fs::read_to_string(entry.path())
                .ok()
                .and_then(|content| extract_spdx_raw_from_content(&content));
            spdx_headers.insert(rel_str.clone(), header);
        }

        files.insert(rel_str);
    }

    let mut metadata = BTreeMap::new();
    if let Ok(cargo) = fs::read_to_string(root.join("Cargo.toml")) {
        if let Some(version) = cargo_package_version(&cargo) {
            metadata.insert("version".to_string(), version);
        }
    }

    Ok(FactSet {
        root: root.display().to_string(),
        files,
        spdx_headers,
        metadata,
        git: read_git_facts(root),
    })
}

/// Extract `[package] version = "..."` from Cargo.toml content.
fn cargo_package_version(content: &str) -> Option<String> {
    let value: toml::Value = content.parse().ok()?;
    value
        .get("package")?
        .get("version")?
        .as_str()
        .map(|s| s.to_string())
}

/// Read `.git/HEAD` directly; no subprocess, no clock.
fn read_git_facts(root: &Path) -> GitFacts {
    let head_path = root.join(".git").join("HEAD");
    match fs::read_to_string(&head_path) {
        Ok(content) => {
            let head = content.trim();
            let head_ref = head
                .strip_prefix("ref: ")
                .map(|r| r.to_string())
                .or_else(|| Some(head.to_string()))
                .filter(|s| !s.is_empty());
            GitFacts {
                is_repo: true,
                head_ref,
            }
        }
        Err(_) => GitFacts::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_basic() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("README.md"), "# hi\n").unwrap();
        fs::write(
            dir.path().join("main.rs"),
            "// SPDX-License-Identifier: MPL-2.0\nfn main() {}\n",
        )
        .unwrap();
        fs::write(dir.path().join("plain.rs"), "fn main() {}\n").unwrap();
        fs::create_dir(dir.path().join("target")).unwrap();
        fs::write(dir.path().join("target").join("skip.rs"), "").unwrap();

        let facts = collect(dir.path()).unwrap();
        assert!(facts.files.contains("README.md"));
        assert!(facts.files.contains("main.rs"));
        assert!(!facts.files.contains("target/skip.rs"));
        assert_eq!(
            facts.spdx_headers.get("main.rs"),
            Some(&Some("MPL-2.0".to_string()))
        );
        assert_eq!(facts.spdx_headers.get("plain.rs"), Some(&None));
        // README.md is not an auditable extension.
        assert!(!facts.spdx_headers.contains_key("README.md"));
        assert!(!facts.git.is_repo);
    }

    #[test]
    fn test_cargo_version_metadata() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"x\"\nversion = \"1.2.3\"\n",
        )
        .unwrap();
        let facts = collect(dir.path()).unwrap();
        assert_eq!(facts.metadata.get("version"), Some(&"1.2.3".to_string()));
    }

    #[test]
    fn test_missing_root() {
        assert!(collect(Path::new("/nonexistent/nowhere")).is_err());
    }
}
