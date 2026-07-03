// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// plasma-engine — deterministic, typed policy evaluation.
//
// This crate provides:
// - A typed policy AST (deontic rules over repository subjects and resources)
// - Versioned policy schema loading (TOML and JSON)
// - Fact collection from a repository tree (the only impure module)
// - A pure, total evaluator: Policy × FactSet → Evaluation
// - Human, JSON, and SARIF report rendering
//
// Design contract (see docs/engine-v0-design.adoc): evaluation is
// deterministic and total — identical policy and facts produce identical
// findings in identical order, with no clocks, randomness, or ambient state.

#![forbid(unsafe_code)]

pub mod ast;
pub mod eval;
pub mod facts;
pub mod finding;
pub mod report;
pub mod schema;

pub use ast::{
    ActionKind, Condition, Modality, Overlay, Policy, PolicyVersion, Resource, Rule, Subject,
};
pub use eval::evaluate;
pub use facts::{collect, FactError, FactSet, GitFacts};
pub use finding::{Evaluation, Finding, FindingStatus, Severity, Summary};
pub use schema::{
    builtin_repo_hygiene, load_policy, load_policy_str, PolicyFormat, SchemaError, SCHEMA_VERSION,
};
