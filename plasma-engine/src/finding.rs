// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Findings — the evaluator's output vocabulary.

use crate::ast::{Modality, PolicyVersion};
use serde::{Deserialize, Serialize};

/// Finding severity, ordered from most to least severe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
        }
    }
}

/// Whether a rule was violated or satisfied for a subject instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FindingStatus {
    Violation,
    Pass,
}

/// One evaluated (rule, subject instance) outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// The id of the rule that produced this finding.
    pub rule_id: String,
    pub modality: Modality,
    pub severity: Severity,
    /// Concrete subject instance: "repo", a relative file path, or a
    /// metadata key.
    pub subject: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub status: FindingStatus,
}

/// Violation counts by severity, plus satisfied-rule count.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Summary {
    pub errors: u32,
    pub warnings: u32,
    pub info: u32,
    pub passes: u32,
}

/// The complete, deterministic result of evaluating a policy against facts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evaluation {
    /// Repository root the facts were collected from.
    pub root: String,
    pub policy_id: String,
    pub policy_version: PolicyVersion,
    pub schema_version: PolicyVersion,
    pub findings: Vec<Finding>,
    pub summary: Summary,
}

impl Evaluation {
    /// True when no violation at or above the given severity exists.
    pub fn clean_at(&self, threshold: Severity) -> bool {
        !self
            .findings
            .iter()
            .any(|f| f.status == FindingStatus::Violation && f.severity <= threshold)
    }
}
