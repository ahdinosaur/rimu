use chumsky::prelude::*;
use rust_decimal::Decimal;
use std::{ops::Range, str::FromStr};

use crate::Token;

type Span = Range<usize>;
type LexerError = Simple<char, Span>;

pub trait Lexer<T>: Parser<char, T, Error = LexerError> + Sized + Clone {}
impl<P, T> Lexer<T> for P where P: Parser<char, T, Error = LexerError> + Clone {}

pub fn tokenize(source: &str) -> Result<Vec<(Token, Span)>, Vec<LexerError>> {
    lexer().parse(source)
}

pub fn lexer() -> impl Lexer<Vec<(Token, Span)>> {
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

    let control = choice((
        just(',').to(Token::Comma),
        just(':').to(Token::Colon),
        just('.').to(Token::Dot),
    ))
    .labelled("control");

    let operator = choice((
        just('+').to(Token::Plus),
        just('-').to(Token::Minus),
        just('*').to(Token::Star),
        just('/').to(Token::Slash),
        just('>').to(Token::Greater),
        just(">=").to(Token::GreaterEqual),
        just('<').to(Token::Less),
        just("<=").to(Token::LessEqual),
        just("and").to(Token::And),
        just("or").to(Token::Or),
        just("xor").to(Token::Xor),
        just("not").to(Token::Not),
        just("div").to(Token::Div),
        just("mod").to(Token::Mod),
    ))
    .labelled("operator");

    let identifier = text::ident().map(Token::Identifier).labelled("identifier");

    let token = choice((
        null, boolean, number, string, delimiter, control, operator, identifier,
    ))
    .recover_with(skip_then_retry_until([]));

    spanned(token).padded().repeated()
}

fn spanned<P, T>(parser: P) -> impl Lexer<(T, Span)>
where
    P: Lexer<T>,
{
    parser.map_with_span(|value, span| (value, span))
}

#[cfg(test)]
mod tests {
    use std::{f64::consts::PI, ops::Range};

    use chumsky::prelude::Simple;
    use pretty_assertions::assert_eq;
    use rust_decimal::{prelude::FromPrimitive, Decimal};

    use super::{tokenize, LexerError, Span, Token};

    #[test]
    fn empty_input() {
        let actual = tokenize("");

        let expected = Ok(vec![]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_null() {
        let actual = tokenize("null");

        let expected = Ok(vec![(Token::Null, (0..4))]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_bool() {
        let actual = tokenize("true");

        let expected = Ok(vec![(Token::Boolean(true), (0..4))]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_integer() {
        let actual = tokenize("9001");

        let expected = Ok(vec![(
            Token::Number(Decimal::from_u64(9001).unwrap()),
            (0..4),
        )]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_float() {
        let actual = tokenize("3.141592653589793");

        let expected = Ok(vec![(
            Token::Number(Decimal::from_f64(PI).unwrap()),
            (0..17),
        )]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_string() {
        let actual = tokenize("\"Hello World\"");

        let expected = Ok(vec![(Token::String(String::from("Hello World")), (0..13))]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn multiple_tokens() {
        let actual = tokenize("1 + 1");

        let expected = Ok(vec![
            (Token::Number(Decimal::from_u8(1).unwrap()), (0..1)),
            (Token::Plus, (2..3)),
            (Token::Number(Decimal::from_u8(1).unwrap()), (4..5)),
        ]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn var_name_underscore() {
        let actual = tokenize("(_SOME_VAR1 * ANOTHER_ONE)");

        let expected = Ok(vec![
            (Token::LeftParen, (0..1)),
            (Token::Identifier(String::from("_SOME_VAR1")), (1..11)),
            (Token::Star, (12..13)),
            (Token::Identifier(String::from("ANOTHER_ONE")), (14..25)),
            (Token::RightParen, (25..26)),
        ]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn unterminated_less() {
        let actual = tokenize("<");

        let expected = Ok(vec![(Token::Less, (0..1))]);

        assert_eq!(actual, expected);
    }

    fn test_number(input: &str, expected: f64) {
        let actual = tokenize(input);

        let expected = Ok(vec![(
            Token::Number(Decimal::from_f64(expected).unwrap()),
            0..input.len(),
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

    fn assert_errors(actual: Result<Vec<(Token, Span)>, Vec<LexerError>>, expected: Vec<String>) {
        assert!(actual.is_err());
        let errors = actual.unwrap_err();
        for index in 0..expected.len() {
            let actual_msg = &errors[index].to_string();
            let expected_msg = &expected[index];
            assert_eq!(actual_msg, expected_msg);
        }
    }

    #[test]
    fn err_unknown_token_1() {
        let actual = tokenize("$");

        let expected = Ok(vec![]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn err_unknown_token_2() {
        let actual = tokenize("$hello");

        // let expected_errs = vec!["".into()];

        assert!(actual.is_err());
        // assert_errors(actual, expected_errs);
    }

    #[test]
    fn err_unterminated_string() {
        let actual = tokenize("\"hello\" + \"world");

        // let expected_errs = vec!["found end of input but expected one of \"\\\\\", \"\\\"\"".into()];

        assert!(actual.is_err());
        // assert_errors(actual, expected_errs);
    }
}
