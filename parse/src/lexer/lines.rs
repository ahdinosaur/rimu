// with help from:
// - https://github.com/casey/just/blob/4b5dd245fa040377312eb65c1312a980c0634a91/src/lexer.rs#L11
// - https://github.com/DennisPrediger/SLAC/blob/main/src/scanner.rs

use line_span::{LineSpanIter, LineSpans};
use rimu_meta::{SourceId, Span, Spanned};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum LinesToken<'src> {
    Indent,
    Dedent,
    Line(&'src str),
    EndOfLine,
}

pub(crate) type SpannedLinesToken<'src> = Spanned<LinesToken<'src>>;

#[derive(Debug, thiserror::Error, PartialEq, Eq, PartialOrd, Ord)]
pub enum LinesLexerError {
    #[error("inconsistent indentation")]
    InconsistentLeadingWhitespace {
        span: Span,
        found: usize,
        expected: Vec<usize>,
    },
}

type Result<T> = std::result::Result<T, LinesLexerError>;

struct LinesLexer<'src> {
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

impl<'src> LinesLexer<'src> {
    fn new(code: &'src str, source_id: SourceId) -> Self {
        Self {
            code,
            source_id,
            lines: code.line_spans(),
            indentation: vec![0],
        }
    }

    fn next(&mut self) -> Option<(Spanned<&'src str>, Span)> {
        self.lines.next().map(|line_span| {
            let span = self.span(line_span.start(), line_span.end());
            let line = Spanned::new(line_span.as_str(), span);
            let ending_span = self.span(line_span.end(), line_span.ending());
            (line, ending_span)
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

    fn tokenize(&mut self) -> Result<Vec<SpannedLinesToken<'src>>> {
        let mut tokens = vec![];

        while let Some((line, ending_span)) = self.next() {
            let Some((space, rest)) = self.get_space(line) else {
                continue;
            };
            for dent in self.get_dents(space)? {
                tokens.push(dent);
            }

            for indent in self.get_list_indents(rest.clone())? {
                tokens.push(indent)
            }

            tokens.push(self.line(rest));
            tokens.push(Spanned::new(LinesToken::EndOfLine, ending_span))
        }

        while self.indented() {
            tokens.push(self.dedent(self.code.len()));
        }

        Ok(tokens)
    }

    fn get_space_index(&self, line: &'src str) -> Option<usize> {
        line.char_indices()
            .skip_while(|&(_, c)| c == ' ' || c == '\t')
            .map(|(i, _)| i)
            .next()
    }

    fn get_space(
        &self,
        line: Spanned<&'src str>,
    ) -> Option<(Spanned<&'src str>, Spanned<&'src str>)> {
        let (line, span) = line.take();

        let Some(nonblank_index) = self.get_space_index(line) else {
            return None;
        };

        let space_str = &line[..nonblank_index];
        let space_span = self.span(span.start(), span.start() + nonblank_index);

        let rest_str = &line[nonblank_index..];
        let rest_span = self.span(span.start() + nonblank_index, span.end());

        let space = Spanned::new(space_str, space_span);
        let rest = Spanned::new(rest_str, rest_span);
        Some((space, rest))
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

    fn get_dents(&mut self, space: Spanned<&'src str>) -> Result<Vec<SpannedLinesToken<'src>>> {
        let (space, span) = space.take();
        let next_indent = space.len();
        match self.get_indent_change(next_indent) {
            IndentChange::Continue => Ok(vec![]),
            IndentChange::Decrease => {
                let mut dedents = Vec::new();
                while self.indentation() > next_indent {
                    dedents.push(self.dedent(span.end()));
                }
                Ok(dedents)
            }
            IndentChange::Increase => {
                let prev_indent = self.indentation();
                let indent_diff = next_indent - prev_indent;
                self.indentation.push(next_indent);
                let indent_token = LinesToken::Indent;
                let indent_span = self.span(span.end() - indent_diff, span.end());
                let indent = Spanned::new(indent_token, indent_span);
                Ok(vec![indent])
            }
            IndentChange::Inconsistent => Err(LinesLexerError::InconsistentLeadingWhitespace {
                span,
                found: next_indent,
                expected: self.indentation.clone(),
            }),
        }
    }

    // SPECIAL CASE: the start of a list adds an indent
    fn get_list_indents(
        &mut self,
        rest: Spanned<&'src str>,
    ) -> Result<Vec<SpannedLinesToken<'src>>> {
        let mut indents = Vec::new();
        let (rest, span) = rest.take();
        let mut index = 0;
        while rest[index..].starts_with('-') {
            if let Some(nonblank_index) = self.get_space_index(&rest[1..]) {
                let next_index = 1 + nonblank_index;

                let indent_span =
                    self.span(span.start() + index, span.start() + index + next_index);
                let indent_token = LinesToken::Indent;
                let indent = Spanned::new(indent_token, indent_span);
                indents.push(indent);

                index += next_index;
            } else {
                break;
            }
        }
        Ok(indents)
    }

    fn line(&self, rest: Spanned<&'src str>) -> SpannedLinesToken<'src> {
        let (rest, span) = rest.take();
        Spanned::new(LinesToken::Line(rest), span)
    }

    fn dedent(&mut self, span_start: usize) -> SpannedLinesToken<'src> {
        self.indentation.pop();
        let dedent_span = self.span(span_start, span_start);
        let dedent_token = LinesToken::Dedent;
        Spanned::new(dedent_token, dedent_span)
    }
}

pub(crate) fn tokenize_lines(code: &str, source_id: SourceId) -> Result<Vec<SpannedLinesToken>> {
    LinesLexer::new(code, source_id).tokenize()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rimu_meta::{SourceId, Span};

    use super::{tokenize_lines, LinesLexerError, LinesToken, Spanned, SpannedLinesToken};

    fn span(range: std::ops::Range<usize>) -> Span {
        Span::new(SourceId::empty(), range.start, range.end)
    }

    fn test(code: &str) -> Result<Vec<SpannedLinesToken>, LinesLexerError> {
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

        let expected = Ok(vec![
            Spanned::new(LinesToken::Line("a:"), span(1..3)),
            Spanned::new(LinesToken::EndOfLine, span(3..4)),
            Spanned::new(LinesToken::Indent, span(4..6)),
            Spanned::new(LinesToken::Line("b:"), span(6..8)),
            Spanned::new(LinesToken::EndOfLine, span(8..9)),
            Spanned::new(LinesToken::Indent, span(11..13)),
            Spanned::new(LinesToken::Line("c: d"), span(13..17)),
            Spanned::new(LinesToken::EndOfLine, span(17..18)),
            Spanned::new(LinesToken::Dedent, span(20..20)),
            Spanned::new(LinesToken::Line("e: f"), span(20..24)),
            Spanned::new(LinesToken::EndOfLine, span(24..25)),
            Spanned::new(LinesToken::Dedent, span(25..25)),
            Spanned::new(LinesToken::Line("g: h"), span(25..29)),
            Spanned::new(LinesToken::EndOfLine, span(29..30)),
        ]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn object_hanging_value() {
        //
        //
        let actual = test(
            "
a:
  b:
    c
d: e
        ",
        );

        let expected = Ok(vec![
            Spanned::new(LinesToken::Line("a:"), span(1..3)),
            Spanned::new(LinesToken::EndOfLine, span(3..4)),
            Spanned::new(LinesToken::Indent, span(4..6)),
            Spanned::new(LinesToken::Line("b:"), span(6..8)),
            Spanned::new(LinesToken::EndOfLine, span(8..9)),
            Spanned::new(LinesToken::Indent, span(11..13)),
            Spanned::new(LinesToken::Line("c"), span(13..14)),
            Spanned::new(LinesToken::EndOfLine, span(14..15)),
            Spanned::new(LinesToken::Dedent, span(15..15)),
            Spanned::new(LinesToken::Dedent, span(15..15)),
            Spanned::new(LinesToken::Line("d: e"), span(15..19)),
            Spanned::new(LinesToken::EndOfLine, span(19..20)),
        ]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn list_indents() {
        let actual = test(
            "
a:
  - b: c
    d: e
  - f: g
    h: i
j: k
        ",
        );

        let expected = Ok(vec![
            Spanned::new(LinesToken::Line("a:"), span(1..3)),
            Spanned::new(LinesToken::EndOfLine, span(3..4)),
            Spanned::new(LinesToken::Indent, span(4..6)),
            Spanned::new(LinesToken::Line("b:"), span(6..8)),
            Spanned::new(LinesToken::EndOfLine, span(8..9)),
            Spanned::new(LinesToken::Indent, span(11..13)),
            Spanned::new(LinesToken::Line("c"), span(13..14)),
            Spanned::new(LinesToken::EndOfLine, span(14..15)),
            Spanned::new(LinesToken::Dedent, span(15..15)),
            Spanned::new(LinesToken::Dedent, span(15..15)),
            Spanned::new(LinesToken::Line("d: e"), span(15..19)),
            Spanned::new(LinesToken::EndOfLine, span(19..20)),
        ]);

        assert_eq!(actual, expected);
    }
}
