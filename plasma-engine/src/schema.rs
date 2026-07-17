// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Policy schema — versioned loading and validation.
//
// Every construct the v0 evaluator cannot give exact semantics to is
// rejected HERE, at load time, never mid-evaluation. This keeps
// `evaluate` total: any policy that loads, evaluates.

use crate::ast::{
    ActionKind, Condition, OverlayEffect, Policy, PolicyVersion, Resource, Rule, Subject,
};
use std::path::Path;
use thiserror::Error;

/// The policy schema version this engine implements.
pub const SCHEMA_VERSION: PolicyVersion = PolicyVersion { major: 0, minor: 1 };

/// Policy document formats accepted by the loader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyFormat {
    Toml,
    Json,
}

/// Errors produced while loading or validating a policy document.
#[derive(Debug, Error)]
pub enum SchemaError {
    #[error("cannot read policy file: {0}")]
    Io(#[from] std::io::Error),
    #[error("cannot parse policy TOML: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("cannot parse policy JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error(
        "unsupported schema version {found} (this engine implements {supported}); \
         migrate the policy or upgrade the engine"
    )]
    UnsupportedSchemaVersion {
        found: PolicyVersion,
        supported: PolicyVersion,
    },
    #[error("rule {rule_id}: {construct} is schema-reserved and not evaluable in v0")]
    ReservedConstruct { rule_id: String, construct: String },
    #[error("overlay {overlay_id}: {construct} is schema-reserved and not evaluable in v0")]
    ReservedOverlayEffect {
        overlay_id: String,
        construct: String,
    },
    #[error("duplicate rule id: {0}")]
    DuplicateRuleId(String),
    #[error("overlay {overlay_id}: override-rules targets unknown base rule id {rule_id}")]
    OverrideUnknownRule { overlay_id: String, rule_id: String },
    #[error("unknown policy file extension {0:?}: use .toml or .json")]
    UnknownExtension(String),
}

/// Load and validate a policy from a file, picking the format by extension.
pub fn load_policy(path: &Path) -> Result<Policy, SchemaError> {
    let format = match path.extension().and_then(|e| e.to_str()) {
        Some("toml") => PolicyFormat::Toml,
        Some("json") => PolicyFormat::Json,
        other => {
            return Err(SchemaError::UnknownExtension(
                other.unwrap_or("<none>").to_string(),
            ))
        }
    };
    let content = std::fs::read_to_string(path)?;
    load_policy_str(&content, format)
}

/// Load and validate a policy from a string in the given format.
pub fn load_policy_str(content: &str, format: PolicyFormat) -> Result<Policy, SchemaError> {
    let policy: Policy = match format {
        PolicyFormat::Toml => toml::from_str(content)?,
        PolicyFormat::Json => serde_json::from_str(content)?,
    };
    validate(&policy)?;
    Ok(policy)
}

/// The repo-hygiene policy bundled with the engine. Guaranteed to load.
pub fn builtin_repo_hygiene() -> Policy {
    let content = include_str!("../policies/repo-hygiene.plasma.toml");
    load_policy_str(content, PolicyFormat::Toml).expect("bundled repo-hygiene policy must be valid")
}

/// Validate a parsed policy against v0 evaluability rules.
fn validate(policy: &Policy) -> Result<(), SchemaError> {
    if policy.schema_version != SCHEMA_VERSION {
        return Err(SchemaError::UnsupportedSchemaVersion {
            found: policy.schema_version,
            supported: SCHEMA_VERSION,
        });
    }

    // `modify-rules` is still schema-reserved (its diff semantics are
    // underspecified); `override-rules` is supported as of schema 0.1.
    for overlay in &policy.overlays {
        for effect in &overlay.effects {
            if let OverlayEffect::ModifyRules { .. } = effect {
                return Err(SchemaError::ReservedOverlayEffect {
                    overlay_id: overlay.id.clone(),
                    construct: "modify-rules".to_string(),
                });
            }
        }
    }

    // Base rule ids must be unique among themselves.
    let mut base_ids = std::collections::BTreeSet::new();
    for rule in &policy.rules {
        if !base_ids.insert(rule.id.clone()) {
            return Err(SchemaError::DuplicateRuleId(rule.id.clone()));
        }
    }

    // Every overridden id must name a real base rule. Collect the set so
    // an added rule may legally reuse an overridden id (a replacement).
    let mut overridden = std::collections::BTreeSet::new();
    for overlay in &policy.overlays {
        for effect in &overlay.effects {
            if let OverlayEffect::OverrideRules { ids } = effect {
                for id in ids {
                    if !base_ids.contains(id) {
                        return Err(SchemaError::OverrideUnknownRule {
                            overlay_id: overlay.id.clone(),
                            rule_id: id.clone(),
                        });
                    }
                    overridden.insert(id.clone());
                }
            }
        }
    }

    // Effective base ids (minus overridden) plus every overlay-added id
    // must be collision-free. A replacement — an added rule whose id was
    // overridden — is allowed precisely because the base rule is gone.
    let mut seen_ids: std::collections::BTreeSet<String> =
        base_ids.difference(&overridden).cloned().collect();
    for overlay in &policy.overlays {
        for effect in &overlay.effects {
            if let OverlayEffect::AddRules { rules } = effect {
                for rule in rules {
                    if !seen_ids.insert(rule.id.clone()) {
                        return Err(SchemaError::DuplicateRuleId(rule.id.clone()));
                    }
                }
            }
        }
    }

    // Validate every rule (base + overlay-added), including overridden base
    // rules — they must still be well-formed even though evaluation skips
    // them, so a later override removal doesn't resurrect a bad rule.
    let added = policy.overlays.iter().flat_map(|o| {
        o.effects.iter().filter_map(|e| match e {
            OverlayEffect::AddRules { rules } => Some(rules.iter()),
            _ => None,
        })
    });
    for rule in policy.rules.iter().chain(added.flatten()) {
        validate_rule(rule)?;
    }

    Ok(())
}

/// Reject rules using schema-reserved constructs the v0 evaluator has no
/// semantics for.
fn validate_rule(rule: &Rule) -> Result<(), SchemaError> {
    let reserved = |construct: &str| SchemaError::ReservedConstruct {
        rule_id: rule.id.clone(),
        construct: construct.to_string(),
    };

    if matches!(rule.subject, Subject::Release { .. }) {
        return Err(reserved("subject release"));
    }
    if matches!(rule.resource, Resource::Release) {
        return Err(reserved("resource release"));
    }
    if matches!(rule.action, ActionKind::ConsistentWith { .. }) {
        return Err(reserved("action consistent-with"));
    }
    validate_condition(&rule.condition, &rule.id)
}

fn validate_condition(condition: &Condition, rule_id: &str) -> Result<(), SchemaError> {
    match condition {
        Condition::FileMatchesPattern { .. } => Err(SchemaError::ReservedConstruct {
            rule_id: rule_id.to_string(),
            construct: "condition file-matches-pattern (v0 facts carry no file contents)"
                .to_string(),
        }),
        Condition::Not { of } => validate_condition(of, rule_id),
        Condition::All { of } | Condition::Any { of } => {
            for c in of {
                validate_condition(c, rule_id)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_loads() {
        let policy = builtin_repo_hygiene();
        assert_eq!(policy.id, "repo-hygiene");
        assert_eq!(policy.schema_version, SCHEMA_VERSION);
        assert!(!policy.rules.is_empty());
    }

    #[test]
    fn test_rejects_future_schema() {
        let doc = r#"
schema_version = { major = 9, minor = 0 }
id = "future"
version = { major = 1, minor = 0 }
rules = []
"#;
        let err = load_policy_str(doc, PolicyFormat::Toml).unwrap_err();
        assert!(matches!(err, SchemaError::UnsupportedSchemaVersion { .. }));
    }

    #[test]
    fn test_rejects_reserved_action() {
        let doc = r#"
schema_version = { major = 0, minor = 1 }
id = "reserved"
version = { major = 1, minor = 0 }

[[rules]]
id = "r1"
modality = "obligation"
subject = { type = "repo" }
resource = { type = "file", path = "LICENSE" }
action = { type = "consistent-with", id = "decision-1" }
"#;
        let err = load_policy_str(doc, PolicyFormat::Toml).unwrap_err();
        assert!(matches!(err, SchemaError::ReservedConstruct { .. }));
    }

    #[test]
    fn test_rejects_duplicate_rule_ids() {
        let doc = r#"
schema_version = { major = 0, minor = 1 }
id = "dupes"
version = { major = 1, minor = 0 }

[[rules]]
id = "same"
modality = "obligation"
subject = { type = "repo" }
resource = { type = "file", path = "LICENSE" }
action = { type = "present" }

[[rules]]
id = "same"
modality = "obligation"
subject = { type = "repo" }
resource = { type = "file", path = "README" }
action = { type = "present" }
"#;
        let err = load_policy_str(doc, PolicyFormat::Toml).unwrap_err();
        assert!(matches!(err, SchemaError::DuplicateRuleId(_)));
    }

    #[test]
    fn test_override_unknown_rule_rejected() {
        let doc = r#"
schema_version = { major = 0, minor = 1 }
id = "t"
version = { major = 1, minor = 0 }

[[rules]]
id = "real"
modality = "obligation"
subject = { type = "repo" }
resource = { type = "file", path = "LICENSE" }
action = { type = "present" }

[[overlays]]
id = "o"
applies_to = []

[[overlays.effects]]
type = "override-rules"
ids = ["does-not-exist"]
"#;
        let err = load_policy_str(doc, PolicyFormat::Toml).unwrap_err();
        assert!(matches!(err, SchemaError::OverrideUnknownRule { .. }));
    }

    #[test]
    fn test_override_allows_same_id_replacement() {
        let doc = r#"
schema_version = { major = 0, minor = 1 }
id = "t"
version = { major = 1, minor = 0 }

[[rules]]
id = "r"
modality = "obligation"
subject = { type = "repo" }
resource = { type = "file", path = "LICENSE" }
action = { type = "present" }

[[overlays]]
id = "o"
applies_to = []

[[overlays.effects]]
type = "override-rules"
ids = ["r"]

[[overlays.effects]]
type = "add-rules"

[[overlays.effects.rules]]
id = "r"
modality = "obligation"
subject = { type = "repo" }
resource = { type = "file", path = "README" }
action = { type = "present" }
"#;
        // Replacing an overridden id is not a duplicate.
        assert!(load_policy_str(doc, PolicyFormat::Toml).is_ok());
    }

    #[test]
    fn test_modify_rules_still_reserved() {
        let doc = r#"
schema_version = { major = 0, minor = 1 }
id = "t"
version = { major = 1, minor = 0 }
rules = []

[[overlays]]
id = "o"
applies_to = []

[[overlays.effects]]
type = "modify-rules"
ids = ["whatever"]
"#;
        let err = load_policy_str(doc, PolicyFormat::Toml).unwrap_err();
        assert!(matches!(err, SchemaError::ReservedOverlayEffect { .. }));
    }

    #[test]
    fn test_json_round_trip() {
        let policy = builtin_repo_hygiene();
        let json = serde_json::to_string_pretty(&policy).unwrap();
        let reloaded = load_policy_str(&json, PolicyFormat::Json).unwrap();
        assert_eq!(policy, reloaded);
    }
}
