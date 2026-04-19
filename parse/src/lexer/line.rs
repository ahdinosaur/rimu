// with help from:
// - https://github.com/zesterer/chumsky/blob/0.12/examples/nano_rust.rs
// - https://github.com/zesterer/tao/blob/6e7be425ba98cb36582b9c836b3b5b120d13194a/syntax/src/token.rs
// - https://github.com/noir-lang/noir/blob/master/crates/noirc_frontend/src/lexer/lexer.rs
// - https://github.com/DennisPrediger/SLAC/blob/main/src/scanner.rs

use chumsky::{
    extra,
    input::{Stream, ValueInput},
    prelude::*,
};
use rimu_meta::{SourceId, Span, Spanned};
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::token::{SpannedToken, Token};

pub type LineLexerError = Rich<'static, char, Span>;

pub(crate) trait LineLexer<'src, I, T>:
    Parser<'src, I, T, extra::Err<Rich<'src, char, Span>>> + Clone
where
    I: ValueInput<'src, Token = char, Span = Span>,
{
}

impl<'src, I, P, T> LineLexer<'src, I, T> for P
where
    I: ValueInput<'src, Token = char, Span = Span>,
    P: Parser<'src, I, T, extra::Err<Rich<'src, char, Span>>> + Clone,
{
}

pub(crate) fn tokenize_line(
    code: &str,
    source: SourceId,
) -> (Option<Vec<SpannedToken>>, Vec<LineLexerError>) {
    let len = code.chars().count();
    let eoi = Span::new(source.clone(), len, len);
    let tokens: Vec<(char, Span)> = code
        .chars()
        .enumerate()
        .map(|(i, c)| (c, Span::new(source.clone(), i, i + 1)))
        .collect();
    let stream = Stream::from_iter(tokens).map(eoi, |(t, s): (char, Span)| (t, s));
    let (output, errors) = line_parser().parse(stream).into_output_errors();
    let errors = errors.into_iter().map(|e| e.into_owned()).collect();
    (output, errors)
}

pub(crate) fn tokenize_spanned_line(
    spanned_line: Spanned<&str>,
    source: SourceId,
) -> (Option<Vec<SpannedToken>>, Vec<LineLexerError>) {
    let (line, span) = spanned_line.take();
    let eoi = Span::new(source.clone(), span.end(), span.end());
    let tokens: Vec<(char, Span)> = line
        .chars()
        .enumerate()
        .map(|(i, c)| {
            (
                c,
                Span::new(source.clone(), span.start() + i, span.start() + i + 1),
            )
        })
        .collect();
    let stream = Stream::from_iter(tokens).map(eoi, |(t, s): (char, Span)| (t, s));
    let (output, errors) = line_parser().parse(stream).into_output_errors();
    let errors = errors.into_iter().map(|e| e.into_owned()).collect();
    (output, errors)
}

fn line_parser<'src, I>() -> impl LineLexer<'src, I, Vec<SpannedToken>>
where
    I: ValueInput<'src, Token = char, Span = Span> + 'src,
{
    let digit = any::<I, _>().filter(|c: &char| c.is_ascii_digit());

    // Integer part: a single `0`, or a non-zero digit followed by any digits.
    // Leading zeros are rejected so `0o..`, `0x..`, etc. remain available as future number prefixes.
    let int = choice((
        just('0').map(|c: char| c.to_string()),
        any::<I, _>()
            .filter(|c: &char| matches!(c, '1'..='9'))
            .then(digit.repeated().collect::<String>())
            .map(|(first, rest): (char, String)| {
                let mut s = String::with_capacity(1 + rest.len());
                s.push(first);
                s.push_str(&rest);
                s
            }),
    ));

    let frac = just('.').ignore_then(digit.repeated().at_least(1).collect::<String>());

    let number = int
        .then(frac.or_not())
        .map(
            |(int_part, frac_part): (String, Option<String>)| match frac_part {
                Some(frac) => format!("{}.{}", int_part, frac),
                None => int_part,
            },
        )
        .try_map(|s, span| Decimal::from_str(&s).map_err(|e| Rich::custom(span, format!("{}", e))))
        .map(Token::Number)
        .labelled("number");

    let escape = just('\\')
        .ignore_then(choice((
            just('\\'),
            just('/'),
            just('"'),
            just('b').to('\x08'),
            just('f').to('\x0C'),
            just('n').to('\n'),
            just('r').to('\r'),
            just('t').to('\t'),
        )))
        .labelled("escape");

    // TODO parse string interpolations
    let string = just('"')
        .ignore_then(
            any::<I, _>()
                .filter(|c: &char| *c != '\\' && *c != '"')
                .or(escape)
                .repeated()
                .collect::<String>(),
        )
        .then_ignore(just('"'))
        .map(Token::String)
        .labelled("string");

    let delimiter = choice((
        just('(').to(Token::LeftParen),
        just(')').to(Token::RightParen),
        just('[').to(Token::LeftBrack),
        just(']').to(Token::RightBrack),
        just('{').to(Token::LeftBrace),
        just('}').to(Token::RightBrace),
    ))
    .labelled("delimiter");

    let control = choice((
        just(',').to(Token::Comma),
        just(':').to(Token::Colon),
        just('.').to(Token::Dot),
        just("=>").to(Token::FatArrow),
    ))
    .labelled("control");

    let operator = choice((
        just(">=").to(Token::GreaterEqual),
        just("<=").to(Token::LessEqual),
        just("==").to(Token::Equal),
        just("!=").to(Token::NotEqual),
        just("&&").to(Token::And),
        just("||").to(Token::Or),
        just('+').to(Token::Plus),
        just('-').to(Token::Dash),
        just('*').to(Token::Star),
        just('/').to(Token::Slash),
        just('>').to(Token::Greater),
        just('<').to(Token::Less),
        just('^').to(Token::Xor),
        just('!').to(Token::Not),
        just('%').to(Token::Rem),
    ))
    .labelled("operator");

    let identifier = ident::<I>()
        .map(|ident: String| match ident.as_str() {
            "null" => Token::Null,
            "true" => Token::Boolean(true),
            "false" => Token::Boolean(false),
            "if" => Token::If,
            "then" => Token::Then,
            "else" => Token::Else,
            "let" => Token::Let,
            "in" => Token::In,
            _ => Token::Identifier(ident),
        })
        .labelled("identifier");

    let token = choice((number, string, delimiter, control, operator, identifier))
        .recover_with(skip_then_retry_until(any().ignored(), end()));

    let padding = || {
        let inline_whitespace = any::<I, _>()
            .filter(|c: &char| c.is_whitespace())
            .ignored();
        let comment = just('#')
            .ignore_then(
                any::<I, _>()
                    .filter(|c: &char| *c != '\n' && *c != '\r')
                    .repeated(),
            )
            .ignored();
        choice((inline_whitespace, comment)).repeated().ignored()
    };

    token
        .map_with(|v, e| Spanned::new(v, e.span()))
        .padded_by(padding())
        .repeated()
        .collect::<Vec<_>>()
        .then_ignore(padding())
        .then_ignore(end())
}

fn ident<'src, I>() -> impl Parser<'src, I, String, extra::Err<Rich<'src, char, Span>>> + Clone
where
    I: ValueInput<'src, Token = char, Span = Span>,
{
    let first = any::<I, _>().filter(|c: &char| c.is_ascii_alphabetic() || *c == '_' || *c == '$');
    let rest = any::<I, _>()
        .filter(|c: &char| c.is_ascii_alphanumeric() || *c == '_')
        .repeated()
        .collect::<String>();
    first.then(rest).map(|(f, r): (char, String)| {
        let mut s = String::with_capacity(1 + r.len());
        s.push(f);
        s.push_str(&r);
        s
    })
}

#[cfg(test)]
mod tests {
    use chumsky::{input::Stream, prelude::*};
    use pretty_assertions::assert_eq;
    use rimu_meta::{SourceId, Span, Spanned};
    use rust_decimal::{prelude::FromPrimitive, Decimal};
    use std::{f64::consts::PI, ops::Range};

    use crate::token::{SpannedToken, Token};

    use super::{line_parser, LineLexerError};

    fn span(range: Range<usize>) -> Span {
        Span::new(SourceId::empty(), range.start, range.end)
    }

    fn test(code: &str) -> Result<Vec<SpannedToken>, Vec<LineLexerError>> {
        let source = SourceId::empty();
        let len = code.chars().count();
        let eoi = Span::new(source.clone(), len, len);
        let tokens: Vec<(char, Span)> = code
            .chars()
            .enumerate()
            .map(|(i, c)| (c, Span::new(source.clone(), i, i + 1)))
            .collect();
        let stream = Stream::from_iter(tokens).map(eoi, |(t, s): (char, Span)| (t, s));
        line_parser()
            .parse(stream)
            .into_result()
            .map_err(|errs| errs.into_iter().map(|e| e.into_owned()).collect())
    }

    #[test]
    fn empty_input() {
        let actual = test("");

        let expected = Ok(vec![]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_null() {
        let actual = test("null");

        let expected = Ok(vec![Spanned::new(Token::Null, span(0..4))]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_bool() {
        let actual = test("true");

        let expected = Ok(vec![Spanned::new(Token::Boolean(true), span(0..4))]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_integer() {
        let actual = test("9001");

        let expected = Ok(vec![Spanned::new(
            Token::Number(Decimal::from_u64(9001).unwrap()),
            span(0..4),
        )]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_float() {
        let actual = test("3.141592653589793");

        let expected = Ok(vec![Spanned::new(
            Token::Number(Decimal::from_f64(PI).unwrap()),
            span(0..17),
        )]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_string() {
        let actual = test("\"Hello World\"");

        let expected = Ok(vec![Spanned::new(
            Token::String(String::from("Hello World")),
            span(0..13),
        )]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn multiple_tokens() {
        let actual = test("1 + 1");

        let expected = Ok(vec![
            Spanned::new(Token::Number(Decimal::from_u8(1).unwrap()), span(0..1)),
            Spanned::new(Token::Plus, span(2..3)),
            Spanned::new(Token::Number(Decimal::from_u8(1).unwrap()), span(4..5)),
        ]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn var_name_underscore() {
        let actual = test("(_SOME_VAR1 * ANOTHER_ONE)");

        let expected = Ok(vec![
            Spanned::new(Token::LeftParen, span(0..1)),
            Spanned::new(Token::Identifier(String::from("_SOME_VAR1")), span(1..11)),
            Spanned::new(Token::Star, span(12..13)),
            Spanned::new(Token::Identifier(String::from("ANOTHER_ONE")), span(14..25)),
            Spanned::new(Token::RightParen, span(25..26)),
        ]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn identifier_not_keyword_prefix() {
        let actual = test("install");

        let expected = Ok(vec![Spanned::new(
            Token::Identifier(String::from("install")),
            span(0..7),
        )]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn keyword_in() {
        let actual = test("in");

        let expected = Ok(vec![Spanned::new(Token::In, span(0..2))]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn unterminated_less() {
        let actual = test("<");

        let expected = Ok(vec![Spanned::new(Token::Less, span(0..1))]);

        assert_eq!(actual, expected);
    }

    fn test_number(input: &str, expected: f64) {
        let actual = test(input);

        let expected = Ok(vec![Spanned::new(
            Token::Number(Decimal::from_f64(expected).unwrap()),
            span(0..input.len()),
        )]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn number_parts() {
        test_number("10", 10.0);
        test_number("10.0", 10.0);
        test_number("20.4", 20.4);
        // test_number("30.", 30.0);
        // test_number(".3", 0.3);
    }

    #[test]
    fn err_unknown_token_1() {
        let actual = test("^&");

        assert!(actual.is_err());
    }

    #[test]
    fn err_unterminated_string() {
        let actual = test("\"hello\" + \"world");

        assert!(actual.is_err());
    }

    #[test]
    fn comment_only() {
        let actual = test("# just a comment");

        let expected = Ok(vec![]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn comment_after_tokens() {
        let actual = test("1 + 1 # add one and one");

        let expected = Ok(vec![
            Spanned::new(Token::Number(Decimal::from_u8(1).unwrap()), span(0..1)),
            Spanned::new(Token::Plus, span(2..3)),
            Spanned::new(Token::Number(Decimal::from_u8(1).unwrap()), span(4..5)),
        ]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn comment_before_tokens() {
        let actual = test("# leading comment");

        let expected = Ok(vec![]);

        assert_eq!(actual, expected);
    }
}
