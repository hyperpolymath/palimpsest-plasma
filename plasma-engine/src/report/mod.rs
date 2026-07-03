// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Report rendering — human text and SARIF views of an Evaluation.
// (JSON is served by Evaluation's Serialize implementation.)

pub mod human;
pub mod sarif;

pub use human::to_human;
pub use sarif::to_sarif;
