// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
#![forbid(unsafe_code)]
//
// plasma — deterministic, typed policy engine and license tooling.
//
//   plasma check    — evaluate a repository against a policy
//   plasma facts    — dump the deterministic fact snapshot the evaluator sees
//   plasma policy   — validate policy files
//   plasma audit    — check all files for correct SPDX headers
//   plasma init     — add a LICENSE and SPDX headers to a project
//   plasma badge    — generate shields.io badge markup
//   plasma migrate  — convert a project from one license to another

use anyhow::Result;
use clap::{Parser, Subcommand};

mod audit;
mod badge;
mod check;
mod diff_cmd;
mod facts_cmd;
mod fix;
mod init;
mod migrate;
mod policy_cmd;

/// plasma — deterministic, typed policy engine and license tooling
#[derive(Parser)]
#[command(name = "plasma", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Evaluate a repository against a policy.
    Check {
        /// Path to the repository root
        #[arg(default_value = ".")]
        path: String,
        /// Policy file (.toml or .json); defaults to the bundled repo-hygiene policy
        #[arg(long)]
        policy: Option<String>,
        /// Output format: human, json, or sarif
        #[arg(long, default_value = "human")]
        format: String,
        /// Minimum violation severity that fails the check: error, warning, info
        #[arg(long, default_value = "error")]
        severity: String,
        /// Exit code only, no output
        #[arg(long)]
        quiet: bool,
        /// Include satisfied (pass) findings in the output
        #[arg(long)]
        verbose: bool,
    },

    /// Plan or apply corrective actions for policy violations.
    Fix {
        /// Path to the repository root
        #[arg(default_value = ".")]
        path: String,
        /// Policy file (.toml or .json); defaults to the bundled repo-hygiene policy
        #[arg(long)]
        policy: Option<String>,
        /// SPDX license identifier written into any headers added
        #[arg(long, default_value = "MPL-2.0")]
        license: String,
        /// Author line for any headers added
        #[arg(long)]
        author: Option<String>,
        /// Actually make the changes (default is a dry-run plan)
        #[arg(long)]
        apply: bool,
        /// Do not write .bak backups before modifying files
        #[arg(long)]
        no_backup: bool,
        /// Output format: human or json
        #[arg(long, default_value = "human")]
        format: String,
    },

    /// Dump the deterministic fact snapshot for a repository (JSON).
    Facts {
        /// Path to the repository root
        #[arg(default_value = ".")]
        path: String,
        /// Also include text-file contents (for content-aware policies)
        #[arg(long)]
        with_contents: bool,
    },

    /// Compare two fact snapshots (exit 0 identical, 1 differ).
    Diff {
        /// Snapshot taken before (JSON from `plasma facts`)
        before: String,
        /// Snapshot taken after (JSON from `plasma facts`)
        after: String,
        /// Output format: human or json
        #[arg(long, default_value = "human")]
        format: String,
    },

    /// Policy file utilities.
    Policy {
        #[command(subcommand)]
        command: PolicyCommands,
    },

    /// Audit all source files for correct SPDX headers.
    Audit {
        /// Path to audit (default: current directory)
        #[arg(default_value = ".")]
        path: String,
        /// SPDX license expression headers are expected to carry
        #[arg(short, long, default_value = "MPL-2.0")]
        license: String,
        /// Fix missing headers automatically
        #[arg(long)]
        fix: bool,
    },

    /// Add a license and SPDX headers to a project.
    Init {
        /// Path to the project root (default: current directory)
        #[arg(short, long, default_value = ".")]
        path: String,
        /// SPDX license identifier to apply
        #[arg(short, long, default_value = "MPL-2.0")]
        license: String,
        /// Author name for SPDX-FileCopyrightText
        #[arg(short, long)]
        author: Option<String>,
    },

    /// Generate a shields.io badge for the project.
    Badge {
        /// SPDX license identifier for the badge
        #[arg(short, long, default_value = "MPL-2.0")]
        license: String,
        /// Badge style: flat, flat-square, for-the-badge, plastic
        #[arg(short, long, default_value = "flat")]
        style: String,
    },

    /// Migrate from one license to another.
    Migrate {
        /// Path to the project root
        #[arg(short, long, default_value = ".")]
        path: String,
        /// Current license to migrate from (auto-detected if omitted)
        #[arg(short, long)]
        from: Option<String>,
        /// Target SPDX license identifier
        #[arg(short, long, default_value = "MPL-2.0")]
        to: String,
    },
}

#[derive(Subcommand)]
enum PolicyCommands {
    /// Parse and schema-check a policy file.
    Validate {
        /// Policy file (.toml or .json)
        file: String,
    },
}

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        // Usage/IO errors exit 2, keeping exit 1 unambiguous ("violations
        // found" / "snapshots differ") per docs/cli-design.adoc.
        eprintln!("error: {e:#}");
        std::process::exit(2);
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Check {
            path,
            policy,
            format,
            severity,
            quiet,
            verbose,
        } => {
            let code = check::run(&check::CheckOptions {
                path: &path,
                policy: policy.as_deref(),
                format: &format,
                severity: &severity,
                quiet,
                verbose,
            })?;
            if code != 0 {
                std::process::exit(code);
            }
        }
        Commands::Fix {
            path,
            policy,
            license,
            author,
            apply,
            no_backup,
            format,
        } => {
            let code = fix::run(&fix::FixOptions {
                path: &path,
                policy: policy.as_deref(),
                license: &license,
                author: author.as_deref(),
                apply_changes: apply,
                backup: !no_backup,
                format: &format,
            })?;
            if code != 0 {
                std::process::exit(code);
            }
        }
        Commands::Facts {
            path,
            with_contents,
        } => {
            facts_cmd::run(&path, with_contents)?;
        }
        Commands::Diff {
            before,
            after,
            format,
        } => {
            let code = diff_cmd::run(&before, &after, &format)?;
            if code != 0 {
                std::process::exit(code);
            }
        }
        Commands::Policy { command } => match command {
            PolicyCommands::Validate { file } => {
                policy_cmd::validate(&file)?;
            }
        },
        Commands::Audit { path, license, fix } => {
            audit::run(&path, &license, fix)?;
        }
        Commands::Init {
            path,
            license,
            author,
        } => {
            init::run(&path, &license, author.as_deref())?;
        }
        Commands::Badge { license, style } => {
            badge::run(&license, &style)?;
        }
        Commands::Migrate { path, from, to } => {
            migrate::run(&path, from.as_deref(), &to)?;
        }
    }

    Ok(())
}
