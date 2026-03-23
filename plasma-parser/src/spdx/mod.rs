// SPDX-License-Identifier: PMPL-2.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// SPDX expression parsing module — lexer, parser, and identifier catalog.

pub mod catalog;
pub mod lexer;

use crate::family::License;
use crate::spdx::catalog::resolve_identifier;
use crate::spdx::lexer::{Lexer, Token};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors produced during SPDX expression parsing.
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum SpdxParseError {
    #[error("unexpected token: {0}")]
    UnexpectedToken(String),
    #[error("unexpected end of expression")]
    UnexpectedEnd,
    #[error("unmatched closing parenthesis")]
    UnmatchedParen,
    #[error("empty expression")]
    EmptyExpression,
    #[error("failed to resolve identifier: {0}")]
    ResolutionError(String),
}

/// A parsed SPDX expression, which may be compound.
///
/// The AST supports single identifiers, OR (dual licensing where the user
/// chooses), AND (both apply), WITH (license + exception), and parenthesised
/// sub-expressions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpdxExpr {
    /// Single license identifier.
    Simple(License),
    /// Dual licensing: A OR B (user chooses).
    Or(Box<SpdxExpr>, Box<SpdxExpr>),
    /// Combined: A AND B (both apply).
    And(Box<SpdxExpr>, Box<SpdxExpr>),
    /// License with exception: A WITH exception-id.
    With(Box<SpdxExpr>, String),
}

impl std::fmt::Display for SpdxExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpdxExpr::Simple(license) => write!(f, "{license}"),
            SpdxExpr::Or(left, right) => write!(f, "{left} OR {right}"),
            SpdxExpr::And(left, right) => write!(f, "{left} AND {right}"),
            SpdxExpr::With(expr, exception) => write!(f, "{expr} WITH {exception}"),
        }
    }
}

/// Recursive-descent parser for SPDX expressions.
///
/// Precedence (lowest to highest): OR, AND, WITH.
/// Parentheses override precedence.
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    /// Create a new parser from a token stream.
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Peek at the current token without consuming it.
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    /// Consume the current token and advance.
    fn advance(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let token = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(token)
        } else {
            None
        }
    }

    /// Parse a complete SPDX expression (entry point).
    fn parse_expr(&mut self) -> Result<SpdxExpr, SpdxParseError> {
        self.parse_or()
    }

    /// Parse an OR expression (lowest precedence).
    fn parse_or(&mut self) -> Result<SpdxExpr, SpdxParseError> {
        let mut left = self.parse_and()?;
        while matches!(self.peek(), Some(Token::Or)) {
            self.advance(); // consume OR
            let right = self.parse_and()?;
            left = SpdxExpr::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    /// Parse an AND expression (middle precedence).
    fn parse_and(&mut self) -> Result<SpdxExpr, SpdxParseError> {
        let mut left = self.parse_with()?;
        while matches!(self.peek(), Some(Token::And)) {
            self.advance(); // consume AND
            let right = self.parse_with()?;
            left = SpdxExpr::And(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    /// Parse a WITH expression (highest binary precedence).
    fn parse_with(&mut self) -> Result<SpdxExpr, SpdxParseError> {
        let left = self.parse_primary()?;
        if matches!(self.peek(), Some(Token::With)) {
            self.advance(); // consume WITH
            let exception = self.parse_identifier_string()?;
            Ok(SpdxExpr::With(Box::new(left), exception))
        } else {
            Ok(left)
        }
    }

    /// Parse a primary expression: parenthesised group or license identifier.
    fn parse_primary(&mut self) -> Result<SpdxExpr, SpdxParseError> {
        match self.peek() {
            Some(Token::LParen) => {
                self.advance(); // consume '('
                let expr = self.parse_expr()?;
                match self.advance() {
                    Some(Token::RParen) => Ok(expr),
                    _ => Err(SpdxParseError::UnmatchedParen),
                }
            }
            Some(Token::Ident(_)) => {
                let (id, or_later) = self.parse_license_id()?;
                let license = resolve_identifier(&id, or_later)
                    .map_err(|e| SpdxParseError::ResolutionError(e.to_string()))?;
                Ok(SpdxExpr::Simple(license))
            }
            Some(token) => Err(SpdxParseError::UnexpectedToken(format!("{token:?}"))),
            None => Err(SpdxParseError::UnexpectedEnd),
        }
    }

    /// Parse a license identifier with optional version and or-later suffix.
    ///
    /// Accumulates tokens like: Ident("PAGPL") Dash Number(1) Dot Number(0) Dash OrLater
    /// into the string "PAGPL-1.0" with or_later=true.
    fn parse_license_id(&mut self) -> Result<(String, bool), SpdxParseError> {
        let mut parts = Vec::new();
        let mut or_later = false;

        // First token must be an identifier.
        match self.advance() {
            Some(Token::Ident(name)) => parts.push(name),
            _ => return Err(SpdxParseError::UnexpectedEnd),
        }

        // Consume subsequent dash-separated segments (version numbers, sub-identifiers).
        loop {
            match self.peek() {
                Some(Token::Dash) => {
                    // Check if next-next is OrLater (i.e., "-or-later").
                    if self.pos + 1 < self.tokens.len()
                        && matches!(self.tokens[self.pos + 1], Token::OrLater)
                    {
                        self.advance(); // consume Dash
                        self.advance(); // consume OrLater
                        or_later = true;
                        break;
                    }
                    self.advance(); // consume Dash
                    parts.push("-".to_string());

                    // Next should be Ident, Number, or more compound parts.
                    match self.peek() {
                        Some(Token::Number(_)) => {
                            if let Some(Token::Number(n)) = self.advance() {
                                parts.push(n.to_string());
                                // Check for dot-separated minor version.
                                if matches!(self.peek(), Some(Token::Dot)) {
                                    self.advance(); // consume Dot
                                    parts.push(".".to_string());
                                    if let Some(Token::Number(m)) = self.advance() {
                                        parts.push(m.to_string());
                                    }
                                }
                            }
                        }
                        Some(Token::Ident(_)) => {
                            if let Some(Token::Ident(s)) = self.advance() {
                                parts.push(s);
                            }
                        }
                        _ => break,
                    }
                }
                Some(Token::OrLater) => {
                    self.advance();
                    or_later = true;
                    break;
                }
                _ => break,
            }
        }

        let id = parts.concat();
        Ok((id, or_later))
    }

    /// Parse a compound identifier string (used for WITH exception names).
    ///
    /// Accumulates Ident, Dash, Number, Dot tokens into a single string
    /// until a non-identifier token or end-of-input is reached.
    fn parse_identifier_string(&mut self) -> Result<String, SpdxParseError> {
        let mut parts = Vec::new();
        loop {
            match self.peek() {
                Some(Token::Ident(_)) => {
                    if let Some(Token::Ident(s)) = self.advance() {
                        parts.push(s);
                    }
                }
                Some(Token::Dash) => {
                    self.advance();
                    parts.push("-".to_string());
                }
                Some(Token::Number(_)) => {
                    if let Some(Token::Number(n)) = self.advance() {
                        parts.push(n.to_string());
                    }
                }
                Some(Token::Dot) => {
                    self.advance();
                    parts.push(".".to_string());
                }
                _ => break,
            }
        }
        if parts.is_empty() {
            return Err(SpdxParseError::UnexpectedEnd);
        }
        Ok(parts.concat())
    }
}

/// Parse an SPDX expression string into an [`SpdxExpr`] AST.
///
/// Supports compound expressions with OR, AND, WITH operators and parentheses.
///
/// # Examples
///
/// ```
/// use plasma_parser::spdx::parse_spdx_expr;
///
/// let expr = parse_spdx_expr("MIT OR Apache-2.0").unwrap();
/// let simple = parse_spdx_expr("PMPL-2.0-or-later").unwrap();
/// ```
pub fn parse_spdx_expr(input: &str) -> Result<SpdxExpr, SpdxParseError> {
    let tokens = Lexer::new(input).tokenize();
    if tokens.is_empty() {
        return Err(SpdxParseError::EmptyExpression);
    }
    let mut parser = Parser::new(tokens);
    let expr = parser.parse_expr()?;

    // Verify all tokens were consumed.
    if parser.pos < parser.tokens.len() {
        return Err(SpdxParseError::UnexpectedToken(format!(
            "{:?}",
            parser.tokens[parser.pos]
        )));
    }
    Ok(expr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_mit() {
        let expr = parse_spdx_expr("MIT").unwrap();
        assert!(matches!(expr, SpdxExpr::Simple(_)));
    }

    #[test]
    fn test_or_expression() {
        let expr = parse_spdx_expr("MIT OR Apache-2.0").unwrap();
        assert!(matches!(expr, SpdxExpr::Or(_, _)));
    }

    #[test]
    fn test_palimpsest_or_later() {
        let expr = parse_spdx_expr("PMPL-2.0-or-later").unwrap();
        if let SpdxExpr::Simple(License::Palimpsest(p)) = &expr {
            assert!(p.or_later);
        } else {
            panic!("expected palimpsest license");
        }
    }

    #[test]
    fn test_with_expression() {
        let expr = parse_spdx_expr("GPL-3.0 WITH Classpath-exception-2.0").unwrap();
        assert!(matches!(expr, SpdxExpr::With(_, _)));
    }

    #[test]
    fn test_compound_and_or() {
        let expr = parse_spdx_expr("MIT AND BSD-3-Clause OR Apache-2.0").unwrap();
        // OR has lowest precedence, so this is (MIT AND BSD-3-Clause) OR Apache-2.0
        assert!(matches!(expr, SpdxExpr::Or(_, _)));
    }

    #[test]
    fn test_parenthesised() {
        let expr = parse_spdx_expr("MIT OR (GPL-3.0 AND BSD-3-Clause)").unwrap();
        assert!(matches!(expr, SpdxExpr::Or(_, _)));
    }
}
