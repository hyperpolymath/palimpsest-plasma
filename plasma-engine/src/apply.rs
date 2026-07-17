// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Apply — execute a remediation Plan against the filesystem.
//
// This is the IO boundary for the action planner (the counterpart to
// `facts` for collection). `plan` decides what to do purely; `apply` does
// it. Applying is idempotent where it can be: an action whose effect is
// already present is skipped, not repeated.

use crate::action::{Action, Plan};
use plasma_parser::audit::header::{comment_prefix, extract_spdx_raw_from_content};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Options controlling how a plan is applied.
#[derive(Debug, Clone)]
pub struct ApplyOptions {
    /// Write a `<file>.bak` copy before modifying an existing file.
    pub backup: bool,
}

impl Default for ApplyOptions {
    fn default() -> Self {
        ApplyOptions { backup: true }
    }
}

/// The result of applying a plan.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApplyOutcome {
    /// Files successfully modified (relative paths as given in the plan).
    pub applied: Vec<String>,
    /// Actions skipped because they were already satisfied, with a reason.
    pub skipped: Vec<(String, String)>,
    /// Actions that failed, with the error message.
    pub errors: Vec<(String, String)>,
}

impl ApplyOutcome {
    pub fn is_clean(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Apply every action in a plan under `root`. Manual items are ignored
/// (they carry no action). Returns a per-action outcome; never panics.
pub fn apply(plan: &Plan, root: &Path, opts: &ApplyOptions) -> ApplyOutcome {
    let mut outcome = ApplyOutcome::default();

    for planned in &plan.actions {
        match &planned.action {
            Action::AddSpdxHeader {
                file,
                license,
                author,
                year,
            } => match add_spdx_header(root, file, license, author, year, opts) {
                Ok(true) => outcome.applied.push(file.clone()),
                Ok(false) => outcome
                    .skipped
                    .push((file.clone(), "already has an SPDX header".to_string())),
                Err(e) => outcome.errors.push((file.clone(), e)),
            },
            Action::CreateFile { path, contents } => match create_file(root, path, contents) {
                Ok(true) => outcome.applied.push(path.clone()),
                Ok(false) => outcome
                    .skipped
                    .push((path.clone(), "file already exists".to_string())),
                Err(e) => outcome.errors.push((path.clone(), e)),
            },
        }
    }

    outcome
}

/// Create `path` with the given contents. Returns Ok(true) when created,
/// Ok(false) when the file already exists (idempotent skip). Never
/// overwrites an existing file, so no backup is needed.
fn create_file(root: &Path, path: &str, contents: &str) -> Result<bool, String> {
    let full = root.join(path);
    if full.exists() {
        return Ok(false);
    }
    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create directories for {path}: {e}"))?;
    }
    fs::write(&full, contents).map_err(|e| format!("cannot create {path}: {e}"))?;
    Ok(true)
}

/// Prepend an SPDX header to `file`. Returns Ok(true) when the file was
/// modified, Ok(false) when it already had a header (idempotent skip).
fn add_spdx_header(
    root: &Path,
    file: &str,
    license: &str,
    author: &str,
    year: &str,
    opts: &ApplyOptions,
) -> Result<bool, String> {
    let path = root.join(file);
    let content = fs::read_to_string(&path).map_err(|e| format!("cannot read {file}: {e}"))?;

    // Idempotent: never double-add a header.
    if extract_spdx_raw_from_content(&content).is_some() {
        return Ok(false);
    }

    let ext = Path::new(file)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let prefix = comment_prefix(ext);
    let header = format!(
        "{prefix} SPDX-License-Identifier: {license}\n{prefix} Copyright (c) {year} {author}\n\n"
    );

    if opts.backup {
        let backup = path.with_extension(match ext {
            "" => "bak".to_string(),
            other => format!("{other}.bak"),
        });
        fs::copy(&path, &backup).map_err(|e| format!("cannot back up {file}: {e}"))?;
    }

    fs::write(&path, format!("{header}{content}"))
        .map_err(|e| format!("cannot write {file}: {e}"))?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{Action, PlannedAction};

    fn header_plan(file: &str) -> Plan {
        Plan {
            actions: vec![PlannedAction {
                rule_id: "headers".to_string(),
                subject: file.to_string(),
                action: Action::AddSpdxHeader {
                    file: file.to_string(),
                    license: "MPL-2.0".to_string(),
                    author: "Tester <t@example.com>".to_string(),
                    year: "2026".to_string(),
                },
            }],
            manual: Vec::new(),
        }
    }

    #[test]
    fn test_apply_adds_header_and_backup() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("plain.rs"), "fn main() {}\n").unwrap();

        let outcome = apply(
            &header_plan("plain.rs"),
            dir.path(),
            &ApplyOptions::default(),
        );
        assert_eq!(outcome.applied, vec!["plain.rs".to_string()]);
        assert!(outcome.is_clean());

        let written = fs::read_to_string(dir.path().join("plain.rs")).unwrap();
        assert!(written.starts_with("// SPDX-License-Identifier: MPL-2.0\n"));
        assert!(written.contains("Copyright (c) 2026 Tester"));
        assert!(written.trim_end().ends_with("fn main() {}"));
        // Backup captured the original.
        let backup = fs::read_to_string(dir.path().join("plain.rs.bak")).unwrap();
        assert_eq!(backup, "fn main() {}\n");
    }

    #[test]
    fn test_apply_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("done.rs"),
            "// SPDX-License-Identifier: MPL-2.0\nfn main() {}\n",
        )
        .unwrap();

        let outcome = apply(
            &header_plan("done.rs"),
            dir.path(),
            &ApplyOptions::default(),
        );
        assert!(outcome.applied.is_empty());
        assert_eq!(outcome.skipped.len(), 1);
        // No backup written for a skipped file.
        assert!(!dir.path().join("done.rs.bak").exists());
    }

    #[test]
    fn test_apply_no_backup_option() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("x.sh"), "echo hi\n").unwrap();

        let outcome = apply(
            &header_plan("x.sh"),
            dir.path(),
            &ApplyOptions { backup: false },
        );
        assert_eq!(outcome.applied, vec!["x.sh".to_string()]);
        assert!(!dir.path().join("x.sh.bak").exists());
        let written = fs::read_to_string(dir.path().join("x.sh")).unwrap();
        assert!(written.starts_with("# SPDX-License-Identifier: MPL-2.0\n"));
    }

    #[test]
    fn test_create_file_action() {
        let dir = tempfile::tempdir().unwrap();
        let plan = Plan {
            actions: vec![PlannedAction {
                rule_id: "sec".to_string(),
                subject: "repo".to_string(),
                action: Action::CreateFile {
                    path: "docs/SECURITY.md".to_string(),
                    contents: "# TODO\n".to_string(),
                },
            }],
            manual: Vec::new(),
        };

        // Creates the file (and its parent dir).
        let outcome = apply(&plan, dir.path(), &ApplyOptions::default());
        assert_eq!(outcome.applied, vec!["docs/SECURITY.md".to_string()]);
        assert_eq!(
            fs::read_to_string(dir.path().join("docs/SECURITY.md")).unwrap(),
            "# TODO\n"
        );

        // Idempotent: a second apply skips the now-existing file and never
        // overwrites it.
        let outcome = apply(&plan, dir.path(), &ApplyOptions::default());
        assert!(outcome.applied.is_empty());
        assert_eq!(outcome.skipped.len(), 1);
    }

    #[test]
    fn test_apply_missing_file_errors() {
        let dir = tempfile::tempdir().unwrap();
        let outcome = apply(
            &header_plan("ghost.rs"),
            dir.path(),
            &ApplyOptions::default(),
        );
        assert!(outcome.applied.is_empty());
        assert_eq!(outcome.errors.len(), 1);
        assert!(!outcome.is_clean());
    }
}
