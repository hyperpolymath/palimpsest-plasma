// SPDX-License-Identifier: PMPL-2.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// License compatibility module — checks whether two licenses can coexist
// in the same repository or combined work.

pub mod matrix;

use serde::{Deserialize, Serialize};

/// Compatibility between two licenses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Compatibility {
    /// Fully compatible — can be combined freely.
    Compatible,
    /// Compatible under conditions (e.g., file-level separation for MPL).
    Conditional(String),
    /// Incompatible — cannot be combined in a single binary/project.
    Incompatible(String),
    /// One is proprietary — combination depends on the open license's terms.
    ProprietaryDepends(String),
    /// Unknown — at least one license is unrecognised.
    Unknown,
}

/// A warning about compatibility between two zones.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatWarning {
    /// Name of the first zone.
    pub zone_a: String,
    /// Name of the second zone.
    pub zone_b: String,
    /// The compatibility result.
    pub compatibility: Compatibility,
    /// Human-readable explanation.
    pub explanation: String,
}
