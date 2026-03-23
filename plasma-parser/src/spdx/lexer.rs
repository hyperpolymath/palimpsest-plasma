// SPDX-License-Identifier: PMPL-2.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// SPDX expression tokeniser — converts raw SPDX expression strings into a
// stream of tokens for the recursive-descent parser.

use serde::{Deserialize, Serialize};

/// Tokens produced by the SPDX expression lexer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Token {
    /// A textual identifier (e.g., "MIT", "PAGPL", "Apache", "Classpath").
    Ident(String),
    /// A numeric literal (e.g., 2, 3, 0).
    Number(u32),
    /// A dash character '-'.
    Dash,
    /// A dot character '.'.
    Dot,
    /// The compound keyword "or-later" (as found in "PMPL-2.0-or-later").
    OrLater,
    /// The keyword "OR" (dual licensing operator).
    Or,
    /// The keyword "AND" (combined licensing operator).
    And,
    /// The keyword "WITH" (license exception operator).
    With,
    /// Left parenthesis '('.
    LParen,
    /// Right parenthesis ')'.
    RParen,
}

/// Lexer for SPDX expression strings.
///
/// Converts input like `"PAGPL-1.0-or-later"` into a token stream:
/// `[Ident("PAGPL"), Dash, Number(1), Dot, Number(0), Dash, OrLater]`.
pub struct Lexer<'a> {
    /// The input string being tokenised.
    input: &'a str,
    /// Remaining characters to process.
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given input string.
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            chars: input.char_indices().peekable(),
        }
    }

    /// Tokenise the entire input, returning a vector of tokens.
    pub fn tokenize(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token() {
            tokens.push(token);
        }
        tokens
    }

    /// Extract the next token from the input, or None if exhausted.
    fn next_token(&mut self) -> Option<Token> {
        // Skip whitespace.
        while let Some(&(_, ch)) = self.chars.peek() {
            if ch.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }

        let &(start, ch) = self.chars.peek()?;

        match ch {
            '(' => {
                self.chars.next();
                Some(Token::LParen)
            }
            ')' => {
                self.chars.next();
                Some(Token::RParen)
            }
            '.' => {
                self.chars.next();
                Some(Token::Dot)
            }
            '-' => {
                self.chars.next();
                // Check for "or-later" keyword: "-or-later" at this position.
                // We need to peek ahead to see if this dash starts "or-later".
                if self.lookahead_or_later(start + 1) {
                    // Consume "or-later" (we already consumed the dash).
                    self.consume_n("or-later".len());
                    Some(Token::OrLater)
                } else {
                    Some(Token::Dash)
                }
            }
            '0'..='9' => {
                let num = self.read_number();
                Some(Token::Number(num))
            }
            '+' => {
                self.chars.next();
                Some(Token::OrLater)
            }
            _ if ch.is_alphanumeric() => {
                let word = self.read_word();
                match word.as_str() {
                    "OR" => Some(Token::Or),
                    "AND" => Some(Token::And),
                    "WITH" => Some(Token::With),
                    _ => Some(Token::Ident(word)),
                }
            }
            _ => {
                // Skip unrecognised characters.
                self.chars.next();
                self.next_token()
            }
        }
    }

    /// Check if the substring starting at `byte_offset` begins with "or-later".
    fn lookahead_or_later(&self, byte_offset: usize) -> bool {
        self.input[byte_offset..].starts_with("or-later")
    }

    /// Consume exactly `n` characters from the input.
    fn consume_n(&mut self, n: usize) {
        for _ in 0..n {
            self.chars.next();
        }
    }

    /// Read a contiguous run of digit characters and return the parsed number.
    fn read_number(&mut self) -> u32 {
        let mut s = String::new();
        while let Some(&(_, ch)) = self.chars.peek() {
            if ch.is_ascii_digit() {
                s.push(ch);
                self.chars.next();
            } else {
                break;
            }
        }
        s.parse().unwrap_or(0)
    }

    /// Read a contiguous word of alphanumeric characters, hyphens within
    /// compound identifiers (but not the kind that separate version numbers),
    /// and plus signs.
    fn read_word(&mut self) -> String {
        let mut s = String::new();
        while let Some(&(_, ch)) = self.chars.peek() {
            if ch.is_alphanumeric() || ch == '_' || ch == '+' {
                s.push(ch);
                self.chars.next();
            } else {
                break;
            }
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_mit() {
        let tokens = Lexer::new("MIT").tokenize();
        assert_eq!(tokens, vec![Token::Ident("MIT".to_string())]);
    }

    #[test]
    fn test_palimpsest_or_later() {
        let tokens = Lexer::new("PAGPL-1.0-or-later").tokenize();
        assert_eq!(
            tokens,
            vec![
                Token::Ident("PAGPL".to_string()),
                Token::Dash,
                Token::Number(1),
                Token::Dot,
                Token::Number(0),
                Token::OrLater,
            ]
        );
    }

    #[test]
    fn test_or_expression() {
        let tokens = Lexer::new("MIT OR Apache-2.0").tokenize();
        assert_eq!(
            tokens,
            vec![
                Token::Ident("MIT".to_string()),
                Token::Or,
                Token::Ident("Apache".to_string()),
                Token::Dash,
                Token::Number(2),
                Token::Dot,
                Token::Number(0),
            ]
        );
    }

    #[test]
    fn test_with_expression() {
        let tokens = Lexer::new("GPL-3.0 WITH Classpath-exception-2.0").tokenize();
        assert_eq!(
            tokens,
            vec![
                Token::Ident("GPL".to_string()),
                Token::Dash,
                Token::Number(3),
                Token::Dot,
                Token::Number(0),
                Token::With,
                Token::Ident("Classpath".to_string()),
                Token::Dash,
                Token::Ident("exception".to_string()),
                Token::Dash,
                Token::Number(2),
                Token::Dot,
                Token::Number(0),
            ]
        );
    }

    #[test]
    fn test_parentheses() {
        let tokens = Lexer::new("(MIT)").tokenize();
        assert_eq!(
            tokens,
            vec![
                Token::LParen,
                Token::Ident("MIT".to_string()),
                Token::RParen,
            ]
        );
    }

    #[test]
    fn test_plus_or_later() {
        let tokens = Lexer::new("GPL-3.0+").tokenize();
        assert_eq!(
            tokens,
            vec![
                Token::Ident("GPL".to_string()),
                Token::Dash,
                Token::Number(3),
                Token::Dot,
                Token::Number(0),
                Token::OrLater,
            ]
        );
    }
}
