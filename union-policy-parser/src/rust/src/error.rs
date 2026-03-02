// SPDX-License-Identifier: PMPL-1.0-or-later

//! Union Policy Parser Error Space.
//!
//! This module defines the failure modes for the contract auditing tool. 
//! It uses `thiserror` to provide semantic error reporting across 
//! parsing, validation, and reporting stages.

use thiserror::Error;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum PolicyError {
    /// PARSE: The A2ML source file is syntactically invalid.
    #[error("Failed to parse A2ML file: {0}")]
    ParseError(String),

    /// VALIDATION: The contract violates a structural or logical rule.
    #[error("Validation failed: {0}")]
    ValidationError(String),

    /// MANDATORY: A required clause (e.g. 'source-protection') was not found.
    #[error("Missing required clause: {0}")]
    MissingClause(String),

    /// VALUE: A clause value (e.g. NET-90) violates union standards.
    #[error("Invalid clause value for '{clause}': expected {expected}, got {actual}")]
    InvalidClauseValue {
        clause: String,
        expected: String,
        actual: String,
    },

    /// TEMPLATE: The grievance letter template is malformed.
    #[error("Template error: {0}")]
    TemplateError(String),

    /// SYSTEM: Wrapped IO and Serialization errors.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, PolicyError>;
