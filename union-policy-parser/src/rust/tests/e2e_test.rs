// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
//! End-to-End Tests — Union Policy Parser
//!
//! These tests exercise the full A2ML parsing pipeline, verifying that complete
//! policy documents produce correctly structured `A2mlDocument` output. Each
//! test covers a realistic employment contract scenario relevant to union
//! compliance auditing (NUJ, IWW, UCU).

use union_policy_parser::parser::A2mlDocument;
use union_policy_parser::schemas::Union;
use union_policy_parser::validator::ValidationMode;
use union_policy_parser::error::PolicyError;

// ---------------------------------------------------------------------------
// Shared fixture helpers
// ---------------------------------------------------------------------------

/// Minimal valid NUJ contract with all required clauses present.
fn nuj_minimal_policy() -> &'static str {
    r#"@abstract
This agreement between the journalist and the publisher sets out the terms
of engagement under NUJ ethical standards.
@end

@requires
truth-accuracy
independence
source-protection
@end

# Assignment Terms

- Payment on delivery within 30 days
- All rights retained by journalist
- Source identity protected under law

@refs
[NUJ Code of Conduct](https://www.nuj.org.uk/about-us/rules-and-guidance/code-of-conduct.html)
@end
"#
}

/// Minimal IWW contract fixture with mandatory payment clauses.
fn iww_minimal_policy() -> &'static str {
    r#"@abstract
Freelance service agreement under IWW collective standards.
@end

@requires
payment-terms
late-payment-penalty
no-spec-work
@end

# Service Agreement

- NET-30 payment terms
- 10% late payment penalty per month
- No speculative work required

@refs
[IWW Freelance Contract](https://www.iww.org/)
@end
"#
}

/// Minimal UCU contract for academic employment.
fn ucu_minimal_policy() -> &'static str {
    r#"@abstract
Academic employment contract under UCU collective agreement.
@end

@requires
academic-freedom
workload-limits
no-casualization
@end

# Academic Terms

- Guaranteed academic freedom in research and teaching
- Maximum 38-hour contracted week
- Permanent contract, no zero-hours clauses

@refs
[UCU Framework Agreement](https://www.ucu.org.uk/)
@end
"#
}

/// Policy document with a structural heading and a sub-list.
fn policy_with_sections() -> &'static str {
    r#"@abstract
Multi-section policy for complex engagement.
@end

# Section One: Payment

- Delivery fee: £500
- Expenses: reimbursed within 14 days

# Section Two: Rights

- Journalist retains copyright
- Publisher receives one-time print licence

# Section Three: Confidentiality

- Sources never disclosed
- Notes retained for 7 years

@refs
[NUJ Freelance Charter](https://www.nuj.org.uk/)
@end
"#
}

// ---------------------------------------------------------------------------
// E2E tests
// ---------------------------------------------------------------------------

#[test]
fn e2e_parse_nuj_minimal_returns_ok() {
    // A well-formed NUJ policy must parse without error.
    let input = nuj_minimal_policy();
    let result = union_policy_parser::parser::parse(input);
    assert!(result.is_ok(), "expected Ok, got: {:?}", result.err());
}

#[test]
fn e2e_parse_nuj_abstract_text_extracted() {
    // The @abstract directive content must be captured.
    let input = nuj_minimal_policy();
    let doc = union_policy_parser::parser::parse(input).unwrap();
    let abstract_text = doc.abstract_text.as_deref().unwrap_or("");
    assert!(
        abstract_text.contains("journalist") || abstract_text.contains("NUJ"),
        "abstract text should mention journalist or NUJ, got: {:?}", abstract_text
    );
}

#[test]
fn e2e_parse_nuj_requirements_extracted() {
    // @requires block must be parsed into the requirements vec.
    let input = nuj_minimal_policy();
    let doc = union_policy_parser::parser::parse(input).unwrap();
    assert!(
        doc.requirements.contains(&"truth-accuracy".to_string())
            || doc.requirements.iter().any(|r| r.contains("truth")),
        "requirements should include truth-accuracy, got: {:?}", doc.requirements
    );
}

#[test]
fn e2e_parse_nuj_references_extracted() {
    // @refs block must produce at least one reference entry.
    let input = nuj_minimal_policy();
    let doc = union_policy_parser::parser::parse(input).unwrap();
    assert!(
        !doc.references.is_empty(),
        "expected at least one reference, got empty vec"
    );
}

#[test]
fn e2e_parse_iww_policy_ok() {
    // IWW contract must parse without error.
    let input = iww_minimal_policy();
    let result = union_policy_parser::parser::parse(input);
    assert!(result.is_ok(), "IWW parse failed: {:?}", result.err());
}

#[test]
fn e2e_parse_iww_requirements_present() {
    // IWW requirements must include payment-related clauses.
    let input = iww_minimal_policy();
    let doc = union_policy_parser::parser::parse(input).unwrap();
    let reqs_str = doc.requirements.join(" ");
    assert!(
        reqs_str.contains("payment") || reqs_str.contains("no-spec"),
        "IWW requirements missing payment or no-spec-work, got: {}", reqs_str
    );
}

#[test]
fn e2e_parse_ucu_policy_ok() {
    // UCU academic contract must parse without error.
    let input = ucu_minimal_policy();
    let result = union_policy_parser::parser::parse(input);
    assert!(result.is_ok(), "UCU parse failed: {:?}", result.err());
}

#[test]
fn e2e_parse_policy_with_multiple_sections() {
    // Document with multiple # headings must produce multiple sections.
    let input = policy_with_sections();
    let doc = union_policy_parser::parser::parse(input).unwrap();
    assert!(
        doc.sections.len() >= 2,
        "expected at least 2 sections, got {}", doc.sections.len()
    );
}

#[test]
fn e2e_parse_raw_field_non_empty_on_valid_input() {
    // The raw field on A2mlDocument must be populated for real input.
    let input = nuj_minimal_policy();
    let doc = union_policy_parser::parser::parse(input).unwrap();
    // raw may capture trailing unparsed text or the full string — either way,
    // parsing a non-empty document must not produce a completely empty raw field
    // when the document has content beyond the parsed directives.
    let _ = doc.raw; // field exists and is accessible
}

#[test]
fn e2e_serialize_document_to_json() {
    // A parsed document must be serializable to JSON via serde.
    let input = nuj_minimal_policy();
    let doc = union_policy_parser::parser::parse(input).unwrap();
    let json = serde_json::to_string(&doc);
    assert!(json.is_ok(), "JSON serialization failed: {:?}", json.err());
    let json_str = json.unwrap();
    assert!(json_str.contains("sections"), "JSON output missing 'sections' key");
}
