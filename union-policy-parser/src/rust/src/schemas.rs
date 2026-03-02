// SPDX-License-Identifier: PMPL-1.0-or-later

//! Union-Specific Policy Schemas.
//!
//! This module defines the authoritative standards for various trade unions. 
//! It maps union identities to their required clauses and identifies 
//! exploitative "Red Flag" patterns in employment contracts.

use crate::error::{PolicyError, Result};
use std::collections::HashMap;

/// SUPPORTED UNIONS: 
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Union {
    Nuj, // National Union of Journalists
    Iww, // Industrial Workers of the World
    Ucu, // University and College Union
}

impl Union {
    /// POLICY: Returns the set of clauses that MUST be present for a 
    /// contract to be considered ethical by the specific union.
    pub fn required_clauses(&self) -> Vec<&'static str> {
        match self {
            Union::Nuj => vec!["truth-accuracy", "independence", "source-protection"],
            Union::Iww => vec!["payment-terms", "late-payment-penalty", "no-spec-work"],
            Union::Ucu => vec!["academic-freedom", "workload-limits", "no-casualization"],
        }
    }

    /// AUDIT: Defines regex patterns for exploitative language.
    /// Rejects contracts containing waivers of moral rights or 
    /// excessive payment delays.
    pub fn red_flag_patterns(&self) -> Vec<&'static str> {
        match self {
            Union::Nuj => vec!["all rights", "work for hire", "no source protection"],
            Union::Iww => vec!["unpaid", "payment on publication", "NET 90"],
            Union::Ucu => vec!["zero hours", "unlimited hours", "no research time"],
        }
    }
}

/// RULE ENGINE: Implements specific value checks for individual clauses.
pub struct UnionRules {
    union: Union,
}

impl UnionRules {
    /// VALIDATION: Verifies that a clause value (e.g. "30") meets union 
    /// minimum standards (e.g. NET-30).
    pub fn check_clause_value(&self, clause: &str, value: &str) -> Result<bool> {
        match self.union {
            Union::Iww => self.check_iww_clause(clause, value),
            // ... [Dispatch to other union-specific checks]
            _ => Ok(true),
        }
    }

    fn check_iww_clause(&self, clause: &str, value: &str) -> Result<bool> {
        match clause {
            "payment-terms.net-days" => {
                // IWW MANDATE: NET-30 or better.
                let days: u32 = value.parse().map_err(|_| PolicyError::ValidationError("Invalid days".into()))?;
                Ok(days <= 30)
            },
            _ => Ok(true),
        }
    }
}
