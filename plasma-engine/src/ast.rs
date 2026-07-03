// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Policy AST — typed, deontic policy representation.
//
// Transposed from docs/policy-ast-v0.1.adoc with documented divergences
// (see docs/engine-v0-design.adoc): exhibits are generalised to overlays,
// rules carry an explicit severity, and the condition vocabulary is
// license-agnostic.

use crate::finding::Severity;
use serde::{Deserialize, Serialize};

/// Schema/policy version as a major.minor pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PolicyVersion {
    pub major: u32,
    pub minor: u32,
}

impl std::fmt::Display for PolicyVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// The entity a rule is evaluated against.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Subject {
    /// The repository as a whole.
    Repo,
    /// A single file by relative path.
    File { path: String },
    /// Every file matching any of the glob patterns. The `glob` crate has
    /// no brace expansion, hence a list rather than `{a,b}` syntax.
    FilePattern { patterns: Vec<String> },
    /// A release by tag. Schema-reserved: rejected by the v0 loader.
    Release { tag: String },
    /// A governance metadata field by key.
    Metadata { key: String },
}

/// The artefact checked within or about a subject.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Resource {
    /// A file by relative path; the path may be a glob (e.g. "LICENSE*").
    File { path: String },
    /// The SPDX header of the subject file.
    Header,
    /// A manifest file by relative path.
    Manifest { path: String },
    /// A governance metadata field by key.
    GovernanceField { key: String },
    /// The release artefact. Schema-reserved: rejected by the v0 loader.
    Release,
}

/// Composable, total predicates over the fact set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Condition {
    /// Always true (the default condition).
    True,
    /// Logical negation.
    Not { of: Box<Condition> },
    /// Logical conjunction over all sub-conditions.
    All { of: Vec<Condition> },
    /// Logical disjunction over the sub-conditions.
    Any { of: Vec<Condition> },
    /// At least one repository file matches the glob.
    RepoHasFile { path: String },
    /// Content predicate. Schema-reserved: rejected by the v0 loader
    /// (v0 facts carry no file contents).
    FileMatchesPattern { path: String, pattern: String },
    /// The subject file has an SPDX header (false for non-file subjects).
    HasSpdxHeader,
    /// The subject file's SPDX header parses and equals this expression
    /// textually after normalisation (false for non-file subjects).
    SpdxLicenseIs { expr: String },
    /// The governance metadata key is set to exactly this value
    /// (false when the key is absent).
    GovernanceFlagSet { key: String, value: String },
    /// The `version` metadata value is at least this version
    /// (false when absent or unparsable).
    VersionAtLeast { version: String },
}

impl Condition {
    pub(crate) fn always_true() -> Condition {
        Condition::True
    }
}

/// Deontic modality: what the rule obligates, prohibits, or permits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Modality {
    Obligation,
    Prohibition,
    Permission,
}

impl std::fmt::Display for Modality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Modality::Obligation => write!(f, "obligation"),
            Modality::Prohibition => write!(f, "prohibition"),
            Modality::Permission => write!(f, "permission"),
        }
    }
}

/// What the rule asserts about the subject-resource pair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ActionKind {
    /// The resource must exist.
    Present,
    /// The resource must not exist.
    Absent,
    /// The resource must be structurally valid (for headers: parse as SPDX).
    Valid,
    /// Consistency with an external decision. Schema-reserved: rejected by
    /// the v0 loader.
    ConsistentWith { id: String },
}

fn default_condition() -> Condition {
    Condition::always_true()
}

fn default_severity() -> Severity {
    Severity::Error
}

/// The fundamental unit of policy: a deontic modality bound to a subject,
/// resource, condition, and action, with optional narrative provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub modality: Modality,
    #[serde(default = "default_severity")]
    pub severity: Severity,
    pub subject: Subject,
    pub resource: Resource,
    #[serde(default = "default_condition")]
    pub condition: Condition,
    pub action: ActionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// An overlay effect: how an overlay extends or modifies the base rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum OverlayEffect {
    /// Add rules to the effective rule set.
    AddRules { rules: Vec<Rule> },
    /// Modify existing rules by id. Schema-reserved: rejected by the v0 loader.
    ModifyRules { ids: Vec<String> },
    /// Override existing rules by id. Schema-reserved: rejected by the v0 loader.
    OverrideRules { ids: Vec<String> },
}

/// An overlay: a named, additive policy extension (the generalisation of the
/// original design's license "exhibits").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Overlay {
    pub id: String,
    #[serde(default)]
    pub applies_to: Vec<String>,
    pub effects: Vec<OverlayEffect>,
}

/// A policy: versioned schema, versioned content, base rules, and overlays.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Policy {
    /// Schema version this document is written against. Checked at load.
    pub schema_version: PolicyVersion,
    pub id: String,
    /// The policy's own content version.
    pub version: PolicyVersion,
    pub rules: Vec<Rule>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overlays: Vec<Overlay>,
}
