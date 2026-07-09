// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// End-to-end tests for `plasma fix`: dry-run reports, --apply mutates and
// backs up, and applying is idempotent.

use std::fs;
use std::process::Command;

fn plasma() -> Command {
    Command::new(env!("CARGO_BIN_EXE_plasma"))
}

/// A throwaway repo with one header-less source file, plus LICENSE + README
/// so only the header rule fires.
fn scratch_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("foo.rs"), "fn main() {}\n").unwrap();
    fs::write(dir.path().join("LICENSE"), "MPL-2.0\n").unwrap();
    fs::write(dir.path().join("README.md"), "# demo\n").unwrap();
    dir
}

#[test]
fn dry_run_reports_but_does_not_write() {
    let dir = scratch_repo();
    let out = plasma()
        .args(["fix", dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    // Pending work → exit 1, and the file is untouched.
    assert_eq!(out.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("add SPDX header"));
    assert_eq!(
        fs::read_to_string(dir.path().join("foo.rs")).unwrap(),
        "fn main() {}\n"
    );
    assert!(!dir.path().join("foo.rs.bak").exists());
}

#[test]
fn apply_adds_header_with_backup() {
    let dir = scratch_repo();
    let out = plasma()
        .args([
            "fix",
            dir.path().to_str().unwrap(),
            "--apply",
            "--author",
            "Jane <j@example.com>",
        ])
        .output()
        .unwrap();

    assert!(out.status.success());
    let foo = fs::read_to_string(dir.path().join("foo.rs")).unwrap();
    assert!(foo.starts_with("// SPDX-License-Identifier: MPL-2.0\n"));
    assert!(foo.contains("Copyright (c) 2026 Jane"));
    assert_eq!(
        fs::read_to_string(dir.path().join("foo.rs.bak")).unwrap(),
        "fn main() {}\n"
    );
}

#[test]
fn apply_is_idempotent() {
    let dir = scratch_repo();
    let run = || {
        plasma()
            .args(["fix", dir.path().to_str().unwrap(), "--apply"])
            .output()
            .unwrap()
    };
    assert!(run().status.success());
    // Second apply re-evaluates a now-clean repo: nothing to fix, exit 0,
    // and no double header on the file.
    let second = run();
    assert!(second.status.success());
    let stdout = String::from_utf8_lossy(&second.stdout);
    assert!(stdout.contains("Nothing to fix."));
    let foo = fs::read_to_string(dir.path().join("foo.rs")).unwrap();
    assert_eq!(foo.matches("SPDX-License-Identifier").count(), 1);
}

#[test]
fn json_format_emits_plan() {
    let dir = scratch_repo();
    let out = plasma()
        .args(["fix", dir.path().to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(doc["plan"]["actions"].as_array().unwrap().len() == 1);
    assert!(doc["outcome"].is_null()); // dry-run: no outcome
}
