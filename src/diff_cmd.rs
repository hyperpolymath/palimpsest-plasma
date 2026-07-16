// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// plasma diff — compare two fact snapshots.
//
// The before/after contract for agent verification: snapshot with
// `plasma facts > before.json`, let the agent run, snapshot again, then
// `plasma diff before.json after.json` shows exactly what changed.
// Exit semantics follow diff(1): 0 = identical, 1 = differ, 2 = error.

use anyhow::{Context, Result};
use plasma_engine::{diff, FactSet, FactsDiff};
use std::path::Path;

/// Exit code when the snapshots differ.
pub const EXIT_DIFFERENT: i32 = 1;

pub fn run(before_file: &str, after_file: &str, format: &str) -> Result<i32> {
    let before = load_snapshot(before_file)?;
    let after = load_snapshot(after_file)?;

    let d = diff(&before, &after);

    match format {
        "json" => println!("{}", serde_json::to_string_pretty(&d)?),
        _ => print!("{}", render_human(&d)),
    }

    Ok(if d.is_empty() { 0 } else { EXIT_DIFFERENT })
}

fn load_snapshot(file: &str) -> Result<FactSet> {
    let content = std::fs::read_to_string(Path::new(file))
        .with_context(|| format!("cannot read snapshot {file}"))?;
    serde_json::from_str(&content).with_context(|| {
        format!("{file} is not a valid facts snapshot (produce one with `plasma facts`)")
    })
}

fn render_human(d: &FactsDiff) -> String {
    let mut out = String::new();

    if d.is_empty() {
        out.push_str("Snapshots are identical.\n");
        return out;
    }

    out.push_str(&format!(
        "Snapshots differ ({} change(s)):\n\n",
        d.change_count()
    ));

    for file in &d.files_added {
        out.push_str(&format!("  + added:   {file}\n"));
    }
    for file in &d.files_removed {
        out.push_str(&format!("  - removed: {file}\n"));
    }
    for h in &d.headers_changed {
        out.push_str(&format!(
            "  ~ header:  {} ({} -> {})\n",
            h.file,
            h.before.as_deref().unwrap_or("<none>"),
            h.after.as_deref().unwrap_or("<none>")
        ));
    }
    for m in &d.metadata_changed {
        out.push_str(&format!(
            "  ~ metadata: {} ({} -> {})\n",
            m.key,
            m.before.as_deref().unwrap_or("<absent>"),
            m.after.as_deref().unwrap_or("<absent>")
        ));
    }
    if let Some(git) = &d.git_changed {
        out.push_str(&format!(
            "  ~ git:     {} -> {}\n",
            git.before.head_ref.as_deref().unwrap_or("<not a repo>"),
            git.after.head_ref.as_deref().unwrap_or("<not a repo>")
        ));
    }

    out
}
