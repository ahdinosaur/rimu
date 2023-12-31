// with help from:
// - https://github.com/zesterer/chumsky/blob/40fe7d1966f375b3c676d01e04c5dca08f7615ac/examples/nano_rust.rs
// - https://github.com/zesterer/tao/blob/6e7be425ba98cb36582b9c836b3b5b120d13194a/syntax/src/token.rs
// - https://github.com/noir-lang/noir/blob/master/crates/noirc_frontend/src/lexer/lexer.rs
// - https://github.com/DennisPrediger/SLAC/blob/main/src/scanner.rs

use chumsky::prelude::*;
use rimu_meta::{SourceId, Span, Spanned};
use rust_decimal::Decimal;
use std::str::FromStr;

use crate::token::{SpannedToken, Token};

pub type LineLexerError = Simple<char, Span>;

pub trait LineLexer<T>: Parser<char, T, Error = LineLexerError> + Sized + Clone {}
impl<P, T> LineLexer<T> for P where P: Parser<char, T, Error = LineLexerError> + Clone {}

pub(crate) fn tokenize_line(
    code: &str,
    source: SourceId,
) -> (Option<Vec<SpannedToken>>, Vec<LineLexerError>) {
    let len = code.chars().count();
    let eoi = Span::new(source.clone(), len, len);
    line_parser().parse_recovery(chumsky::Stream::from_iter(
        eoi,
        code.chars()
            .enumerate()
            .map(|(i, c)| (c, Span::new(source.clone(), i, i + 1))),
    ))
}

pub(crate) fn tokenize_spanned_line(
    spanned_line: Spanned<&str>,
    source: SourceId,
) -> (Option<Vec<SpannedToken>>, Vec<LineLexerError>) {
    let (line, span) = spanned_line.take();
    let eoi = Span::new(source.clone(), span.end(), span.end());
    line_parser().parse_recovery(chumsky::Stream::from_iter(
        eoi,
        line.chars().enumerate().map(|(i, c)| {
            (
                c,
                Span::new(source.clone(), span.start() + i, span.start() + i + 1),
            )
        }),
    ))
}

fn line_parser() -> impl LineLexer<Vec<SpannedToken>> {
    let null = just("null").to(Token::Null).labelled("null");

    let boolean = choice((
        just("true").to(Token::Boolean(true)),
        just("false").to(Token::Boolean(false)),
    ))
    .labelled("boolean");

    let number = text::int(10)
        .chain::<char, _, _>(just('.').chain(text::digits(10)).or_not().flatten())
        .collect::<String>()
        .try_map(|s, span| {
            Decimal::from_str(&s).map_err(|e| Simple::custom(span, format!("{}", e)))
        })
        .map(Token::Number)
        .labelled("number");

    let escape = just('\\')
        .ignore_then(
            just('\\')
                .or(just('/'))
                .or(just('"'))
                .or(just('b').to('\x08'))
                .or(just('f').to('\x0C'))
                .or(just('n').to('\n'))
                .or(just('r').to('\r'))
                .or(just('t').to('\t')),
        )
        .labelled("escape");

    // TODO parse string interpolations
    let string = just('"')
        .ignore_then(filter(|c| *c != '\\' && *c != '"').or(escape).repeated())
        .then_ignore(just('"'))
        .collect::<String>()
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

    let keyword = choice((
        just("if").to(Token::If),
        just("then").to(Token::Then),
        just("else").to(Token::Else),
        just("let").to(Token::Let),
        just("in").to(Token::In),
    ))
    .labelled("keyword");

    let control = choice((
        just(',').to(Token::Comma),
        just(':').to(Token::Colon),
        just('.').to(Token::Dot),
        just("=>").to(Token::FatArrow),
    ))
    .labelled("control");

    let operator = choice((
        just('+').to(Token::Plus),
        just('-').to(Token::Dash),
        just('*').to(Token::Star),
        just('/').to(Token::Slash),
        just('>').to(Token::Greater),
        just(">=").to(Token::GreaterEqual),
        just('<').to(Token::Less),
        just("<=").to(Token::LessEqual),
        just("==").to(Token::Equal),
        just("!=").to(Token::NotEqual),
        just("&&").to(Token::And),
        just("||").to(Token::Or),
        just("^").to(Token::Xor),
        just("!").to(Token::Not),
        just("%").to(Token::Rem),
    ))
    .labelled("operator");

    let identifier = ident().map(Token::Identifier).labelled("identifier");

    let token = choice((
        null, boolean, number, string, delimiter, keyword, control, operator, identifier,
    ))
    .recover_with(skip_then_retry_until([]));

    token
        .map_with_span(Spanned::new)
        .padded()
        .repeated()
        .then_ignore(end())
}

pub fn ident<C: text::Character, E: chumsky::Error<C>>(
) -> impl Parser<C, C::Collection, Error = E> + Copy + Clone {
    filter(|c: &C| c.to_char().is_ascii_alphabetic() || c.to_char() == '_' || c.to_char() == '$')
        .map(Some)
        .chain::<C, Vec<_>, _>(
            filter(|c: &C| c.to_char().is_ascii_alphanumeric() || c.to_char() == '_').repeated(),
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use chumsky::Parser;
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
        line_parser().parse(chumsky::Stream::from_iter(
            eoi,
            code.chars()
                .enumerate()
                .map(|(i, c)| (c, Span::new(source.clone(), i, i + 1))),
        ))
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

    /*
    fn assert_errors(actual: Result<Vec<(Token, Span)>, Vec<Error>>, expected: Vec<String>) {
        assert!(actual.is_err());
        let errors = actual.unwrap_err();
        for index in 0..expected.len() {
            let actual_msg = &errors[index].to_string();
            let expected_msg = &expected[index];
            assert_eq!(actual_msg, expected_msg);
        }
    }
    */

    #[test]
    fn err_unknown_token_1() {
        let actual = test("^&#");

        assert!(actual.is_err());
    }

    #[test]
    fn err_unterminated_string() {
        let actual = test("\"hello\" + \"world");

        // let expected_errs = vec!["found end of input but expected one of \"\\\\\", \"\\\"\"".into()];

        assert!(actual.is_err());
        // assert_errors(actual, expected_errs);
    }
}
