// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Action planner — map policy violations to corrective actions.
//
// `plan` is pure (like `eval`): it takes a policy, an evaluation, and a fix
// context, and returns a Plan. Every violation is either turned into a
// mechanical Action or recorded as a ManualItem with a reason — nothing is
// silently dropped. Applying the plan (IO) lives in `apply`.

use crate::ast::{ActionKind, Modality, Policy, Resource, Rule};
use crate::finding::{Evaluation, FindingStatus};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Inputs the planner needs to synthesise fixes.
#[derive(Debug, Clone)]
pub struct FixContext {
    /// SPDX identifier written into newly added headers.
    pub license: String,
    /// Copyright holder line for newly added headers.
    pub author: String,
    /// Year written into newly added headers (injected, never clock-derived,
    /// so planning stays deterministic).
    pub year: String,
}

impl Default for FixContext {
    fn default() -> Self {
        FixContext {
            license: "MPL-2.0".to_string(),
            author: "CHANGE-ME <your@email.com>".to_string(),
            year: "2026".to_string(),
        }
    }
}

/// A concrete corrective action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Action {
    /// Prepend an SPDX header to a source file that lacks one.
    AddSpdxHeader {
        file: String,
        license: String,
        author: String,
        year: String,
    },
    /// Create a required file that is missing, with placeholder content the
    /// user is expected to replace.
    CreateFile { path: String, contents: String },
}

/// True when `path` contains glob metacharacters, so no single filename can
/// be derived from it.
fn is_glob(path: &str) -> bool {
    path.contains(['*', '?', '[', ']', '{', '}'])
}

/// Placeholder content for a newly created required file. Deliberately
/// obvious as a stub so it is not mistaken for finished content.
fn stub_for(path: &str, rule_id: &str) -> String {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext {
        "md" | "adoc" | "markdown" => format!(
            "# TODO\n\nPlaceholder created by `plasma fix` to satisfy policy rule \
             `{rule_id}`. Replace this with real content.\n"
        ),
        _ => format!(
            "Placeholder created by plasma fix to satisfy policy rule {rule_id}. \
             Replace this with real content.\n"
        ),
    }
}

/// An action bound to the finding it resolves.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedAction {
    pub rule_id: String,
    pub subject: String,
    pub action: Action,
}

/// A violation with no mechanical fix, and why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManualItem {
    pub rule_id: String,
    pub subject: String,
    pub reason: String,
}

/// The full remediation plan for an evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    pub actions: Vec<PlannedAction>,
    pub manual: Vec<ManualItem>,
}

impl Plan {
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty() && self.manual.is_empty()
    }
}

/// Build a remediation plan from an evaluation. Pure: no IO.
pub fn plan(policy: &Policy, evaluation: &Evaluation, ctx: &FixContext) -> Plan {
    let rules = rule_index(policy);

    let mut actions = Vec::new();
    let mut manual = Vec::new();

    for finding in &evaluation.findings {
        if finding.status != FindingStatus::Violation {
            continue;
        }
        let Some(rule) = rules.get(finding.rule_id.as_str()) else {
            // A finding whose rule we cannot see (should not happen for a
            // plan built from the same policy) — record it as manual.
            manual.push(ManualItem {
                rule_id: finding.rule_id.clone(),
                subject: finding.subject.clone(),
                reason: "no rule found for finding".to_string(),
            });
            continue;
        };

        match fix_for(rule, &finding.subject, ctx) {
            Some(action) => actions.push(PlannedAction {
                rule_id: finding.rule_id.clone(),
                subject: finding.subject.clone(),
                action,
            }),
            None => manual.push(ManualItem {
                rule_id: finding.rule_id.clone(),
                subject: finding.subject.clone(),
                reason: manual_reason(rule),
            }),
        }
    }

    Plan { actions, manual }
}

/// Decide the mechanical fix for a violated rule at a concrete subject.
fn fix_for(rule: &Rule, subject: &str, ctx: &FixContext) -> Option<Action> {
    match (&rule.modality, &rule.action, &rule.resource) {
        // A file that must carry an SPDX header but does not: add one.
        (Modality::Obligation, ActionKind::Present, Resource::Header) => {
            Some(Action::AddSpdxHeader {
                file: subject.to_string(),
                license: ctx.license.clone(),
                author: ctx.author.clone(),
                year: ctx.year.clone(),
            })
        }
        // A required file (at a concrete path) that is missing: create a
        // placeholder. A glob path names no single file, so it stays manual.
        (Modality::Obligation, ActionKind::Present, Resource::File { path })
        | (Modality::Obligation, ActionKind::Present, Resource::Manifest { path })
            if !is_glob(path) =>
        {
            Some(Action::CreateFile {
                path: path.clone(),
                contents: stub_for(path, &rule.id),
            })
        }
        _ => None,
    }
}

/// Explain why a violated rule has no mechanical fix in this version.
fn manual_reason(rule: &Rule) -> String {
    match (&rule.modality, &rule.action, &rule.resource) {
        // Only globs reach here now — concrete paths become CreateFile.
        (Modality::Obligation, ActionKind::Present, Resource::File { .. })
        | (Modality::Obligation, ActionKind::Present, Resource::Manifest { .. }) => {
            "cannot create a required file from a glob pattern (specify a concrete path)"
                .to_string()
        }
        (Modality::Obligation, ActionKind::Valid, Resource::Header) => {
            "an unparsable SPDX header must be corrected by hand".to_string()
        }
        (Modality::Prohibition, _, _) => {
            "removing a prohibited resource is not auto-applied".to_string()
        }
        _ => "no automatic fix is available for this rule".to_string(),
    }
}

/// Index the effective rule set (base minus overridden, plus overlay-added)
/// by id — the same set the evaluator saw, so a finding never resolves to a
/// rule that was overridden out of evaluation.
fn rule_index(policy: &Policy) -> BTreeMap<&str, &Rule> {
    crate::eval::effective_rules(policy)
        .into_iter()
        .map(|(rule, _origin)| (rule.id.as_str(), rule))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::evaluate;
    use crate::facts::FactSet;
    use crate::schema::{load_policy_str, PolicyFormat};
    use std::collections::{BTreeMap, BTreeSet};

    fn facts(files: &[&str], headers: &[(&str, Option<&str>)]) -> FactSet {
        FactSet {
            root: "root".to_string(),
            files: files.iter().map(|s| s.to_string()).collect::<BTreeSet<_>>(),
            spdx_headers: headers
                .iter()
                .map(|(k, v)| (k.to_string(), v.map(|s| s.to_string())))
                .collect::<BTreeMap<_, _>>(),
            metadata: BTreeMap::new(),
            git: Default::default(),
            file_contents: BTreeMap::new(),
        }
    }

    fn ctx() -> FixContext {
        FixContext {
            license: "MPL-2.0".to_string(),
            author: "Tester <t@example.com>".to_string(),
            year: "2026".to_string(),
        }
    }

    #[test]
    fn test_missing_header_becomes_action() {
        let policy = load_policy_str(
            r#"
schema_version = { major = 0, minor = 1 }
id = "t"
version = { major = 1, minor = 0 }

[[rules]]
id = "headers"
modality = "obligation"
severity = "warning"
subject = { type = "file-pattern", patterns = ["**/*.rs"] }
resource = { type = "header" }
action = { type = "present" }
"#,
            PolicyFormat::Toml,
        )
        .unwrap();

        let facts = facts(
            &["a.rs", "b.rs"],
            &[("a.rs", Some("MPL-2.0")), ("b.rs", None)],
        );
        let eval = evaluate(&policy, &facts);
        let p = plan(&policy, &eval, &ctx());

        assert_eq!(p.actions.len(), 1);
        assert!(p.manual.is_empty());
        assert_eq!(
            p.actions[0].action,
            Action::AddSpdxHeader {
                file: "b.rs".to_string(),
                license: "MPL-2.0".to_string(),
                author: "Tester <t@example.com>".to_string(),
                year: "2026".to_string(),
            }
        );
    }

    #[test]
    fn test_missing_concrete_file_becomes_create_action() {
        let policy = load_policy_str(
            r#"
schema_version = { major = 0, minor = 1 }
id = "t"
version = { major = 1, minor = 0 }

[[rules]]
id = "contributing-present"
modality = "obligation"
subject = { type = "repo" }
resource = { type = "file", path = "CONTRIBUTING.md" }
action = { type = "present" }
"#,
            PolicyFormat::Toml,
        )
        .unwrap();

        let eval = evaluate(&policy, &facts(&["README.md"], &[]));
        let p = plan(&policy, &eval, &ctx());
        assert!(p.manual.is_empty());
        assert_eq!(p.actions.len(), 1);
        match &p.actions[0].action {
            Action::CreateFile { path, contents } => {
                assert_eq!(path, "CONTRIBUTING.md");
                assert!(contents.contains("contributing-present"));
                assert!(contents.starts_with("# TODO"));
            }
            other => panic!("expected CreateFile, got {other:?}"),
        }
    }

    #[test]
    fn test_missing_file_is_manual() {
        let policy = load_policy_str(
            r#"
schema_version = { major = 0, minor = 1 }
id = "t"
version = { major = 1, minor = 0 }

[[rules]]
id = "license-present"
modality = "obligation"
subject = { type = "repo" }
resource = { type = "file", path = "LICENSE*" }
action = { type = "present" }
"#,
            PolicyFormat::Toml,
        )
        .unwrap();

        let eval = evaluate(&policy, &facts(&["README.md"], &[]));
        let p = plan(&policy, &eval, &ctx());

        assert!(p.actions.is_empty());
        assert_eq!(p.manual.len(), 1);
        assert_eq!(p.manual[0].subject, "repo");
    }

    #[test]
    fn test_clean_eval_empty_plan() {
        let policy = crate::schema::builtin_repo_hygiene();
        let facts = facts(
            &["LICENSE", "README.adoc", "a.rs"],
            &[("a.rs", Some("MPL-2.0"))],
        );
        let eval = evaluate(&policy, &facts);
        let p = plan(&policy, &eval, &ctx());
        assert!(p.is_empty());
    }
}
