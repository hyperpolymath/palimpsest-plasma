// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Policy evaluation — pure and total.
//
// `evaluate` is a pure function Policy × FactSet → Evaluation: no IO, no
// clocks, no randomness, no environment reads. Every partial case has a
// defined false/absent semantics (documented in docs/engine-v0-design.adoc),
// so any policy that passed schema validation evaluates without panicking.
// Facts iterate in BTree order, so findings arrive in deterministic order.

use crate::ast::{ActionKind, Condition, Modality, OverlayEffect, Policy, Resource, Rule, Subject};
use crate::facts::FactSet;
use crate::finding::{Evaluation, Finding, FindingStatus, Severity, Summary};
use plasma_parser::spdx::parse_spdx_expr;

/// A concrete instance a rule is checked against.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Instance {
    Repo,
    Path(String),
    Metadata(String),
}

impl Instance {
    fn label(&self) -> String {
        match self {
            Instance::Repo => "repo".to_string(),
            Instance::Path(p) => p.clone(),
            Instance::Metadata(k) => format!("metadata:{k}"),
        }
    }
}

/// Evaluate a policy against a fact set.
pub fn evaluate(policy: &Policy, facts: &FactSet) -> Evaluation {
    let mut findings = Vec::new();

    for (rule, origin) in effective_rules(policy) {
        for instance in expand_subject(&rule.subject, facts) {
            if !eval_condition(&rule.condition, &instance, facts) {
                continue; // rule not applicable to this instance
            }
            findings.push(check_rule(rule, origin, &instance, facts));
        }
    }

    let mut summary = Summary::default();
    for finding in &findings {
        match finding.status {
            FindingStatus::Pass => summary.passes += 1,
            FindingStatus::Violation => match finding.severity {
                Severity::Error => summary.errors += 1,
                Severity::Warning => summary.warnings += 1,
                Severity::Info => summary.info += 1,
            },
        }
    }

    Evaluation {
        root: facts.root.clone(),
        policy_id: policy.id.clone(),
        policy_version: policy.version,
        schema_version: policy.schema_version,
        findings,
        summary,
    }
}

/// Base rules (minus any overridden by an overlay) followed by
/// overlay-added rules, each with its origin (None = base policy,
/// Some(id) = overlay). Shared with the action planner so both see the
/// same effective set.
pub(crate) fn effective_rules(policy: &Policy) -> Vec<(&Rule, Option<&str>)> {
    // Ids removed from the base set by an `override-rules` effect.
    let overridden: std::collections::BTreeSet<&str> = policy
        .overlays
        .iter()
        .flat_map(|o| &o.effects)
        .filter_map(|e| match e {
            OverlayEffect::OverrideRules { ids } => Some(ids.iter().map(String::as_str)),
            _ => None,
        })
        .flatten()
        .collect();

    let mut rules: Vec<(&Rule, Option<&str>)> = policy
        .rules
        .iter()
        .filter(|r| !overridden.contains(r.id.as_str()))
        .map(|r| (r, None))
        .collect();
    for overlay in &policy.overlays {
        for effect in &overlay.effects {
            if let OverlayEffect::AddRules { rules: added } = effect {
                rules.extend(added.iter().map(|r| (r, Some(overlay.id.as_str()))));
            }
        }
    }
    rules
}

/// Expand a subject to its concrete instances over the fact set.
fn expand_subject(subject: &Subject, facts: &FactSet) -> Vec<Instance> {
    match subject {
        Subject::Repo => vec![Instance::Repo],
        Subject::File { path } => vec![Instance::Path(path.clone())],
        Subject::FilePattern { patterns } => facts
            .files
            .iter()
            .filter(|file| patterns.iter().any(|p| glob_matches(p, file)))
            .map(|file| Instance::Path(file.clone()))
            .collect(),
        Subject::Metadata { key } => vec![Instance::Metadata(key.clone())],
        // Rejected at load; unreachable for validated policies. Defined
        // as empty to keep evaluation total regardless.
        Subject::Release { .. } => vec![],
    }
}

/// Glob matching over '/'-separated relative paths.
///
/// `**/` prefixes also match zero directories: "**/*.rs" matches both
/// "src/main.rs" and "main.rs".
fn glob_matches(pattern: &str, path: &str) -> bool {
    match glob::Pattern::new(pattern) {
        Ok(p) => {
            if p.matches(path) {
                return true;
            }
            // glob's "**/" requires at least one component; allow zero.
            if let Some(rest) = pattern.strip_prefix("**/") {
                if let Ok(p2) = glob::Pattern::new(rest) {
                    return p2.matches(path);
                }
            }
            false
        }
        Err(_) => false, // malformed pattern matches nothing (documented)
    }
}

/// Total condition semantics over (facts, instance).
fn eval_condition(condition: &Condition, instance: &Instance, facts: &FactSet) -> bool {
    match condition {
        Condition::True => true,
        Condition::Not { of } => !eval_condition(of, instance, facts),
        Condition::All { of } => of.iter().all(|c| eval_condition(c, instance, facts)),
        Condition::Any { of } => of.iter().any(|c| eval_condition(c, instance, facts)),
        Condition::RepoHasFile { path } => facts.files.iter().any(|f| glob_matches(path, f)),
        Condition::HasSpdxHeader => match instance {
            Instance::Path(p) => matches!(facts.spdx_headers.get(p), Some(Some(_))),
            _ => false,
        },
        Condition::SpdxLicenseIs { expr } => match instance {
            Instance::Path(p) => match facts.spdx_headers.get(p) {
                Some(Some(raw)) => spdx_equal(raw, expr),
                _ => false,
            },
            _ => false,
        },
        Condition::GovernanceFlagSet { key, value } => {
            facts.metadata.get(key).map(|v| v == value).unwrap_or(false)
        }
        Condition::VersionAtLeast { version } => facts
            .metadata
            .get("version")
            .and_then(|actual| compare_versions(actual, version))
            .unwrap_or(false),
        // Rejected at load; defined as false to keep evaluation total.
        Condition::FileMatchesPattern { .. } => false,
    }
}

/// Two SPDX expressions are equal when both parse and their parsed forms
/// agree; unparsable expressions compare by exact text.
fn spdx_equal(actual: &str, expected: &str) -> bool {
    match (parse_spdx_expr(actual), parse_spdx_expr(expected)) {
        (Ok(a), Ok(b)) => a == b,
        _ => actual.trim() == expected.trim(),
    }
}

/// Numeric dotted-version comparison: Some(actual >= minimum), None when
/// either side has no leading numeric components.
fn compare_versions(actual: &str, minimum: &str) -> Option<bool> {
    let parse = |s: &str| -> Option<Vec<u64>> {
        let parts: Vec<u64> = s
            .trim()
            .split('.')
            .map_while(|p| {
                let digits: String = p.chars().take_while(|c| c.is_ascii_digit()).collect();
                digits.parse().ok()
            })
            .collect();
        if parts.is_empty() {
            None
        } else {
            Some(parts)
        }
    };
    let a = parse(actual)?;
    let m = parse(minimum)?;
    // Compare component-wise, treating missing components as zero.
    let len = a.len().max(m.len());
    for i in 0..len {
        let av = a.get(i).copied().unwrap_or(0);
        let mv = m.get(i).copied().unwrap_or(0);
        if av != mv {
            return Some(av > mv);
        }
    }
    Some(true)
}

/// Does the resource exist for this instance?
fn resource_exists(resource: &Resource, instance: &Instance, facts: &FactSet) -> bool {
    match resource {
        Resource::File { path } | Resource::Manifest { path } => {
            facts.files.iter().any(|f| glob_matches(path, f))
        }
        Resource::Header => match instance {
            Instance::Path(p) => matches!(facts.spdx_headers.get(p), Some(Some(_))),
            _ => false,
        },
        Resource::GovernanceField { key } => facts.metadata.contains_key(key),
        // Rejected at load; defined as absent to keep evaluation total.
        Resource::Release => false,
    }
}

/// Is the resource structurally valid for this instance? Validity implies
/// existence. For headers, validity means the SPDX expression parses; for
/// everything else in v0, validity coincides with existence.
fn resource_valid(resource: &Resource, instance: &Instance, facts: &FactSet) -> bool {
    match resource {
        Resource::Header => match instance {
            Instance::Path(p) => match facts.spdx_headers.get(p) {
                Some(Some(raw)) => parse_spdx_expr(raw).is_ok(),
                _ => false,
            },
            _ => false,
        },
        _ => resource_exists(resource, instance, facts),
    }
}

fn describe_resource(resource: &Resource) -> String {
    match resource {
        Resource::File { path } => format!("file {path:?}"),
        Resource::Header => "SPDX header".to_string(),
        Resource::Manifest { path } => format!("manifest {path:?}"),
        Resource::GovernanceField { key } => format!("governance field {key:?}"),
        Resource::Release => "release".to_string(),
    }
}

/// Apply the deontic matrix for one (rule, instance) pair.
fn check_rule(rule: &Rule, origin: Option<&str>, instance: &Instance, facts: &FactSet) -> Finding {
    let exists = resource_exists(&rule.resource, instance, facts);
    let resource_desc = describe_resource(&rule.resource);

    let (violated, message) = match (&rule.modality, &rule.action) {
        (Modality::Permission, _) => (
            false,
            format!("{resource_desc} is permitted for {}", instance.label()),
        ),
        (Modality::Obligation, ActionKind::Present) => (
            !exists,
            if exists {
                format!("{resource_desc} present for {}", instance.label())
            } else {
                format!("{resource_desc} missing for {}", instance.label())
            },
        ),
        (Modality::Obligation, ActionKind::Absent) => (
            exists,
            if exists {
                format!(
                    "{resource_desc} present for {} but must be absent",
                    instance.label()
                )
            } else {
                format!("{resource_desc} absent for {}", instance.label())
            },
        ),
        (Modality::Obligation, ActionKind::Valid) => {
            let valid = resource_valid(&rule.resource, instance, facts);
            (
                !valid,
                if valid {
                    format!("{resource_desc} valid for {}", instance.label())
                } else {
                    format!("{resource_desc} invalid for {}", instance.label())
                },
            )
        }
        (Modality::Prohibition, ActionKind::Present) => (
            exists,
            if exists {
                format!(
                    "{resource_desc} present for {} but is prohibited",
                    instance.label()
                )
            } else {
                format!("{resource_desc} absent for {}", instance.label())
            },
        ),
        (Modality::Prohibition, ActionKind::Absent) => (
            !exists,
            if exists {
                format!("{resource_desc} present for {}", instance.label())
            } else {
                format!(
                    "{resource_desc} absent for {} but its absence is prohibited",
                    instance.label()
                )
            },
        ),
        (Modality::Prohibition, ActionKind::Valid) => {
            let valid = resource_valid(&rule.resource, instance, facts);
            (
                valid,
                if valid {
                    format!(
                        "{resource_desc} valid for {} but validity is prohibited",
                        instance.label()
                    )
                } else {
                    format!("{resource_desc} not valid for {}", instance.label())
                },
            )
        }
        // ConsistentWith is rejected at load; treated as a pass to keep
        // evaluation total.
        (_, ActionKind::ConsistentWith { .. }) => (
            false,
            format!("consistency check skipped for {}", instance.label()),
        ),
    };

    let source = rule
        .source
        .clone()
        .or_else(|| origin.map(|overlay_id| format!("overlay:{overlay_id}")));

    Finding {
        rule_id: rule.id.clone(),
        modality: rule.modality,
        severity: rule.severity,
        subject: instance.label(),
        message,
        rationale: rule.rationale.clone(),
        source,
        status: if violated {
            FindingStatus::Violation
        } else {
            FindingStatus::Pass
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{load_policy_str, PolicyFormat};
    use std::collections::{BTreeMap, BTreeSet};

    fn facts_with(files: &[&str], headers: &[(&str, Option<&str>)]) -> FactSet {
        FactSet {
            root: "test-root".to_string(),
            files: files.iter().map(|s| s.to_string()).collect::<BTreeSet<_>>(),
            spdx_headers: headers
                .iter()
                .map(|(k, v)| (k.to_string(), v.map(|s| s.to_string())))
                .collect::<BTreeMap<_, _>>(),
            metadata: BTreeMap::new(),
            git: Default::default(),
        }
    }

    fn policy(doc: &str) -> Policy {
        load_policy_str(doc, PolicyFormat::Toml).unwrap()
    }

    #[test]
    fn test_glob_zero_component_doublestar() {
        assert!(glob_matches("**/*.rs", "main.rs"));
        assert!(glob_matches("**/*.rs", "src/deep/main.rs"));
        assert!(!glob_matches("**/*.rs", "main.toml"));
        assert!(glob_matches("LICENSE*", "LICENSE"));
        assert!(!glob_matches("LICENSE*", "docs/LICENSE"));
    }

    #[test]
    fn test_obligation_present_violation_and_pass() {
        let p = policy(
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
        );
        let bad = evaluate(&p, &facts_with(&["README.md"], &[]));
        assert_eq!(bad.summary.errors, 1);
        assert_eq!(bad.findings[0].status, FindingStatus::Violation);

        let good = evaluate(&p, &facts_with(&["LICENSE", "README.md"], &[]));
        assert_eq!(good.summary.errors, 0);
        assert_eq!(good.summary.passes, 1);
    }

    #[test]
    fn test_prohibition_present() {
        let p = policy(
            r#"
schema_version = { major = 0, minor = 1 }
id = "t"
version = { major = 1, minor = 0 }

[[rules]]
id = "no-lockfiles"
modality = "prohibition"
severity = "warning"
subject = { type = "repo" }
resource = { type = "file", path = "package-lock.json" }
action = { type = "present" }
"#,
        );
        let bad = evaluate(&p, &facts_with(&["package-lock.json"], &[]));
        assert_eq!(bad.summary.warnings, 1);

        let good = evaluate(&p, &facts_with(&["Cargo.lock"], &[]));
        assert_eq!(good.summary.warnings, 0);
        assert_eq!(good.summary.passes, 1);
    }

    #[test]
    fn test_file_pattern_header_rule() {
        let p = policy(
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
        );
        let facts = facts_with(
            &["src/a.rs", "src/b.rs", "README.md"],
            &[("src/a.rs", Some("MPL-2.0")), ("src/b.rs", None)],
        );
        let eval = evaluate(&p, &facts);
        assert_eq!(eval.summary.warnings, 1);
        assert_eq!(eval.summary.passes, 1);
        let violation = eval
            .findings
            .iter()
            .find(|f| f.status == FindingStatus::Violation)
            .unwrap();
        assert_eq!(violation.subject, "src/b.rs");
    }

    #[test]
    fn test_header_valid_action() {
        let p = policy(
            r#"
schema_version = { major = 0, minor = 1 }
id = "t"
version = { major = 1, minor = 0 }

[[rules]]
id = "headers-parse"
modality = "obligation"
severity = "warning"
subject = { type = "file-pattern", patterns = ["**/*.rs"] }
resource = { type = "header" }
condition = { type = "has-spdx-header" }
action = { type = "valid" }
"#,
        );
        let facts = facts_with(
            &["good.rs", "bad.rs", "none.rs"],
            &[
                ("good.rs", Some("MIT OR Apache-2.0")),
                ("bad.rs", Some("NOT ) A ( LICENSE")),
                ("none.rs", None),
            ],
        );
        let eval = evaluate(&p, &facts);
        // none.rs is filtered out by the condition; bad.rs violates.
        assert_eq!(eval.summary.warnings, 1);
        assert_eq!(eval.summary.passes, 1);
    }

    #[test]
    fn test_condition_gates_rule() {
        let p = policy(
            r#"
schema_version = { major = 0, minor = 1 }
id = "t"
version = { major = 1, minor = 0 }

[[rules]]
id = "changelog-once-versioned"
modality = "obligation"
subject = { type = "repo" }
resource = { type = "file", path = "CHANGELOG*" }
condition = { type = "version-at-least", version = "1.0" }
action = { type = "present" }
"#,
        );
        // No version metadata: condition false, rule not applicable.
        let eval = evaluate(&p, &facts_with(&["README.md"], &[]));
        assert_eq!(eval.findings.len(), 0);

        // Version >= 1.0: rule applies and is violated.
        let mut facts = facts_with(&["README.md"], &[]);
        facts
            .metadata
            .insert("version".to_string(), "1.2.0".to_string());
        let eval = evaluate(&p, &facts);
        assert_eq!(eval.summary.errors, 1);
    }

    #[test]
    fn test_permission_never_violates() {
        let p = policy(
            r#"
schema_version = { major = 0, minor = 1 }
id = "t"
version = { major = 1, minor = 0 }

[[rules]]
id = "vendoring-permitted"
modality = "permission"
subject = { type = "repo" }
resource = { type = "file", path = "vendor/**" }
action = { type = "present" }
"#,
        );
        let eval = evaluate(&p, &facts_with(&[], &[]));
        assert_eq!(eval.summary.errors, 0);
        assert_eq!(eval.summary.passes, 1);
    }

    #[test]
    fn test_override_disables_base_rule() {
        // Base requires a LICENSE; overlay overrides that rule away.
        let p = policy(
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

[[overlays]]
id = "vendored-exemption"
applies_to = []

[[overlays.effects]]
type = "override-rules"
ids = ["license-present"]
"#,
        );
        // No LICENSE, but the rule is overridden away → no findings at all.
        let eval = evaluate(&p, &facts_with(&["README.md"], &[]));
        assert_eq!(eval.findings.len(), 0);
    }

    #[test]
    fn test_override_with_replacement() {
        // Base marks missing LICENSE an error; overlay overrides it and adds
        // a same-id replacement at warning severity.
        let p = policy(
            r#"
schema_version = { major = 0, minor = 1 }
id = "t"
version = { major = 1, minor = 0 }

[[rules]]
id = "license-present"
modality = "obligation"
severity = "error"
subject = { type = "repo" }
resource = { type = "file", path = "LICENSE*" }
action = { type = "present" }

[[overlays]]
id = "relax"
applies_to = []

[[overlays.effects]]
type = "override-rules"
ids = ["license-present"]

[[overlays.effects]]
type = "add-rules"

[[overlays.effects.rules]]
id = "license-present"
modality = "obligation"
severity = "warning"
subject = { type = "repo" }
resource = { type = "file", path = "LICENSE*" }
action = { type = "present" }
"#,
        );
        let eval = evaluate(&p, &facts_with(&["README.md"], &[]));
        // Exactly one finding, at the replacement's warning severity.
        assert_eq!(eval.summary.errors, 0);
        assert_eq!(eval.summary.warnings, 1);
        assert_eq!(eval.findings[0].source.as_deref(), Some("overlay:relax"));
    }

    #[test]
    fn test_overlay_rules_carry_origin() {
        let p = policy(
            r#"
schema_version = { major = 0, minor = 1 }
id = "t"
version = { major = 1, minor = 0 }
rules = []

[[overlays]]
id = "extra"
applies_to = []

[[overlays.effects]]
type = "add-rules"

[[overlays.effects.rules]]
id = "extra-rule"
modality = "obligation"
subject = { type = "repo" }
resource = { type = "file", path = "SECURITY.md" }
action = { type = "present" }
"#,
        );
        let eval = evaluate(&p, &facts_with(&[], &[]));
        assert_eq!(eval.summary.errors, 1);
        assert_eq!(eval.findings[0].source.as_deref(), Some("overlay:extra"));
    }

    #[test]
    fn test_determinism_identical_runs() {
        let p = crate::schema::builtin_repo_hygiene();
        let facts = facts_with(
            &["LICENSE", "README.adoc", "src/main.rs", "Cargo.toml"],
            &[
                ("src/main.rs", Some("MPL-2.0")),
                ("Cargo.toml", Some("MPL-2.0")),
            ],
        );
        let a = evaluate(&p, &facts);
        let b = evaluate(&p, &facts);
        assert_eq!(a, b);
        assert_eq!(
            serde_json::to_string(&a).unwrap(),
            serde_json::to_string(&b).unwrap()
        );
    }

    #[test]
    fn test_version_compare() {
        assert_eq!(compare_versions("1.2.3", "1.0"), Some(true));
        assert_eq!(compare_versions("0.9", "1.0"), Some(false));
        assert_eq!(compare_versions("1.0.0", "1.0"), Some(true));
        assert_eq!(compare_versions("2", "1.9.9"), Some(true));
        assert_eq!(compare_versions("garbage", "1.0"), None);
    }
}
