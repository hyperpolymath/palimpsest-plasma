// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// plasma fix — plan or apply corrective actions for policy violations.
//
// Dry-run by default: it prints what it *would* do. With --apply it makes
// the changes, backing up each modified file first (unless --no-backup).

use anyhow::{Context, Result};
use plasma_engine::{
    apply, builtin_repo_hygiene, collect_opts, evaluate, load_policy, plan, Action, ApplyOptions,
    CollectOptions, FixContext,
};
use std::path::Path;

/// Exit code when actions remain unapplied (dry-run) or an apply failed.
pub const EXIT_PENDING: i32 = 1;

pub struct FixOptions<'a> {
    pub path: &'a str,
    pub policy: Option<&'a str>,
    pub license: &'a str,
    pub author: Option<&'a str>,
    pub apply_changes: bool,
    pub backup: bool,
    pub format: &'a str,
}

pub fn run(opts: &FixOptions) -> Result<i32> {
    let policy = match opts.policy {
        Some(file) => {
            load_policy(Path::new(file)).with_context(|| format!("failed to load policy {file}"))?
        }
        None => builtin_repo_hygiene(),
    };

    let facts = collect_opts(
        Path::new(opts.path),
        &CollectOptions {
            contents: policy.needs_content(),
        },
    )
    .with_context(|| format!("failed to collect facts from {}", opts.path))?;
    let evaluation = evaluate(&policy, &facts);

    let ctx = FixContext {
        license: opts.license.to_string(),
        author: opts
            .author
            .unwrap_or("CHANGE-ME <your@email.com>")
            .to_string(),
        year: "2026".to_string(),
    };
    let remediation = plan(&policy, &evaluation, &ctx);

    if opts.format == "json" {
        // In JSON mode, emit the plan (and outcome if applied) as one object.
        let outcome = if opts.apply_changes {
            Some(apply(
                &remediation,
                Path::new(opts.path),
                &ApplyOptions {
                    backup: opts.backup,
                },
            ))
        } else {
            None
        };
        let doc = serde_json::json!({
            "plan": remediation,
            "outcome": outcome,
        });
        println!("{}", serde_json::to_string_pretty(&doc)?);
        return Ok(exit_code(
            opts.apply_changes,
            &remediation,
            outcome.as_ref(),
        ));
    }

    // Human output.
    if remediation.is_empty() {
        println!("Nothing to fix.");
        return Ok(0);
    }

    if !opts.apply_changes {
        println!("Plan (dry-run — re-run with --apply to make changes):\n");
        for planned in &remediation.actions {
            println!("  {}", describe_action(&planned.action));
        }
        if !remediation.manual.is_empty() {
            println!("\nManual (no automatic fix):");
            for item in &remediation.manual {
                println!("  [{}] {} — {}", item.rule_id, item.subject, item.reason);
            }
        }
        println!(
            "\n{} action(s) to apply, {} manual item(s).",
            remediation.actions.len(),
            remediation.manual.len()
        );
        return Ok(exit_code(false, &remediation, None));
    }

    let outcome = apply(
        &remediation,
        Path::new(opts.path),
        &ApplyOptions {
            backup: opts.backup,
        },
    );
    for file in &outcome.applied {
        println!("  APPLIED: {file}");
    }
    for (file, reason) in &outcome.skipped {
        println!("  SKIPPED: {file} ({reason})");
    }
    for (file, err) in &outcome.errors {
        println!("  ERROR:   {file} ({err})");
    }
    if !remediation.manual.is_empty() {
        println!("\nManual (no automatic fix):");
        for item in &remediation.manual {
            println!("  [{}] {} — {}", item.rule_id, item.subject, item.reason);
        }
    }
    println!(
        "\n{} applied, {} skipped, {} error(s), {} manual.",
        outcome.applied.len(),
        outcome.skipped.len(),
        outcome.errors.len(),
        remediation.manual.len()
    );

    Ok(exit_code(true, &remediation, Some(&outcome)))
}

fn describe_action(action: &Action) -> String {
    match action {
        Action::AddSpdxHeader { file, license, .. } => {
            format!("add SPDX header ({license}) to {file}")
        }
        Action::CreateFile { path, .. } => {
            format!("create {path} (placeholder content)")
        }
    }
}

/// Exit non-zero when the working tree is not clean after the command:
/// in dry-run, if there is anything to do; in apply, if any action errored.
fn exit_code(
    applied: bool,
    remediation: &plasma_engine::Plan,
    outcome: Option<&plasma_engine::ApplyOutcome>,
) -> i32 {
    if applied {
        match outcome {
            Some(o) if o.is_clean() => 0,
            Some(_) => EXIT_PENDING,
            None => 0,
        }
    } else if remediation.is_empty() {
        0
    } else {
        EXIT_PENDING
    }
}
