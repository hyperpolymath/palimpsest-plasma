// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Human-readable rendering of an Evaluation.

use crate::finding::{Evaluation, FindingStatus};

/// Render an evaluation as human-readable text.
///
/// Violations are always shown; satisfied (pass) findings only with
/// `verbose`.
pub fn to_human(eval: &Evaluation, verbose: bool) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "Policy {} v{} (schema {}) against {}\n\n",
        eval.policy_id, eval.policy_version, eval.schema_version, eval.root
    ));

    let mut shown = 0usize;
    for finding in &eval.findings {
        match finding.status {
            FindingStatus::Violation => {
                out.push_str(&format!(
                    "  {}: [{}] {} — {}\n",
                    finding.severity.to_string().to_uppercase(),
                    finding.rule_id,
                    finding.subject,
                    finding.message
                ));
                if let Some(rationale) = &finding.rationale {
                    out.push_str(&format!("      rationale: {rationale}\n"));
                }
                if let Some(source) = &finding.source {
                    out.push_str(&format!("      source: {source}\n"));
                }
                shown += 1;
            }
            FindingStatus::Pass if verbose => {
                out.push_str(&format!(
                    "  PASS: [{}] {} — {}\n",
                    finding.rule_id, finding.subject, finding.message
                ));
                shown += 1;
            }
            FindingStatus::Pass => {}
        }
    }

    if shown == 0 {
        out.push_str("  No violations.\n");
    }

    out.push_str(&format!(
        "\nSummary: {} error(s), {} warning(s), {} info, {} rule(s) satisfied\n",
        eval.summary.errors, eval.summary.warnings, eval.summary.info, eval.summary.passes
    ));

    out
}
