// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Facts diff — the before/after contract for agent verification.
//
// `diff` is pure (like `eval` and `plan`): two FactSets in, one FactsDiff
// out, deterministically ordered. Claim-verification tooling
// (did-you-actually-do-that) snapshots facts before an agent runs, again
// after, and diffs the two: the result is exactly what changed in the
// repository as far as policy evaluation is concerned — no more, no less.

use crate::facts::{FactSet, GitFacts};
use serde::{Deserialize, Serialize};

/// A field whose value changed between two snapshots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Changed<T> {
    pub before: T,
    pub after: T,
}

/// A per-file SPDX header transition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeaderChange {
    pub file: String,
    /// None = file had no header (or was absent) in that snapshot.
    pub before: Option<String>,
    pub after: Option<String>,
}

/// A governance metadata transition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetadataChange {
    pub key: String,
    /// None = key absent in that snapshot.
    pub before: Option<String>,
    pub after: Option<String>,
}

/// The deterministic difference between two fact snapshots.
///
/// All lists are sorted (they derive from BTree iteration), so two
/// identical snapshot pairs always produce byte-identical JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactsDiff {
    pub root_before: String,
    pub root_after: String,
    /// Files present after but not before.
    pub files_added: Vec<String>,
    /// Files present before but not after.
    pub files_removed: Vec<String>,
    /// SPDX header transitions on auditable files present in both
    /// snapshots (added/removed files are already covered above).
    pub headers_changed: Vec<HeaderChange>,
    /// Governance metadata transitions (added, removed, or changed keys).
    pub metadata_changed: Vec<MetadataChange>,
    /// Git state transition, when it changed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_changed: Option<Changed<GitFacts>>,
}

impl FactsDiff {
    /// True when the two snapshots describe identical repository state
    /// (roots may differ — a snapshot is comparable across checkouts).
    pub fn is_empty(&self) -> bool {
        self.files_added.is_empty()
            && self.files_removed.is_empty()
            && self.headers_changed.is_empty()
            && self.metadata_changed.is_empty()
            && self.git_changed.is_none()
    }

    /// Total number of recorded changes.
    pub fn change_count(&self) -> usize {
        self.files_added.len()
            + self.files_removed.len()
            + self.headers_changed.len()
            + self.metadata_changed.len()
            + usize::from(self.git_changed.is_some())
    }
}

/// Compute the difference between two fact snapshots. Pure: no IO.
pub fn diff(before: &FactSet, after: &FactSet) -> FactsDiff {
    let files_added: Vec<String> = after.files.difference(&before.files).cloned().collect();
    let files_removed: Vec<String> = before.files.difference(&after.files).cloned().collect();

    // Header transitions on files common to both snapshots. Headers of
    // added/removed files are implied by the file lists.
    let mut headers_changed = Vec::new();
    for (file, before_header) in &before.spdx_headers {
        if !after.files.contains(file) {
            continue;
        }
        let after_header = after.spdx_headers.get(file).cloned().flatten();
        if before_header != &after_header {
            headers_changed.push(HeaderChange {
                file: file.clone(),
                before: before_header.clone(),
                after: after_header,
            });
        }
    }
    // A file that became auditable only in the after snapshot (e.g. gained
    // an extension mapping) but existed before: surface its header too.
    for (file, after_header) in &after.spdx_headers {
        if before.files.contains(file)
            && !before.spdx_headers.contains_key(file)
            && after_header.is_some()
        {
            headers_changed.push(HeaderChange {
                file: file.clone(),
                before: None,
                after: after_header.clone(),
            });
        }
    }
    headers_changed.sort_by(|a, b| a.file.cmp(&b.file));

    // Metadata: union of keys, record any difference.
    let mut metadata_changed = Vec::new();
    let keys: std::collections::BTreeSet<&String> = before
        .metadata
        .keys()
        .chain(after.metadata.keys())
        .collect();
    for key in keys {
        let b = before.metadata.get(key).cloned();
        let a = after.metadata.get(key).cloned();
        if b != a {
            metadata_changed.push(MetadataChange {
                key: key.clone(),
                before: b,
                after: a,
            });
        }
    }

    let git_changed = if before.git != after.git {
        Some(Changed {
            before: before.git.clone(),
            after: after.git.clone(),
        })
    } else {
        None
    };

    FactsDiff {
        root_before: before.root.clone(),
        root_after: after.root.clone(),
        files_added,
        files_removed,
        headers_changed,
        metadata_changed,
        git_changed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    fn facts(
        files: &[&str],
        headers: &[(&str, Option<&str>)],
        metadata: &[(&str, &str)],
    ) -> FactSet {
        FactSet {
            root: "r".to_string(),
            files: files.iter().map(|s| s.to_string()).collect::<BTreeSet<_>>(),
            spdx_headers: headers
                .iter()
                .map(|(k, v)| (k.to_string(), v.map(|s| s.to_string())))
                .collect::<BTreeMap<_, _>>(),
            metadata: metadata
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<BTreeMap<_, _>>(),
            git: Default::default(),
        }
    }

    #[test]
    fn test_identical_snapshots_empty_diff() {
        let a = facts(
            &["x.rs"],
            &[("x.rs", Some("MPL-2.0"))],
            &[("version", "1.0")],
        );
        let d = diff(&a, &a.clone());
        assert!(d.is_empty());
        assert_eq!(d.change_count(), 0);
    }

    #[test]
    fn test_files_added_and_removed() {
        let before = facts(&["a.rs", "gone.rs"], &[], &[]);
        let after = facts(&["a.rs", "new.rs"], &[], &[]);
        let d = diff(&before, &after);
        assert_eq!(d.files_added, vec!["new.rs".to_string()]);
        assert_eq!(d.files_removed, vec!["gone.rs".to_string()]);
    }

    #[test]
    fn test_header_transition() {
        let before = facts(&["a.rs"], &[("a.rs", None)], &[]);
        let after = facts(&["a.rs"], &[("a.rs", Some("MPL-2.0"))], &[]);
        let d = diff(&before, &after);
        assert_eq!(
            d.headers_changed,
            vec![HeaderChange {
                file: "a.rs".to_string(),
                before: None,
                after: Some("MPL-2.0".to_string()),
            }]
        );
        // Reverse direction records the removal.
        let d = diff(&after, &before);
        assert_eq!(d.headers_changed[0].before, Some("MPL-2.0".to_string()));
        assert_eq!(d.headers_changed[0].after, None);
    }

    #[test]
    fn test_removed_file_header_not_double_counted() {
        let before = facts(&["a.rs"], &[("a.rs", Some("MPL-2.0"))], &[]);
        let after = facts(&[], &[], &[]);
        let d = diff(&before, &after);
        assert_eq!(d.files_removed, vec!["a.rs".to_string()]);
        assert!(d.headers_changed.is_empty());
    }

    #[test]
    fn test_metadata_added_changed_removed() {
        let before = facts(&[], &[], &[("version", "1.0"), ("old", "x")]);
        let after = facts(&[], &[], &[("version", "2.0"), ("new", "y")]);
        let d = diff(&before, &after);
        assert_eq!(d.metadata_changed.len(), 3);
        // BTree order: new, old, version.
        assert_eq!(d.metadata_changed[0].key, "new");
        assert_eq!(d.metadata_changed[0].before, None);
        assert_eq!(d.metadata_changed[1].key, "old");
        assert_eq!(d.metadata_changed[1].after, None);
        assert_eq!(d.metadata_changed[2].key, "version");
        assert_eq!(d.metadata_changed[2].before, Some("1.0".to_string()));
        assert_eq!(d.metadata_changed[2].after, Some("2.0".to_string()));
    }

    #[test]
    fn test_git_transition() {
        let before = facts(&[], &[], &[]);
        let mut after = facts(&[], &[], &[]);
        after.git = GitFacts {
            is_repo: true,
            head_ref: Some("refs/heads/main".to_string()),
        };
        let d = diff(&before, &after);
        let git = d.git_changed.unwrap();
        assert!(!git.before.is_repo);
        assert!(git.after.is_repo);
    }

    #[test]
    fn test_diff_is_deterministic() {
        let before = facts(&["b.rs", "a.rs"], &[("b.rs", None)], &[("k", "1")]);
        let after = facts(&["c.rs", "a.rs"], &[], &[("k", "2")]);
        let d1 = diff(&before, &after);
        let d2 = diff(&before, &after);
        assert_eq!(
            serde_json::to_string(&d1).unwrap(),
            serde_json::to_string(&d2).unwrap()
        );
    }
}
