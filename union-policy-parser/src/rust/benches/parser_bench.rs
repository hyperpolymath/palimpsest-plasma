// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
//! Criterion Benchmarks — Union Policy Parser
//!
//! Measures the throughput of the A2ML parser across representative
//! workloads: a minimal policy document, a full multi-section contract,
//! and a large synthetic document constructed from 100 repeated sections.
//!
//! Run with: `cargo bench --bench parser_bench`

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

// ---------------------------------------------------------------------------
// Benchmark fixtures
// ---------------------------------------------------------------------------

/// Minimal A2ML policy: @abstract block only. Represents the fastest-path case.
const MINIMAL_POLICY: &str = r#"@abstract
A minimal freelance engagement policy.
@end
"#;

/// Full NUJ contract with all directive types and multiple sections.
/// Representative of a typical real-world input.
const FULL_NUJ_POLICY: &str = r#"@abstract
Full NUJ compliant freelance contract.
@end

@requires
truth-accuracy
independence
source-protection
@end

# Engagement Terms

- Flat fee on acceptance
- No rights transfer
- Source identity protected under law

# Payment Terms

- NET-30 payment
- Late payment penalty: 10% per month compounded
- No speculative work required

# Confidentiality

- Sources never disclosed to third parties
- Notes retained by journalist for 7 years
- Publisher may not compel disclosure

@refs
[NUJ Code of Conduct](https://www.nuj.org.uk/about-us/rules-and-guidance/code-of-conduct.html)
[NUJ Freelance Charter](https://www.nuj.org.uk/freelance/)
@end
"#;

/// Large synthetic policy: 100 sections, stress-tests section parsing throughput.
fn large_policy() -> String {
    let header = "@abstract\nSynthetic stress-test policy.\n@end\n\n@requires\ntruth-accuracy\n@end\n\n";
    let mut doc = String::from(header);
    for i in 0..100 {
        doc.push_str(&format!(
            "# Section {i}: Terms and Conditions\n\n- Clause {i}.1: standard terms apply\n- Clause {i}.2: specific obligations\n- Clause {i}.3: governing law: England and Wales\n\n"
        ));
    }
    doc.push_str("@refs\n[Reference](https://example.com/)\n@end\n");
    doc
}

// ---------------------------------------------------------------------------
// Benchmark group: minimal input
// ---------------------------------------------------------------------------

fn bench_minimal_parse(c: &mut Criterion) {
    c.bench_function("parse_minimal_policy", |b| {
        b.iter(|| {
            let _ = union_policy_parser::parser::parse(black_box(MINIMAL_POLICY));
        });
    });
}

// ---------------------------------------------------------------------------
// Benchmark group: full NUJ document
// ---------------------------------------------------------------------------

fn bench_full_nuj_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_nuj");
    group.throughput(Throughput::Bytes(FULL_NUJ_POLICY.len() as u64));

    group.bench_function("parse_full_nuj_policy", |b| {
        b.iter(|| {
            let _ = union_policy_parser::parser::parse(black_box(FULL_NUJ_POLICY));
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark group: large synthetic document
// ---------------------------------------------------------------------------

fn bench_large_document_parse(c: &mut Criterion) {
    let doc = large_policy();
    let byte_len = doc.len() as u64;

    let mut group = c.benchmark_group("large_document");
    group.throughput(Throughput::Bytes(byte_len));

    group.bench_function("parse_100_section_policy", |b| {
        b.iter(|| {
            let _ = union_policy_parser::parser::parse(black_box(doc.as_str()));
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Criterion registration
// ---------------------------------------------------------------------------

criterion_group!(
    benches,
    bench_minimal_parse,
    bench_full_nuj_parse,
    bench_large_document_parse
);
criterion_main!(benches);
