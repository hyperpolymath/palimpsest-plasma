// SPDX-License-Identifier: PMPL-2.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// plasma-parser — Core library for the palimpsest license family parser.
//
// This crate provides:
// - SPDX expression lexing and parsing (including palimpsest extensions)
// - License family type system (base + layer + fallback)
// - Repository zone scanning and auditing
// - License compatibility matrix
// - JSON and SARIF report generation

#![forbid(unsafe_code)]

pub mod audit;
pub mod compat;
pub mod family;
pub mod report;
pub mod spdx;
pub mod zone;

// Re-export core types for ergonomic usage by consumers.
pub use audit::{AuditStatus, FileAudit, RepoAudit};
pub use compat::{CompatWarning, Compatibility};
pub use family::{
    BaseLicense, CcVariant, FamilyError, License, PalimpsestLayer, PalimpsestLicense,
    PalimpsestVariant, Version,
};
pub use spdx::SpdxExpr;
pub use zone::{LicenseMap, LicenseZone};
