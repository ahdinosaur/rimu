use rimu_report::{SourceId, Spanned};

use self::{
    line::{tokenize_line, LineLexerError, LineToken},
    lines::{tokenize_lines, LinesLexerError, LinesToken},
};

mod line;
mod lines;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Token {
    Indent,
    Dedent,
    Key(String),
    ListItem,
    Value(String),
    EndOfLine,
}

pub(crate) type SpannedToken = Spanned<Token>;

#[derive(Debug, PartialEq, Eq)]
pub enum LexerError {
    Lines(LinesLexerError),
    Line(LineLexerError),
}

pub(crate) fn tokenize(
    code: &str,
    source: SourceId,
) -> (Option<Vec<SpannedToken>>, Vec<LexerError>) {
    let lines_tokens = match tokenize_lines(code, source.clone()) {
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
                let (line_tokens, line_lexer_errors) = tokenize_line(spanned_line, source.clone());
                if let Some(line_tokens) = line_tokens {
                    for line_token in line_tokens {
                        let (line_token, span) = line_token.take();
                        let token = match line_token {
                            LineToken::Key(key) => Spanned::new(Token::Key(key), span),
                            LineToken::ListItem => Spanned::new(Token::ListItem, span),
                            LineToken::Value(value) => Spanned::new(Token::Value(value), span),
                        };
                        tokens.push(token);
                    }
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
    use rimu_report::{SourceId, Span, Spanned};

    use super::{tokenize, LexerError, SpannedToken, Token};

    fn span(range: Range<usize>) -> Span {
        Span::new(SourceId::empty(), range.start, range.end)
    }

    fn test(code: &str) -> Result<Vec<SpannedToken>, Vec<LexerError>> {
        let (tokens, errors) = tokenize(code, SourceId::empty());
        if let Some(tokens) = tokens {
            Ok(tokens)
        } else {
            Err(errors)
        }
    }

    #[test]
    fn something() {
        let actual = test(
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
            Spanned::new(Token::Key("a".into()), span(1..2)),
            Spanned::new(Token::EndOfLine, span(3..4)),
            Spanned::new(Token::Indent, span(4..6)),
            Spanned::new(Token::Key("b".into()), span(6..7)),
            Spanned::new(Token::EndOfLine, span(8..9)),
            Spanned::new(Token::Indent, span(11..13)),
            Spanned::new(Token::ListItem, span(13..14)),
            Spanned::new(Token::Value("c".into()), span(15..16)),
            Spanned::new(Token::EndOfLine, span(16..17)),
            Spanned::new(Token::ListItem, span(21..22)),
            Spanned::new(Token::Value("d".into()), span(23..24)),
            Spanned::new(Token::EndOfLine, span(24..25)),
            Spanned::new(Token::ListItem, span(29..30)),
            Spanned::new(Token::Key("e".into()), span(31..32)),
            Spanned::new(Token::Value("f".into()), span(34..35)),
            Spanned::new(Token::EndOfLine, span(35..36)),
            Spanned::new(Token::Dedent, span(38..38)),
            Spanned::new(Token::Key("g".into()), span(38..39)),
            Spanned::new(Token::Value("h".into()), span(41..42)),
            Spanned::new(Token::EndOfLine, span(42..43)),
            Spanned::new(Token::Dedent, span(43..43)),
        ]);

        assert_eq!(actual, expected);
    }
}
