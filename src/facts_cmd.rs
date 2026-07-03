// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// plasma facts — dump the deterministic fact snapshot for a repository.
// This is the input the evaluator sees, and the snapshot claim-verification
// tooling can diff before/after an agent run.

use anyhow::{Context, Result};
use plasma_engine::collect;
use std::path::Path;

pub fn run(path: &str) -> Result<()> {
    let facts =
        collect(Path::new(path)).with_context(|| format!("failed to collect facts from {path}"))?;
    println!("{}", serde_json::to_string_pretty(&facts)?);
    Ok(())
}
