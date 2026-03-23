// SPDX-License-Identifier: PMPL-2.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// JSON report output — serialises RepoAudit to JSON for CLI and daemon
// consumption.

use crate::audit::RepoAudit;

/// Serialise a RepoAudit to a pretty-printed JSON string.
pub fn to_json(audit: &RepoAudit) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(audit)
}

/// Serialise a RepoAudit to a compact JSON string (for HTTP responses).
pub fn to_json_compact(audit: &RepoAudit) -> Result<String, serde_json::Error> {
    serde_json::to_string(audit)
}
