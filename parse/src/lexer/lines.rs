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
    Dash,
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

            let dents = self.get_dents(space.clone())?;
            tokens.extend(dents);

            let indentation_start = space.inner().len();
            let (rest, list_tokens) = self.get_list_tokens(indentation_start, rest)?;
            tokens.extend(list_tokens);

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

        let nonblank_index = self.get_space_index(line)?;

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

    // NOTE (mw): We have to handle list markers in this lexer.
    // - Each list marker (`-`) starts a new indentation
    fn get_list_tokens(
        &mut self,
        indentation_start: usize,
        rest: Spanned<&'src str>,
    ) -> Result<(Spanned<&'src str>, Vec<SpannedLinesToken<'src>>)> {
        let mut tokens = Vec::new();
        let (rest, span) = rest.take();

        let mut index = 0;

        while rest[index..].starts_with('-') {
            let nonblank_index = self.get_space_index(&rest[index + 1..]);

            // if there's a non-empty character but no empty space before,
            // then is a negate unary operator, not a list marker.
            if nonblank_index.is_some() && nonblank_index.unwrap() == 0 {
                break;
            }

            // otherwise, is a list marker, so add token
            let dash_token = LinesToken::Dash;
            let dash_span = self.span(span.start() + index, span.start() + index + 1);
            let dash = Spanned::new(dash_token, dash_span);
            tokens.push(dash);

            // if empty space before next non-empty character
            if let Some(nonblank_index) = nonblank_index {
                // get length of space before next non-empty charater
                let next_index = 1 + nonblank_index;

                // add indent token
                let indent_span =
                    self.span(span.start() + index, span.start() + index + next_index);
                let indent_token = LinesToken::Indent;
                let indent = Spanned::new(indent_token, indent_span);
                tokens.push(indent);

                // increment our remaining string index
                index += next_index;

                // add indent marker
                let indentation = indentation_start + index;
                self.indentation.push(indentation);
            }
            // otherwise if no non-empty character
            else {
                // rest starts at the end
                index = rest.len() - 1;
                break;
            }
        }

        let remainder_str = &rest[index..];
        let remainder_span = self.span(span.start() + index, span.end());
        let remainder = Spanned::new(remainder_str, remainder_span);

        Ok((remainder, tokens))
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

pub(crate) fn tokenize_lines<'src>(
    code: &'src str,
    source_id: SourceId,
) -> Result<Vec<SpannedLinesToken<'src>>> {
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

    fn test<'src>(code: &'src str) -> Result<Vec<SpannedLinesToken<'src>>, LinesLexerError> {
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
            Spanned::new(LinesToken::Dash, span(6..7)),
            Spanned::new(LinesToken::Indent, span(6..8)),
            Spanned::new(LinesToken::Line("b: c"), span(8..12)),
            Spanned::new(LinesToken::EndOfLine, span(12..13)),
            Spanned::new(LinesToken::Line("d: e"), span(17..21)),
            Spanned::new(LinesToken::EndOfLine, span(21..22)),
            Spanned::new(LinesToken::Dedent, span(24..24)),
            Spanned::new(LinesToken::Dash, span(24..25)),
            Spanned::new(LinesToken::Indent, span(24..26)),
            Spanned::new(LinesToken::Line("f: g"), span(26..30)),
            Spanned::new(LinesToken::EndOfLine, span(30..31)),
            Spanned::new(LinesToken::Line("h: i"), span(35..39)),
            Spanned::new(LinesToken::EndOfLine, span(39..40)),
            Spanned::new(LinesToken::Dedent, span(40..40)),
            Spanned::new(LinesToken::Dedent, span(40..40)),
            Spanned::new(LinesToken::Line("j: k"), span(40..44)),
            Spanned::new(LinesToken::EndOfLine, span(44..45)),
        ]);

        assert_eq!(actual, expected);
    }

    // TODO tests
    // - list mania: lists within lists within lists
    // - list marker vs negate unary
}
