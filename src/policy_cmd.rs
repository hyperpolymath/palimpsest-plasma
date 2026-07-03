// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// plasma policy — policy file utilities.

use anyhow::{Context, Result};
use plasma_engine::load_policy;
use std::path::Path;

/// Validate a policy file: parse + schema check. Prints a summary on success.
pub fn validate(file: &str) -> Result<()> {
    let policy =
        load_policy(Path::new(file)).with_context(|| format!("policy {file} failed validation"))?;

    let overlay_rules: usize = policy
        .overlays
        .iter()
        .flat_map(|o| &o.effects)
        .map(|e| match e {
            plasma_engine::ast::OverlayEffect::AddRules { rules } => rules.len(),
            _ => 0,
        })
        .sum();

    println!(
        "OK: {} v{} (schema {}) — {} base rule(s), {} overlay rule(s)",
        policy.id,
        policy.version,
        policy.schema_version,
        policy.rules.len(),
        overlay_rules
    );
    Ok(())
}
