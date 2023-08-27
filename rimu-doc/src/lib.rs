// with help from:
// - https://github.com/casey/just/blob/4b5dd245fa040377312eb65c1312a980c0634a91/src/lexer.rs#L11
// - https://github.com/DennisPrediger/SLAC/blob/main/src/scanner.rs

// maybe this shouldn't use chumsky.
//
// state
// - current indentation
//
// parse each line
//   - get indentation (relative to current)
//   - get type
//     - just use regexes
//     - if has ":" (not inside string), then object entry
//     - if starts with "-", then list item
//
// tokens:
// - indent
// - dedent
// - key
// - value
// - list item
//
// doc:
// - object
// - list
// - expression: string

use std::{collections::BTreeMap, str::Chars};

use lazy_static::lazy_static;
use line_span::LineSpanExt;
use regex::Regex;
use rimu_report::{SourceId, Span, Spanned};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LineToken {
    Indent,
    Dedent,
    Whitespace,
    Line(String),
}

pub type SpannedLineToken = Spanned<LineToken>;

#[derive(Debug, thiserror::Error, PartialEq, Eq, PartialOrd, Ord)]
pub enum LineLexerError {
    #[error("unexpected end of file")]
    EndOfFile,
}

type Result<T> = std::result::Result<T, LineLexerError>;

struct LineLexer<'src> {
    code: &'src str,
    source_id: SourceId,
    chars: Chars<'src>,
    tokens: Vec<SpannedLineToken>,
    start: usize,
    current: usize,
    indentation: Vec<&'src str>,
}

enum Indentation<'src> {
    // Line only contains whitespace
    Blank,
    // Indentation continues
    Continue,
    // Indentation decreases
    Decrease,
    // Indentation isn't consistent
    Inconsistent,
    // Indentation increases
    Increase,
    // Indentation mixes spaces and tabs
    Mixed { whitespace: &'src str },
}

impl<'src> LineLexer<'src> {
    fn new(code: &'src str, source_id: SourceId) -> Self {
        Self {
            code,
            source_id,
            chars: code.chars(),
            tokens: vec![],
            start: 0,
            current: 0,
            indentation: vec![],
        }
    }
    fn advance(&mut self) {
        self.current += 1;
    }

    fn next(&mut self) -> Option<char> {
        self.chars.nth(self.current - 1)
    }

    fn next_is(&self, c: char) -> bool {
        self.next() == Some(c)
    }

    fn next_is_whitespace(&self) -> bool {
        self.next_is(' ') || self.next_is('\t')
    }

    fn rest(&self) -> &'src str {
        &self.code[self.start..]
    }

    fn rest_starts_with(&self, prefix: &str) -> bool {
        self.rest().starts_with(prefix)
    }

    fn lexeme(&self) -> &'src str {
        &self.code[self.start..self.current]
    }

    fn token(&mut self, token: LineToken) {
        let span = Span::new(self.source_id, self.start, self.current);
        self.tokens.push(Spanned::new(token, span));

        self.start = self.current;
    }

    fn at_eol(&self) -> bool {
        self.next_is('\n') || self.rest_starts_with("\r\n")
    }

    fn at_eof(&self) -> bool {
        self.rest().is_empty()
    }

    fn at_eol_or_eof(&self) -> bool {
        self.at_eol() || self.at_eof()
    }

    fn is_identifier_start(c: char) -> bool {
        matches!(c, 'a'..='z' | 'A'..='Z' | '_')
    }

    fn is_identifier_continue(c: char) -> bool {
        if Self::is_identifier_start(c) {
            return true;
        }

        matches!(c, '0'..='9' | '-')
    }

    fn indentation(&self) -> &'src str {
        self.indentation.last().unwrap()
    }

    fn indented(&self) -> bool {
        !self.indentation().is_empty()
    }

    fn tokenize(&mut self) -> Result<Vec<SpannedLineToken>> {
        while !self.at_eof() {
            self.lex_line_start()?;
        }

        while self.indented() {
            self.lex_dedent();
        }

        if self.tokens.is_empty() {
            Err(LineLexerError::EndOfFile)
        } else {
            Ok(self.tokens)
        }
    }

    fn lex_line_start(&mut self) -> Result<Vec<SpannedLineToken>> {
        use Indentation::*;

        let nonblank_index = self
            .rest()
            .char_indices()
            .skip_while(|&(_, c)| c == ' ' || c == '\t')
            .map(|(i, _)| i)
            .next()
            .unwrap_or_else(|| self.rest().len());

        let rest = &self.rest()[nonblank_index..];

        let whitespace = &self.rest()[..nonblank_index];

        let body_whitespace = &whitespace[..whitespace
            .char_indices()
            .take(self.indentation().chars().count())
            .map(|(i, _c)| i)
            .next()
            .unwrap_or(0)];

        let spaces = whitespace.chars().any(|c| c == ' ');
        let tabs = whitespace.chars().any(|c| c == '\t');

        let body_spaces = body_whitespace.chars().any(|c| c == ' ');
        let body_tabs = body_whitespace.chars().any(|c| c == '\t');

        #[allow(clippy::if_same_then_else)]
        let indentation = if rest.starts_with('\n') || rest.starts_with("\r\n") || rest.is_empty() {
            Blank
        } else if whitespace == self.indentation() {
            Continue
        } else if self.indentation.contains(&whitespace) {
            Decrease
        } else if body_spaces && body_tabs {
            Mixed {
                whitespace: body_whitespace,
            }
        } else if spaces && tabs {
            Mixed { whitespace }
        } else if whitespace.len() < self.indentation().len() {
            Inconsistent
        } else if body_whitespace.len() >= self.indentation().len()
            && !body_whitespace.starts_with(self.indentation())
        {
            Inconsistent
        } else if whitespace.len() >= self.indentation().len()
            && !whitespace.starts_with(self.indentation())
        {
            Inconsistent
        } else {
            Increase
        };

        match indentation {
            Blank => {
                if !whitespace.is_empty() {
                    while self.next_is_whitespace() {
                        self.advance()?;
                    }

                    self.token(Whitespace);
                };

                Ok(())
            }
            Continue => {
                if !self.indentation().is_empty() {
                    for _ in self.indentation().chars() {
                        self.advance()?;
                    }

                    self.token(Whitespace);
                }

                Ok(())
            }
            Decrease => {
                while self.indentation() != whitespace {
                    self.lex_dedent();
                }

                if !whitespace.is_empty() {
                    while self.next_is_whitespace() {
                        self.advance()?;
                    }

                    self.token(Whitespace);
                }

                Ok(())
            }
            Mixed { whitespace } => {
                for _ in whitespace.chars() {
                    self.advance()?;
                }

                Err(self.error(MixedLeadingWhitespace { whitespace }))
            }
            Inconsistent => {
                for _ in whitespace.chars() {
                    self.advance()?;
                }

                Err(self.error(InconsistentLeadingWhitespace {
                    expected: self.indentation(),
                    found: whitespace,
                }))
            }
            Increase => {
                while self.next_is_whitespace() {
                    self.advance()?;
                }

                if self.open_delimiters() {
                    self.token(Whitespace);
                } else {
                    let indentation = self.lexeme();
                    self.indentation.push(indentation);
                    self.token(Indent);
                    if self.recipe_body_pending {
                        self.recipe_body = true;
                    }
                }

                Ok(())
            }
        }
    }

    fn lex_dedent(&self) {
        fn lex_dedent(&mut self) {
            assert_eq!(self.current_token_length(), 0);
            self.token(Dedent);
            self.indentation.pop();
            self.recipe_body_pending = false;
            self.recipe_body = false;
        }
    }
}

pub fn tokenize_lines(code: &str, source_id: SourceId) -> Result<Vec<SpannedLineToken>> {
    LineLexer::new(code, source_id).tokenize()
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Token {
    Indent,
    Dedent,
    Key(String),
    Value(String),
    ListItem(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Doc {
    Object(BTreeMap<String, Box<SpannedDoc>>),
    List(Box<SpannedDoc>),
    Expression(String),
}

pub type SpannedDoc = Spanned<Doc>;

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rimu_report::{SourceId, Span, Spanned};

    use super::{tokenize, LexerError};
    use crate::{SpannedToken, Token};

    fn span(range: std::ops::Range<usize>) -> Span {
        Span::new(SourceId::empty(), range.start, range.end)
    }

    fn test(code: &str) -> Result<Vec<SpannedToken>, LexerError> {
        tokenize(code, SourceId::empty())
    }

    #[test]
    fn empty_input() {
        let actual = test("");

        let expected = Ok(vec![]);

        assert_eq!(actual, expected);
    }
}
