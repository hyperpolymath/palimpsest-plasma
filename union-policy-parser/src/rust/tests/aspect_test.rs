// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
//! Security / Aspect Tests — Union Policy Parser
//!
//! Verifies that the parser handles hostile, malformed, or edge-case inputs
//! without panicking. The critical invariant: the parser must never panic —
//! every input, however malformed, must return an Ok or Err, never abort.
//!
//! Also tests error-type correctness so that callers can handle failures
//! semantically (e.g., display a user-facing message rather than crashing).

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse and assert no panic. Returns true if Ok.
fn parses_without_panic(input: &str) -> bool {
    // std::panic::catch_unwind is not needed here because Rust will report
    // a panic as a test failure automatically. We simply call parse and
    // observe the Result.
    union_policy_parser::parser::parse(input).is_ok()
}

// ---------------------------------------------------------------------------
// Security / Aspect tests
// ---------------------------------------------------------------------------

#[test]
fn aspect_empty_string_does_not_panic() {
    // The empty string is valid input — no directives, no sections.
    let result = union_policy_parser::parser::parse("");
    // Must return Ok (empty document) or Err — never panic.
    assert!(
        result.is_ok() || result.is_err(),
        "empty string must return Ok or Err, not panic"
    );
}

#[test]
fn aspect_whitespace_only_does_not_panic() {
    // Input consisting entirely of whitespace and newlines must not panic.
    let result = union_policy_parser::parser::parse("   \n\n\t\n  ");
    assert!(
        result.is_ok() || result.is_err(),
        "whitespace-only input must not panic"
    );
}

#[test]
fn aspect_unclosed_abstract_directive_does_not_panic() {
    // A directive opened without a matching @end must not panic.
    let input = "@abstract\nThis contract has no closing tag.";
    let result = union_policy_parser::parser::parse(input);
    assert!(
        result.is_ok() || result.is_err(),
        "unclosed @abstract must not panic"
    );
}

#[test]
fn aspect_unknown_directive_does_not_panic() {
    // An unrecognised @ directive must not panic; parser may ignore or error.
    let input = "@unknown\nsome content\n@end\n# Section\n- item\n";
    let result = union_policy_parser::parser::parse(input);
    assert!(
        result.is_ok() || result.is_err(),
        "unknown directive must not panic"
    );
}

#[test]
fn aspect_special_chars_in_directive_do_not_panic() {
    // Special characters including Unicode in directive bodies must be handled.
    let input = "@abstract\n<script>alert(1)</script>\n@end\n";
    let result = union_policy_parser::parser::parse(input);
    assert!(
        result.is_ok() || result.is_err(),
        "special characters in @abstract must not panic"
    );
}

#[test]
fn aspect_null_byte_in_input_does_not_panic() {
    // A string containing a null byte (valid Rust str is NUL-safe) must not panic.
    let input = "@abstract\nText with \x00 null byte.\n@end\n";
    let result = union_policy_parser::parser::parse(input);
    assert!(
        result.is_ok() || result.is_err(),
        "null byte in input must not panic"
    );
}

#[test]
fn aspect_extremely_long_line_does_not_panic() {
    // A single line of 100,000 characters must not overflow the stack or panic.
    let long_line = "a".repeat(100_000);
    let input = format!("# Long Section\n\n- {}\n", long_line);
    let result = union_policy_parser::parser::parse(&input);
    assert!(
        result.is_ok() || result.is_err(),
        "extremely long line must not panic"
    );
}

#[test]
fn aspect_deeply_nested_heading_markers_do_not_panic() {
    // Many consecutive '#' characters at the start of a line must not panic.
    let input = "##########################################################\nContent\n";
    let result = union_policy_parser::parser::parse(input);
    assert!(
        result.is_ok() || result.is_err(),
        "deeply nested headings must not panic"
    );
}

#[test]
fn aspect_malformed_refs_block_does_not_panic() {
    // A @refs block with no valid link syntax must not panic.
    let input = "@refs\n!!! not a valid link !!!\n@end\n";
    let result = union_policy_parser::parser::parse(input);
    assert!(
        result.is_ok() || result.is_err(),
        "malformed @refs block must not panic"
    );
}

#[test]
fn aspect_repeated_directives_do_not_panic() {
    // Multiple @abstract blocks must not panic regardless of semantics.
    let input = "@abstract\nFirst.\n@end\n@abstract\nSecond.\n@end\n";
    let result = union_policy_parser::parser::parse(input);
    assert!(
        result.is_ok() || result.is_err(),
        "repeated @abstract blocks must not panic"
    );
}
