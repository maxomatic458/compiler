use crate::lexer::utils::all_enum_variants_display;
use lazy_static::lazy_static;
use std::str::FromStr;

use super::{
    error::LexerError,
    position::{Position, Span, Spanned},
    tokens::{Keyword, Literal, Operator, Punctuation, ReassignmentOperator, Token},
};

lazy_static! {
    pub static ref TOKEN_PATTERNS: Vec<(String, Token)> = {
        let mut tokens = vec![];

        tokens.push(Token::Assignment.to_string());
        tokens.extend(all_enum_variants_display::<Punctuation>());
        tokens.extend(all_enum_variants_display::<Operator>());
        tokens.extend(all_enum_variants_display::<Keyword>());
        tokens.extend(all_enum_variants_display::<ReassignmentOperator>());

        tokens.extend(vec![
            Literal::Boolean(true).to_string(),
            Literal::Boolean(false).to_string(),
        ]);

        let mut patterns = vec![];
        for token in tokens {
            patterns.push((token.clone(), Token::from_str(&token).unwrap()));
        }

        patterns.sort_by(|(a, _), (b, _)| a.len().cmp(&b.len()));
        patterns
    };
}

pub fn lex(input: &str) -> Result<Vec<Spanned<Token>>, Spanned<LexerError>> {
    let mut tokens = vec![];
    let mut lexer = Lexer::new(input);

    while let Some(token) = lexer.next_token()? {
        tokens.push(token)
    }

    Ok(tokens)
}

pub fn lex_unspanned(input: &str) -> Result<Vec<Token>, Spanned<LexerError>> {
    let mut tokens = vec![];
    let mut lexer = Lexer::new(input);
    while let Some(token) = lexer.next_token_unspanned()? {
        tokens.push(token)
    }

    Ok(tokens)
}

pub struct Lexer {
    chars: Vec<char>,
    // lines: Vec<Vec<char>>,
    position: Position,
    patterns: Vec<(String, Token)>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            chars: input.chars().collect(),
            // lines: input.lines().map(|x| x.chars().collect()).collect(),
            position: Position::new(0, 0, 0),
            patterns: TOKEN_PATTERNS.to_owned(),
        }
    }

    pub fn next_token(&mut self) -> Result<Option<Spanned<Token>>, Spanned<LexerError>> {
        self.consume_whitespace();

        let start = self.position;

        let mut token: Option<Spanned<Token>> = None;

        if let Some(tok) = self.lex_literal(start)? {
            token = Some(tok)
        } else if let Some(tok) = self.lex_string(start)? {
            token = Some(tok)
        } else if let Some(tok) = self.lex_from_pattern(start)? {
            token = Some(tok)
        } else if let Some(tok) = self.lex_ident(start)? {
            token = Some(tok)
        }

        Ok(token)
    }

    pub fn next_token_unspanned(&mut self) -> Result<Option<Token>, Spanned<LexerError>> {
        if let Some(spanned_token) = self.next_token()? {
            return Ok(Some(spanned_token.value));
        }
        Ok(None)
    }

    fn consume_whitespace(&mut self) {
        while let Some(c) = self.chars.get(self.position.abs) {
            match c {
                ' ' | '\t' | '\r' => {
                    self.advance(1);
                }
                '\n' => {
                    self.advance(1);
                }
                // Kommentare
                '#' => {
                    while let Some(c) = self.chars.get(self.position.abs).cloned() {
                        self.advance(1);
                        if c == '\n' {
                            break;
                        }
                    }
                }
                _ => break,
            };
        }
    }

    fn lex_from_pattern(
        &mut self,
        start: Position,
    ) -> Result<Option<Spanned<Token>>, Spanned<LexerError>> {
        let mut matched_chars = String::new();
        let mut still_matching = self.patterns.clone();
        let mut token_cursor = self.position.abs;
        let mut last_match: Option<(String, Token)> = None; // für den fall "=" und "=="

        while !still_matching.is_empty() {
            if let Some(c) = self.chars.get(token_cursor) {
                matched_chars.push(*c);
                token_cursor += 1;

                still_matching.retain(|(pattern, _)| pattern.starts_with(&matched_chars));

                if let Some((_, token)) = still_matching
                    .iter()
                    .find(|(pattern, _)| *pattern == matched_chars)
                {
                    // let (_, token) = &still_matching[index];
                    last_match = Some((matched_chars.clone(), token.clone()));
                }
                continue;
            }
            break;
        }

        if let Some((pattern, token)) = last_match {
            self.advance(pattern.len());
            return Ok(Some(Spanned {
                value: token,
                span: Span {
                    start,
                    end: self.position,
                },
            }));
        }

        Ok(None)
    }

    fn lex_literal(
        &mut self,
        start: Position,
    ) -> Result<Option<Spanned<Token>>, Spanned<LexerError>> {
        if let Some(float_literal) = self.lex_float(start)? {
            return Ok(Some(float_literal));
        }
        if let Some(int_literal) = self.lex_int(start)? {
            return Ok(Some(int_literal));
        }

        Ok(None)
    }

    fn lex_int(&mut self, start: Position) -> Result<Option<Spanned<Token>>, Spanned<LexerError>> {
        let mut digits = String::new();
        let mut token_cursor = self.position.abs;

        while let Some(c) = self.chars.get(token_cursor) {
            if !(c.is_numeric() || *c == '_') {
                break;
            } // 1_000
            if let Some(next) = self.chars.get(token_cursor + 1) {
                if next.is_alphabetic() {
                    return Ok(None);
                }
            }
            token_cursor += 1;
            digits.push(*c);
        }

        if digits.is_empty() || digits.starts_with('_') {
            return Ok(None);
        }

        if let Ok(token) = Token::from_str(&digits.replace('_', "")) {
            self.advance(digits.len());
            return Ok(Some(Spanned {
                value: token,
                span: Span {
                    start,
                    end: self.position,
                },
            }));
        }

        Ok(None)
        // unreachable!()
    }

    fn lex_float(
        &mut self,
        start: Position,
    ) -> Result<Option<Spanned<Token>>, Spanned<LexerError>> {
        let mut digits = String::new();
        let mut token_cursor = self.position.abs;

        while let Some(c) = self.chars.get(token_cursor) {
            if !(c.is_numeric() || *c == '_' || *c == '.') {
                break;
            }
            if let Some(next) = self.chars.get(token_cursor + 1) {
                if next.is_alphabetic() {
                    return Ok(None);
                }
            }
            token_cursor += 1;
            digits.push(*c);
        }

        if digits.is_empty()
            || !digits.contains('.') // kein float
            || digits.len() == 1 // '.'
            || digits.starts_with('_')
        {
            return Ok(None);
        }

        if digits.starts_with('.') {
            digits = format!("0{digits}"); // .1 => 0.1
        }

        if let Ok(token) = Token::from_str(&digits.replace('_', "")) {
            self.advance(digits.len());
            return Ok(Some(Spanned {
                value: token,
                span: Span {
                    start,
                    end: self.position,
                },
            }));
        }

        // Ok(None);
        Ok(None)
        // unreachable!()
    }

    fn lex_string(
        &mut self,
        start: Position,
    ) -> Result<Option<Spanned<Token>>, Spanned<LexerError>> {
        let mut string = String::new();
        let mut token_cursor = self.position.abs;

        if self.chars.get(token_cursor) != Some(&'"') {
            return Ok(None);
        }

        token_cursor += 1;
        while let Some(c) = self.chars.get(token_cursor) {
            if c == &'"' {
                break;
            }
            token_cursor += 1;
            string.push(*c)
        }

        self.advance(string.len() + 2);

        Ok(Some(Spanned {
            value: Token::String(string),
            span: Span {
                start,
                end: self.position,
            },
        }))
    }

    fn lex_ident(
        &mut self,
        start: Position,
    ) -> Result<Option<Spanned<Token>>, Spanned<LexerError>> {
        let mut ident = String::new();
        let mut token_cursor = self.position.abs;

        while let Some(c) = self.chars.get(token_cursor) {
            if !(c.is_alphanumeric() || *c == '_' || *c == '!') {
                break;
            }
            token_cursor += 1;
            ident.push(*c);
        }

        if ident.is_empty() {
            return Ok(None);
        }

        if let Some(first) = ident.chars().next() {
            self.advance(ident.len());
            if first.is_numeric() {
                return Err(Spanned {
                    value: LexerError::IllegalIdentifier(
                        "identifier must not start with a number".to_string(),
                    ),
                    span: Span {
                        start,
                        end: self.position,
                    },
                });
            }
        }

        if let Some('!') = ident.chars().last() {
            return Ok(Some(Spanned {
                value: Token::MacroKeyword(ident),
                span: Span {
                    start,
                    end: self.position,
                },
            }));
        }

        Ok(Some(Spanned {
            value: Token::Identifier(ident),
            span: Span {
                start,
                end: self.position,
            },
        }))
    }

    fn advance(&mut self, amount: usize) -> bool {
        for _ in 0..amount {
            if let Some(char) = self.chars.get(self.position.abs) {
                self.position.abs += 1;
                if *char == '\n' {
                    self.position.row += 1;
                    self.position.column = 0;
                } else {
                    self.position.column += 1;
                }
            } else {
                return false;
            }
        }
        true
    }
}
