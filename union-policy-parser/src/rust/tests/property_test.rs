// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
//! Property Tests — Union Policy Parser
//!
//! Each test represents a distinct policy input variant and verifies that:
//! 1. Valid input parses successfully (no panic, Ok result).
//! 2. Parsing is deterministic (same input → same output on repeated calls).
//! 3. The output structure is internally consistent.
//!
//! Eight input variants are exercised, with multiple assertions per variant.

// ---------------------------------------------------------------------------
// Variant helpers
// ---------------------------------------------------------------------------

/// Variant A: abstract only — no requirements, no sections, no refs.
fn variant_abstract_only() -> &'static str {
    "@abstract\nBasic policy statement.\n@end\n"
}

/// Variant B: requirements only — no abstract, no sections.
fn variant_requires_only() -> &'static str {
    "@requires\nsource-protection\ntruth-accuracy\n@end\n"
}

/// Variant C: single heading section, no directives.
fn variant_single_section() -> &'static str {
    "# Payment Terms\n\n- NET-30 payment\n- No spec work\n"
}

/// Variant D: refs only — minimal reference block.
fn variant_refs_only() -> &'static str {
    "@refs\n[NUJ Code](https://nuj.org.uk/)\n@end\n"
}

/// Variant E: full NUJ document with all blocks present.
fn variant_full_nuj() -> &'static str {
    r#"@abstract
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

@refs
[NUJ Code of Conduct](https://www.nuj.org.uk/about-us/rules-and-guidance/code-of-conduct.html)
@end
"#
}

/// Variant F: document with special ASCII characters in content.
fn variant_special_characters() -> &'static str {
    "@abstract\nPolicy: fees & costs — see §3(a)(i).\n@end\n# Terms\n- Rate: £500/day\n"
}

/// Variant G: multi-paragraph abstract block.
fn variant_multi_paragraph_abstract() -> &'static str {
    r#"@abstract
This agreement is entered into by the parties below.

It supersedes all prior arrangements.
It is governed by the laws of England and Wales.
@end

# Definitions

- "Publisher" means the company named above
"#
}

/// Variant H: deeply nested content with multiple list items.
fn variant_many_list_items() -> &'static str {
    r#"# Deliverables

- Article: 800 words
- Photographs: up to 12
- Captions: per photo
- Audio clips: optional
- Video: excluded
- Revisions: 1 included
- Expenses: pre-approved only
- Deadlines: as agreed in writing
"#
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

#[test]
fn property_abstract_only_parses_ok() {
    let result = union_policy_parser::parser::parse(variant_abstract_only());
    assert!(result.is_ok(), "abstract-only variant failed: {:?}", result.err());
}

#[test]
fn property_abstract_only_has_abstract_text() {
    let doc = union_policy_parser::parser::parse(variant_abstract_only()).unwrap();
    assert!(doc.abstract_text.is_some(), "abstract_text must be Some for abstract-only variant");
    assert!(
        !doc.abstract_text.as_deref().unwrap_or("").is_empty(),
        "abstract_text must not be empty"
    );
}

#[test]
fn property_requires_only_parses_ok() {
    let result = union_policy_parser::parser::parse(variant_requires_only());
    assert!(result.is_ok(), "requires-only variant failed: {:?}", result.err());
}

#[test]
fn property_requires_only_has_requirements() {
    let doc = union_policy_parser::parser::parse(variant_requires_only()).unwrap();
    assert!(
        !doc.requirements.is_empty(),
        "requirements must be non-empty for requires-only variant, got {:?}", doc.requirements
    );
}

#[test]
fn property_single_section_parses_ok() {
    let result = union_policy_parser::parser::parse(variant_single_section());
    assert!(result.is_ok(), "single-section variant failed: {:?}", result.err());
}

#[test]
fn property_refs_only_parses_ok() {
    let result = union_policy_parser::parser::parse(variant_refs_only());
    assert!(result.is_ok(), "refs-only variant failed: {:?}", result.err());
}

#[test]
fn property_full_nuj_parses_ok() {
    let result = union_policy_parser::parser::parse(variant_full_nuj());
    assert!(result.is_ok(), "full NUJ variant failed: {:?}", result.err());
}

#[test]
fn property_full_nuj_sections_non_empty() {
    let doc = union_policy_parser::parser::parse(variant_full_nuj()).unwrap();
    assert!(
        !doc.sections.is_empty(),
        "full NUJ document must have at least one section"
    );
}

#[test]
fn property_special_characters_parses_ok() {
    let result = union_policy_parser::parser::parse(variant_special_characters());
    assert!(result.is_ok(), "special-characters variant failed: {:?}", result.err());
}

#[test]
fn property_special_characters_no_panic() {
    // Calling parse twice on the same input must produce identical Ok/Err status.
    let r1 = union_policy_parser::parser::parse(variant_special_characters()).is_ok();
    let r2 = union_policy_parser::parser::parse(variant_special_characters()).is_ok();
    assert_eq!(r1, r2, "parse result must be deterministic for special-characters variant");
}

#[test]
fn property_multi_paragraph_abstract_parses_ok() {
    let result = union_policy_parser::parser::parse(variant_multi_paragraph_abstract());
    assert!(result.is_ok(), "multi-paragraph abstract variant failed: {:?}", result.err());
}

#[test]
fn property_many_list_items_parses_ok() {
    let result = union_policy_parser::parser::parse(variant_many_list_items());
    assert!(result.is_ok(), "many-list-items variant failed: {:?}", result.err());
}

#[test]
fn property_determinism_full_nuj() {
    // Parsing the same input twice must yield the same section count.
    let doc1 = union_policy_parser::parser::parse(variant_full_nuj()).unwrap();
    let doc2 = union_policy_parser::parser::parse(variant_full_nuj()).unwrap();
    assert_eq!(
        doc1.sections.len(), doc2.sections.len(),
        "section count must be deterministic"
    );
    assert_eq!(
        doc1.requirements.len(), doc2.requirements.len(),
        "requirements count must be deterministic"
    );
}
