use rimu_meta::{SourceId, Spanned};

use self::{
    line::{tokenize_line, tokenize_spanned_line, LineLexerError},
    lines::{tokenize_lines, LinesLexerError, LinesToken},
};
use crate::{SpannedToken, Token};

pub(crate) mod line;
pub(crate) mod lines;

#[derive(Debug, PartialEq, Eq)]
pub enum LexerError {
    Lines(LinesLexerError),
    Line(LineLexerError),
}

pub(crate) fn tokenize_expression(
    code: &str,
    source_id: SourceId,
) -> (Option<Vec<SpannedToken>>, Vec<LexerError>) {
    let (tokens, errors) = tokenize_line(code, source_id);
    let errors = errors.into_iter().map(LexerError::Line).collect();
    (tokens, errors)
}

pub(crate) fn tokenize_block(
    code: &str,
    source_id: SourceId,
) -> (Option<Vec<SpannedToken>>, Vec<LexerError>) {
    let lines_tokens = match tokenize_lines(code, source_id.clone()) {
        Ok(lines_tokens) => lines_tokens,
        Err(lines_lexer_error) => return (None, vec![LexerError::Lines(lines_lexer_error)]),
    };

    let mut tokens = vec![];
    let mut errors = vec![];

    for lines_token in lines_tokens {
        let (lines_token, span) = lines_token.take();

        match lines_token {
            LinesToken::Indent => tokens.push(Spanned::new(Token::Indent, span)),
            LinesToken::Dedent => tokens.push(Spanned::new(Token::Dedent, span)),
            LinesToken::EndOfLine => tokens.push(Spanned::new(Token::EndOfLine, span)),
            LinesToken::Line(line) => {
                let spanned_line = Spanned::new(line, span);
                let (line_tokens, line_lexer_errors) =
                    tokenize_spanned_line(spanned_line, source_id.clone());
                if let Some(line_tokens) = line_tokens {
                    tokens.extend(line_tokens.into_iter());
                }
                for line_lexer_error in line_lexer_errors {
                    errors.push(LexerError::Line(line_lexer_error));
                }
            }
        };
    }

    (Some(tokens), errors)
}

#[cfg(test)]
mod tests {

    use std::ops::Range;

    use pretty_assertions::assert_eq;
    use rimu_meta::{SourceId, Span, Spanned};

    use super::{tokenize_block, LexerError, SpannedToken, Token};

    fn span(range: Range<usize>) -> Span {
        Span::new(SourceId::empty(), range.start, range.end)
    }

    fn test_block(code: &str) -> Result<Vec<SpannedToken>, Vec<LexerError>> {
        let (tokens, errors) = tokenize_block(code, SourceId::empty());
        if let Some(tokens) = tokens {
            Ok(tokens)
        } else {
            Err(errors)
        }
    }

    #[test]
    fn block_misc() {
        let actual = test_block(
            "
a:
  b:
    - c
    - d
    - e: f
  g: h
",
        );

        let expected = Ok(vec![
            Spanned::new(Token::Identifier("a".into()), span(1..2)),
            Spanned::new(Token::Colon, span(2..3)),
            Spanned::new(Token::EndOfLine, span(3..4)),
            Spanned::new(Token::Indent, span(4..6)),
            Spanned::new(Token::Identifier("b".into()), span(6..7)),
            Spanned::new(Token::Colon, span(7..8)),
            Spanned::new(Token::EndOfLine, span(8..9)),
            Spanned::new(Token::Indent, span(11..13)),
            Spanned::new(Token::Minus, span(13..14)),
            Spanned::new(Token::Identifier("c".into()), span(15..16)),
            Spanned::new(Token::EndOfLine, span(16..17)),
            Spanned::new(Token::Minus, span(21..22)),
            Spanned::new(Token::Identifier("d".into()), span(23..24)),
            Spanned::new(Token::EndOfLine, span(24..25)),
            Spanned::new(Token::Minus, span(29..30)),
            Spanned::new(Token::Identifier("e".into()), span(31..32)),
            Spanned::new(Token::Colon, span(32..33)),
            Spanned::new(Token::Identifier("f".into()), span(34..35)),
            Spanned::new(Token::EndOfLine, span(35..36)),
            Spanned::new(Token::Dedent, span(38..38)),
            Spanned::new(Token::Identifier("g".into()), span(38..39)),
            Spanned::new(Token::Colon, span(39..40)),
            Spanned::new(Token::Identifier("h".into()), span(41..42)),
            Spanned::new(Token::EndOfLine, span(42..43)),
            Spanned::new(Token::Dedent, span(43..43)),
        ]);

        assert_eq!(actual, expected);
    }
}
