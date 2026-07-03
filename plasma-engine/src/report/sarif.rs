// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// SARIF output — renders an Evaluation as SARIF v2.1.0 for GitHub Code
// Scanning and sibling tools. Rule ids are namespaced "plasma/<rule_id>"
// and stable across runs: they are the interchange contract consumed by
// forensic and claim-verification tooling.
//
// SARIF spec: https://docs.oasis-open.org/sarif/sarif/v2.1.0/sarif-v2.1.0.html

use crate::finding::{Evaluation, Finding, FindingStatus, Severity};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// SARIF `level` vocabulary: error | warning | note (never "info").
fn sarif_level(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "note",
    }
}

fn result_for(finding: &Finding) -> Value {
    let mut result = json!({
        "ruleId": format!("plasma/{}", finding.rule_id),
        "message": { "text": finding.message },
    });

    match finding.status {
        FindingStatus::Violation => {
            result["level"] = json!(sarif_level(finding.severity));
        }
        FindingStatus::Pass => {
            result["kind"] = json!("pass");
            result["level"] = json!("none");
        }
    }

    // Repo- and metadata-scoped findings have no artifact location;
    // SARIF permits results without locations.
    if finding.subject != "repo" && !finding.subject.starts_with("metadata:") {
        result["locations"] = json!([{
            "physicalLocation": {
                "artifactLocation": { "uri": finding.subject },
                "region": { "startLine": 1 }
            }
        }]);
    }

    result
}

/// Render an evaluation as a SARIF v2.1.0 document.
///
/// Violations are always included; pass findings (`kind: "pass"`) only
/// with `include_passes`.
pub fn to_sarif(eval: &Evaluation, include_passes: bool) -> Value {
    // One SARIF rule per policy rule id, in deterministic order.
    let mut rules: BTreeMap<String, Value> = BTreeMap::new();
    for finding in &eval.findings {
        let id = format!("plasma/{}", finding.rule_id);
        rules.entry(id.clone()).or_insert_with(|| {
            let mut rule = json!({ "id": id });
            if let Some(rationale) = &finding.rationale {
                rule["shortDescription"] = json!({ "text": rationale });
            }
            rule
        });
    }

    let results: Vec<Value> = eval
        .findings
        .iter()
        .filter(|f| include_passes || f.status == FindingStatus::Violation)
        .map(result_for)
        .collect();

    json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "plasma",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/hyperpolymath/palimpsest-plasma",
                    "rules": rules.into_values().collect::<Vec<_>>()
                }
            },
            "results": results
        }]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Modality, PolicyVersion};
    use crate::finding::{Evaluation, Finding, Summary};

    #[test]
    fn test_sarif_shape() {
        let eval = Evaluation {
            root: ".".to_string(),
            policy_id: "t".to_string(),
            policy_version: PolicyVersion { major: 1, minor: 0 },
            schema_version: PolicyVersion { major: 0, minor: 1 },
            findings: vec![Finding {
                rule_id: "license-file-present".to_string(),
                modality: Modality::Obligation,
                severity: Severity::Error,
                subject: "repo".to_string(),
                message: "file \"LICENSE*\" missing for repo".to_string(),
                rationale: Some("Every repository must carry its license text.".to_string()),
                source: None,
                status: FindingStatus::Violation,
            }],
            summary: Summary {
                errors: 1,
                ..Default::default()
            },
        };

        let sarif = to_sarif(&eval, false);
        assert_eq!(sarif["version"], "2.1.0");
        assert_eq!(sarif["runs"][0]["tool"]["driver"]["name"], "plasma");
        let result = &sarif["runs"][0]["results"][0];
        assert_eq!(result["ruleId"], "plasma/license-file-present");
        assert_eq!(result["level"], "error");
        // Repo-scoped: no artifact location.
        assert!(result.get("locations").is_none());
    }
}
