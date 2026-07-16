// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// End-to-end tests for `plasma diff`: the snapshot → mutate → snapshot →
// diff verification protocol, with diff(1) exit semantics.

use std::fs;
use std::process::Command;

fn plasma() -> Command {
    Command::new(env!("CARGO_BIN_EXE_plasma"))
}

fn snapshot(repo: &std::path::Path, out: &std::path::Path) {
    let output = plasma()
        .args(["facts", repo.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    fs::write(out, &output.stdout).unwrap();
}

#[test]
fn identical_snapshots_exit_zero() {
    let repo = tempfile::tempdir().unwrap();
    fs::write(repo.path().join("a.rs"), "fn main() {}\n").unwrap();
    let dir = tempfile::tempdir().unwrap();
    let before = dir.path().join("before.json");
    let after = dir.path().join("after.json");
    snapshot(repo.path(), &before);
    snapshot(repo.path(), &after);

    let out = plasma()
        .args(["diff", before.to_str().unwrap(), after.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&out.stdout).contains("identical"));
}

#[test]
fn agent_run_shows_up_in_diff() {
    let repo = tempfile::tempdir().unwrap();
    fs::write(repo.path().join("a.rs"), "fn main() {}\n").unwrap();
    let dir = tempfile::tempdir().unwrap();
    let before = dir.path().join("before.json");
    let after = dir.path().join("after.json");
    snapshot(repo.path(), &before);

    // Simulate an agent: fix headers via plasma itself, and add a file.
    let fix = plasma()
        .args([
            "fix",
            repo.path().to_str().unwrap(),
            "--apply",
            "--no-backup",
        ])
        .output()
        .unwrap();
    assert!(fix.status.success());
    fs::write(repo.path().join("extra.txt"), "hello\n").unwrap();

    snapshot(repo.path(), &after);

    let out = plasma()
        .args([
            "diff",
            before.to_str().unwrap(),
            after.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    // Snapshots differ → exit 1, per diff(1) semantics.
    assert_eq!(out.status.code(), Some(1));

    let d: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let added: Vec<&str> = d["files_added"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(added.contains(&"extra.txt"));
    assert!(d["files_removed"].as_array().unwrap().is_empty());

    let headers = d["headers_changed"].as_array().unwrap();
    assert_eq!(headers.len(), 1);
    assert_eq!(headers[0]["file"], "a.rs");
    assert!(headers[0]["before"].is_null());
    assert_eq!(headers[0]["after"], "MPL-2.0");
}

#[test]
fn diff_json_is_deterministic() {
    let repo = tempfile::tempdir().unwrap();
    fs::write(repo.path().join("a.rs"), "fn main() {}\n").unwrap();
    let dir = tempfile::tempdir().unwrap();
    let before = dir.path().join("before.json");
    let after = dir.path().join("after.json");
    snapshot(repo.path(), &before);
    fs::write(
        repo.path().join("b.rs"),
        "// SPDX-License-Identifier: MIT\n",
    )
    .unwrap();
    snapshot(repo.path(), &after);

    let run = || {
        plasma()
            .args([
                "diff",
                before.to_str().unwrap(),
                after.to_str().unwrap(),
                "--format",
                "json",
            ])
            .output()
            .unwrap()
            .stdout
    };
    assert_eq!(run(), run());
}

#[test]
fn garbage_snapshot_is_a_usage_error() {
    let dir = tempfile::tempdir().unwrap();
    let bad = dir.path().join("bad.json");
    fs::write(&bad, "not json").unwrap();
    let out = plasma()
        .args(["diff", bad.to_str().unwrap(), bad.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert_ne!(out.status.code(), Some(1)); // error, not "differ"
}
