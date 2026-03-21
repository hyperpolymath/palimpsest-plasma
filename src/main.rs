// SPDX-License-Identifier: PMPL-2.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// palimpsest-plasma — Palimpsest license adoption toolkit.
//
// One-command PMPL-2.0 adoption for any project:
//   plasma init     — add LICENSE, SPDX headers, Covenant
//   plasma audit    — check all files for correct SPDX headers
//   plasma badge    — generate shields.io badge markdown
//   plasma migrate  — convert from MIT/Apache/GPL to PMPL-2.0
//   plasma verify   — verify provenance chain integrity

use anyhow::Result;
use clap::{Parser, Subcommand};

mod audit;
mod badge;
mod init;
mod migrate;

/// palimpsest-plasma — adopt PMPL-2.0 in one command
#[derive(Parser)]
#[command(name = "plasma", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add PMPL-2.0 license, SPDX headers, and Covenant to a project.
    Init {
        /// Path to the project root (default: current directory)
        #[arg(short, long, default_value = ".")]
        path: String,
        /// Author name for SPDX-FileCopyrightText
        #[arg(short, long)]
        author: Option<String>,
        /// Also add the Palimpsest Community Covenant
        #[arg(long, default_value = "true")]
        covenant: bool,
    },

    /// Audit all source files for correct SPDX headers.
    Audit {
        /// Path to audit (default: current directory)
        #[arg(short, long, default_value = ".")]
        path: String,
        /// Fix missing headers automatically
        #[arg(long)]
        fix: bool,
    },

    /// Generate a shields.io badge for the project.
    Badge {
        /// Badge style: flat, flat-square, for-the-badge, plastic
        #[arg(short, long, default_value = "flat")]
        style: String,
    },

    /// Migrate from another license to PMPL-2.0.
    Migrate {
        /// Path to the project root
        #[arg(short, long, default_value = ".")]
        path: String,
        /// Current license to migrate from (auto-detected if omitted)
        #[arg(short, long)]
        from: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            path,
            author,
            covenant,
        } => {
            init::run(&path, author.as_deref(), covenant)?;
        }
        Commands::Audit { path, fix } => {
            audit::run(&path, fix)?;
        }
        Commands::Badge { style } => {
            badge::run(&style)?;
        }
        Commands::Migrate { path, from } => {
            migrate::run(&path, from.as_deref())?;
        }
    }

    Ok(())
}
