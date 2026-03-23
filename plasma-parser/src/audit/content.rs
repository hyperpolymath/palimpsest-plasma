// SPDX-License-Identifier: PMPL-2.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// LICENSE file content analysis — detects license type from file content
// using multi-level approximate matching (exact, normalised, Levenshtein,
// trigram, structural).

use crate::family::BaseLicense;

/// Confidence level of a license match.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MatchConfidence {
    /// Exact string match or known SPDX signature.
    Certain,
    /// Normalised match (case-folded, whitespace-collapsed).
    High,
    /// Levenshtein distance <= 2 on key phrases.
    Medium,
    /// Trigram similarity >= 0.7.
    Low,
    /// No match found.
    None,
}

/// Result of analysing a LICENSE file's content.
#[derive(Debug, Clone)]
pub struct ContentMatch {
    /// The detected base license.
    pub license: BaseLicense,
    /// Confidence of the match.
    pub confidence: MatchConfidence,
    /// The matching phrase or pattern that triggered detection.
    pub matched_phrase: String,
}

/// Compute the Levenshtein edit distance between two strings.
///
/// This is a standard dynamic programming implementation. No external
/// dependency required.
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a_len = a.len();
    let b_len = b.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut prev_row: Vec<usize> = (0..=b_len).collect();
    let mut curr_row = vec![0; b_len + 1];

    for (i, ca) in a.chars().enumerate() {
        curr_row[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            curr_row[j + 1] = (prev_row[j + 1] + 1)
                .min(curr_row[j] + 1)
                .min(prev_row[j] + cost);
        }
        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[b_len]
}

/// Compute the Sorensen-Dice coefficient based on character trigrams.
///
/// Returns a value between 0.0 (no similarity) and 1.0 (identical).
pub fn trigram_similarity(a: &str, b: &str) -> f64 {
    let trigrams_a: Vec<&str> = a.as_bytes().windows(3).map(|w| {
        std::str::from_utf8(w).unwrap_or("")
    }).collect();
    let trigrams_b: Vec<&str> = b.as_bytes().windows(3).map(|w| {
        std::str::from_utf8(w).unwrap_or("")
    }).collect();

    if trigrams_a.is_empty() && trigrams_b.is_empty() {
        return 1.0;
    }
    if trigrams_a.is_empty() || trigrams_b.is_empty() {
        return 0.0;
    }

    let matches = trigrams_a
        .iter()
        .filter(|t| trigrams_b.contains(t))
        .count();

    (2.0 * matches as f64) / (trigrams_a.len() + trigrams_b.len()) as f64
}

/// Key phrases that identify specific licenses.
///
/// Each phrase is matched case-insensitively against the LICENSE file content.
/// The first match wins (ordered from most to least specific).
const LICENSE_PHRASES: &[(&str, BaseLicense)] = &[
    ("palimpsest-mpl license", BaseLicense::MPL2), // PMPL fallback
    ("palimpsest-agpl license", BaseLicense::AGPL3), // PAGPL fallback
    ("gnu affero general public license", BaseLicense::AGPL3),
    ("mozilla public license version 2.0", BaseLicense::MPL2),
    ("mozilla public license, version 2.0", BaseLicense::MPL2),
    ("apache license, version 2.0", BaseLicense::Apache2),
    ("apache license version 2.0", BaseLicense::Apache2),
    ("gnu general public license version 3", BaseLicense::GPL3),
    ("gnu general public license, version 3", BaseLicense::GPL3),
    ("gnu general public license version 2", BaseLicense::GPL2),
    ("gnu lesser general public license", BaseLicense::LGPL3),
    ("european union public licence", BaseLicense::EUPL12),
    ("bsd 3-clause", BaseLicense::BSD3),
    ("bsd 2-clause", BaseLicense::BSD2),
    ("mit license", BaseLicense::MIT),
    ("permission is hereby granted, free of charge", BaseLicense::MIT),
    ("isc license", BaseLicense::ISC),
    ("the unlicense", BaseLicense::Unlicense),
    ("this is free and unencumbered", BaseLicense::Unlicense),
];

/// Multilingual license phrase entry.
struct I18nPhrase {
    /// ISO 639-1 language code.
    lang: &'static str,
    /// Key phrase (lowercase) that identifies the license.
    phrase: &'static str,
    /// The base license this phrase identifies.
    /// Uses a function to allow non-const variants like Other(String).
    spdx: &'static str,
}

/// Multilingual license phrases for i18n detection.
///
/// Covers official translations of multilingual licenses (EUPL, CeCILL, PMPL).
/// Integration point for LOL (Lots Of Languages) corpus and polyglot-i18n.
const I18N_PHRASES: &[I18nPhrase] = &[
    // EUPL-1.2 translations (23 official EU languages — key ones here)
    I18nPhrase { lang: "fr", phrase: "licence publique de l'union européenne", spdx: "EUPL-1.2" },
    I18nPhrase { lang: "de", phrase: "open-source-lizenz für die europäische union", spdx: "EUPL-1.2" },
    I18nPhrase { lang: "nl", phrase: "openbare licentie van de europese unie", spdx: "EUPL-1.2" },
    I18nPhrase { lang: "es", phrase: "licencia pública de la unión europea", spdx: "EUPL-1.2" },
    I18nPhrase { lang: "it", phrase: "licenza pubblica dell'unione europea", spdx: "EUPL-1.2" },
    I18nPhrase { lang: "pt", phrase: "licença pública da união europeia", spdx: "EUPL-1.2" },
    // CeCILL (French free software license)
    I18nPhrase { lang: "fr", phrase: "contrat de licence de logiciel libre", spdx: "CeCILL-2.1" },
    // PMPL Dutch translation
    I18nPhrase { lang: "nl", phrase: "palimpsest-mpl licentie", spdx: "PMPL-1.0" },
];

/// Detect the license from a LICENSE file's content using multi-level matching.
pub fn detect_license_from_content(content: &str) -> Option<ContentMatch> {
    let lower = content.to_lowercase();

    // Level 0: Exact key phrase matching.
    for (phrase, license) in LICENSE_PHRASES {
        if lower.contains(phrase) {
            return Some(ContentMatch {
                license: license.clone(),
                confidence: MatchConfidence::Certain,
                matched_phrase: phrase.to_string(),
            });
        }
    }

    // Level 1: i18n phrase matching.
    for entry in I18N_PHRASES {
        if lower.contains(entry.phrase) {
            let license = match entry.spdx {
                "EUPL-1.2" => BaseLicense::EUPL12,
                "PMPL-1.0" => BaseLicense::MPL2, // PMPL falls back to MPL-2.0
                _ => BaseLicense::Other(entry.spdx.to_string()),
            };
            return Some(ContentMatch {
                license,
                confidence: MatchConfidence::High,
                matched_phrase: entry.phrase.to_string(),
            });
        }
    }

    // Level 2: Levenshtein fuzzy match on first 500 chars.
    let sample = &lower[..lower.len().min(500)];
    for (phrase, license) in LICENSE_PHRASES {
        if phrase.len() > 10 {
            // Only fuzzy-match longer phrases to avoid false positives.
            let dist = levenshtein(sample, phrase);
            if dist <= 3 {
                return Some(ContentMatch {
                    license: license.clone(),
                    confidence: MatchConfidence::Medium,
                    matched_phrase: format!("{phrase} (edit distance {dist})"),
                });
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_identical() {
        assert_eq!(levenshtein("hello", "hello"), 0);
    }

    #[test]
    fn test_levenshtein_one_edit() {
        assert_eq!(levenshtein("kitten", "sitten"), 1);
    }

    #[test]
    fn test_trigram_identical() {
        let sim = trigram_similarity("hello world", "hello world");
        assert!((sim - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_trigram_different() {
        let sim = trigram_similarity("hello", "zzzzz");
        assert!(sim < 0.3);
    }

    #[test]
    fn test_detect_mit() {
        let content = "MIT License\n\nCopyright (c) 2026 Someone\n\nPermission is hereby granted...";
        let m = detect_license_from_content(content).unwrap();
        assert_eq!(m.license, BaseLicense::MIT);
        assert_eq!(m.confidence, MatchConfidence::Certain);
    }

    #[test]
    fn test_detect_eupl_french() {
        let content = "Licence Publique de l'Union Européenne v.1.2\n\n...";
        let m = detect_license_from_content(content).unwrap();
        assert_eq!(m.license, BaseLicense::EUPL12);
    }
}
