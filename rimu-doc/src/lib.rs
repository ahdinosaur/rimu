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

use lazy_static::lazy_static;
use line_span::LineSpanExt;
use regex::Regex;
use rimu_report::{SourceId, Span, Spanned};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Token {
    Indent,
    Dedent,
    Key(String),
    Value(String),
    ListItem(String),
}

pub type SpannedToken = Spanned<Token>;

#[derive(Debug, thiserror::Error, PartialEq, Eq, PartialOrd, Ord)]
pub enum LexerError {}

type Result<T> = std::result::Result<T, LexerError>;

struct Lexer<'a> {
    code: &'a str,
    source_id: SourceId,
    indent_level: u64,
}

impl<'a> Lexer<'a> {
    fn new(code: &'a str, source_id: SourceId) -> Self {
        Self {
            code,
            source_id,
            indent_level: 0,
        }
    }

    fn tokenize(&mut self) -> Result<Vec<SpannedToken>> {
        let mut tokens = vec![];
        for line_span in self.code.line_spans() {
            let line = line_span.as_str();
            let span = Span::new(self.source_id.clone(), line_span.start(), line_span.end());
            tokens.extend(self.next_line(line, span)?.into_iter());
        }
        Ok(tokens)
    }

    fn next_line(&mut self, line: &str, span: Span) -> Result<Vec<SpannedToken>> {
        lazy_static! {
            static ref RE: Regex = Regex::new("").unwrap();
        }
        Ok(vec![])
    }
}

pub fn tokenize(code: &str, source_id: SourceId) -> Result<Vec<SpannedToken>> {
    Lexer::new(code, source_id).tokenize()
}

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
