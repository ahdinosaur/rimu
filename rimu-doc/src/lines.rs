// with help from:
// - https://github.com/casey/just/blob/4b5dd245fa040377312eb65c1312a980c0634a91/src/lexer.rs#L11
// - https://github.com/DennisPrediger/SLAC/blob/main/src/scanner.rs

use std::{str::Chars, thread::current};

use line_span::{LineSpan, LineSpanIter, LineSpans};
use rimu_report::{SourceId, Span, Spanned};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LineToken<'src> {
    Indent,
    Dedent,
    Line(&'src str),
}

pub type SpannedLineToken<'src> = Spanned<LineToken<'src>>;

#[derive(Debug, thiserror::Error, PartialEq, Eq, PartialOrd, Ord)]
pub enum LineLexerError<'src> {
    #[error("mixed leading whitespace")]
    MixedLeadingWhitespace { whitespace: &'src str },

    #[error("inconsistent indentation")]
    InconsistentLeadingWhitespace { found: usize, expected: Vec<usize> },
}

type Result<'src, T> = std::result::Result<T, LineLexerError<'src>>;

struct LineLexer<'src> {
    code: &'src str,
    source_id: SourceId,
    lines: LineSpanIter<'src>,
    indentation: Vec<usize>,
}

enum IndentChange {
    // Indentation continues
    Continue,
    // Indentation decreases
    Decrease,
    // Indentation increases
    Increase,
    // Indentation isn't consistent
    Inconsistent,
}

impl<'src> LineLexer<'src> {
    fn new(code: &'src str, source_id: SourceId) -> Self {
        Self {
            code,
            source_id,
            lines: code.line_spans(),
            indentation: vec![0],
        }
    }

    fn next(&mut self) -> Option<Spanned<&'src str>> {
        self.lines.next().map(|line_span| {
            let span = self.span(line_span.start(), line_span.end());
            Spanned::new(line_span.as_str(), span)
        })
    }

    fn span(&self, start: usize, end: usize) -> Span {
        Span::new(self.source_id.clone(), start, end)
    }

    fn indentation(&self) -> usize {
        *self.indentation.last().unwrap()
    }

    fn indented(&self) -> bool {
        self.indentation() != 0
    }

    fn tokenize(&mut self) -> Result<'src, Vec<SpannedLineToken<'src>>> {
        let mut tokens = vec![];

        while let Some(line) = self.next() {
            let (space, rest) = self.get_space(line);
            if let Some(indent) = self.maybe_indent(space)? {
                tokens.push(indent);
            }
            tokens.push(self.line(rest));
        }

        while self.indented() {
            tokens.push(self.dedent(self.code.len()));
        }

        Ok(tokens)
    }

    fn get_space(&self, line: Spanned<&'src str>) -> (Spanned<&'src str>, Spanned<&'src str>) {
        let (line, span) = line.take();

        let nonblank_index = line
            .char_indices()
            .skip_while(|&(_, c)| c == ' ' || c == '\t')
            .map(|(i, _)| i)
            .next()
            .unwrap_or_else(|| line.len());

        let space_str = &line[..nonblank_index];
        let space_span = self.span(span.start(), span.start() + nonblank_index);
        let space = Spanned::new(space_str, space_span);

        let rest_str = &line[nonblank_index..];
        let rest_span = self.span(span.start() + nonblank_index, span.end());
        let rest = Spanned::new(rest_str, rest_span);

        (space, rest)
    }

    fn get_indent_change(&self, next_indentation: usize) -> IndentChange {
        let current_indentation = self.indentation();
        if next_indentation == current_indentation {
            IndentChange::Continue
        } else if next_indentation > current_indentation {
            IndentChange::Increase
        } else if self.indentation.contains(&next_indentation) {
            IndentChange::Decrease
        } else {
            IndentChange::Inconsistent
        }
    }

    fn maybe_indent(
        &mut self,
        space: Spanned<&'src str>,
    ) -> Result<'src, Option<SpannedLineToken<'src>>> {
        let (space, span) = space.take();
        let next_indent = space.len();
        match self.get_indent_change(next_indent) {
            IndentChange::Continue => Ok(None),
            IndentChange::Decrease => Ok(Some(self.dedent(span.end()))),
            IndentChange::Increase => {
                self.indentation.push(next_indent);
                let indent_token = LineToken::Indent;
                let indent_span = self.span(span.end() - next_indent, span.end());
                let indent = Spanned::new(indent_token, indent_span);
                Ok(Some(indent))
            }
            IndentChange::Inconsistent => Err(LineLexerError::InconsistentLeadingWhitespace {
                found: next_indent,
                expected: self.indentation.clone(),
            }),
        }
    }

    fn line(&self, rest: Spanned<&'src str>) -> SpannedLineToken<'src> {
        let (rest, span) = rest.take();
        Spanned::new(LineToken::Line(rest), span)
    }

    fn dedent(&mut self, span_start: usize) -> SpannedLineToken<'src> {
        self.indentation.pop();
        let dedent_span = self.span(span_start, span_start);
        let dedent_token = LineToken::Dedent;
        Spanned::new(dedent_token, dedent_span)
    }
}

pub fn tokenize_lines(code: &str, source_id: SourceId) -> Result<Vec<SpannedLineToken>> {
    LineLexer::new(code, source_id).tokenize()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rimu_report::{SourceId, Span};

    use super::{tokenize_lines, LineLexerError, SpannedLineToken};

    fn span(range: std::ops::Range<usize>) -> Span {
        Span::new(SourceId::empty(), range.start, range.end)
    }

    fn test(code: &str) -> Result<Vec<SpannedLineToken>, LineLexerError> {
        tokenize_lines(code, SourceId::empty())
    }

    #[test]
    fn empty_input() {
        let actual = test("");

        let expected = Ok(vec![]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn empty_basic_indents() {
        let actual = test(
            "
a:
  b:
    c: d
  e: f
g: h
",
        );

        let expected = Ok(vec![]);

        assert_eq!(actual, expected);
    }
}
