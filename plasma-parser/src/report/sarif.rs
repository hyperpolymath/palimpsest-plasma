// SPDX-License-Identifier: PMPL-2.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// SARIF output — generates Static Analysis Results Interchange Format
// for GitHub Code Scanning integration.
//
// SARIF spec: https://docs.oasis-open.org/sarif/sarif/v2.1.0/sarif-v2.1.0.html

use crate::audit::{AuditStatus, RepoAudit};
use serde_json::{json, Value};

/// Generate a SARIF v2.1.0 report from a RepoAudit.
///
/// This produces a report compatible with GitHub Code Scanning's
/// `github/codeql-action/upload-sarif` action.
pub fn to_sarif(audit: &RepoAudit) -> Value {
    let results: Vec<Value> = audit
        .files
        .iter()
        .filter(|f| f.status != AuditStatus::Compliant && f.status != AuditStatus::Unreadable)
        .map(|f| {
            let (rule_id, message) = match &f.status {
                AuditStatus::WrongLicense => (
                    "plasma/wrong-license",
                    format!(
                        "Expected {} but found {:?}",
                        f.expected_license,
                        f.actual_header
                    ),
                ),
                AuditStatus::MissingHeader => (
                    "plasma/missing-header",
                    format!("Missing SPDX header; expected {}", f.expected_license),
                ),
                AuditStatus::ProprietaryViolation => (
                    "plasma/proprietary-violation",
                    "Open-source header found in proprietary zone".to_string(),
                ),
                AuditStatus::OpenSourceViolation => (
                    "plasma/open-source-violation",
                    "Proprietary header found in open-source zone".to_string(),
                ),
                _ => ("plasma/unknown", "Unknown audit issue".to_string()),
            };

            json!({
                "ruleId": rule_id,
                "level": "warning",
                "message": { "text": message },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": f.path.display().to_string()
                        },
                        "region": { "startLine": 1 }
                    }
                }]
            })
        })
        .collect();

    json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "palimpsest-plasma",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/hyperpolymath/palimpsest-plasma",
                    "rules": [
                        {
                            "id": "plasma/wrong-license",
                            "shortDescription": { "text": "SPDX header does not match zone license" },
                            "helpUri": "https://github.com/hyperpolymath/palimpsest-license"
                        },
                        {
                            "id": "plasma/missing-header",
                            "shortDescription": { "text": "Source file is missing SPDX-License-Identifier header" },
                            "helpUri": "https://github.com/hyperpolymath/palimpsest-license"
                        },
                        {
                            "id": "plasma/proprietary-violation",
                            "shortDescription": { "text": "Open-source header in proprietary zone" }
                        },
                        {
                            "id": "plasma/open-source-violation",
                            "shortDescription": { "text": "Proprietary header in open-source zone" }
                        }
                    ]
                }
            },
            "results": results
        }]
    })
}
