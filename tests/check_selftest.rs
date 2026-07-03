// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Self-test: `plasma check` against this very repository must pass, and
// its JSON output must be deterministic across runs.

use std::process::Command;

fn plasma() -> Command {
    Command::new(env!("CARGO_BIN_EXE_plasma"))
}

fn repo_root() -> &'static str {
    env!("CARGO_MANIFEST_DIR")
}

#[test]
fn check_self_passes() {
    let output = plasma()
        .args(["check", repo_root(), "--format", "json"])
        .output()
        .expect("plasma check must run");

    assert!(
        output.status.success(),
        "plasma check on this repo must exit 0; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let evaluation: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("check --format json must emit JSON");
    assert_eq!(evaluation["policy_id"], "repo-hygiene");
    assert_eq!(evaluation["summary"]["errors"], 0);
}

#[test]
fn check_json_is_deterministic() {
    let run = || {
        plasma()
            .args(["check", repo_root(), "--format", "json"])
            .output()
            .expect("plasma check must run")
            .stdout
    };
    assert_eq!(run(), run(), "identical runs must produce identical JSON");
}

#[test]
fn facts_emits_valid_json() {
    let output = plasma()
        .args(["facts", repo_root()])
        .output()
        .expect("plasma facts must run");
    assert!(output.status.success());
    let facts: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(facts["files"].as_array().is_some_and(|a| !a.is_empty()));
}

#[test]
fn policy_validate_accepts_bundled_policy() {
    let policy_path = format!(
        "{}/plasma-engine/policies/repo-hygiene.plasma.toml",
        repo_root()
    );
    let output = plasma()
        .args(["policy", "validate", &policy_path])
        .output()
        .expect("plasma policy validate must run");
    assert!(
        output.status.success(),
        "bundled policy must validate; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
