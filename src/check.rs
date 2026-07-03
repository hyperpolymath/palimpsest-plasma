// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// plasma check — evaluate a repository against a policy.

use anyhow::{Context, Result};
use plasma_engine::{builtin_repo_hygiene, collect, evaluate, load_policy, report, Severity};
use std::path::Path;

/// Exit code for a check with violations at or above the threshold.
pub const EXIT_VIOLATIONS: i32 = 1;

pub struct CheckOptions<'a> {
    pub path: &'a str,
    pub policy: Option<&'a str>,
    pub format: &'a str,
    pub severity: &'a str,
    pub quiet: bool,
    pub verbose: bool,
}

pub fn run(opts: &CheckOptions) -> Result<i32> {
    let policy = match opts.policy {
        Some(file) => {
            load_policy(Path::new(file)).with_context(|| format!("failed to load policy {file}"))?
        }
        None => builtin_repo_hygiene(),
    };

    let facts = collect(Path::new(opts.path))
        .with_context(|| format!("failed to collect facts from {}", opts.path))?;

    let evaluation = evaluate(&policy, &facts);

    let threshold = parse_severity(opts.severity)?;

    if !opts.quiet {
        match opts.format {
            "json" => println!("{}", serde_json::to_string_pretty(&evaluation)?),
            "sarif" => println!(
                "{}",
                serde_json::to_string_pretty(&report::to_sarif(&evaluation, opts.verbose))?
            ),
            _ => print!("{}", report::to_human(&evaluation, opts.verbose)),
        }
    }

    Ok(if evaluation.clean_at(threshold) {
        0
    } else {
        EXIT_VIOLATIONS
    })
}

fn parse_severity(s: &str) -> Result<Severity> {
    match s {
        "error" => Ok(Severity::Error),
        "warning" => Ok(Severity::Warning),
        "info" => Ok(Severity::Info),
        other => anyhow::bail!("unknown severity {other:?}: use error, warning, or info"),
    }
}
