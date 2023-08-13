use chumsky::prelude::*;

use crate::Token;

type Span = std::ops::Range<usize>;

pub fn lexer() -> impl Parser<char, Vec<(Token, Span)>, Error = Simple<char>> {
    let null = just("null").to(Token::Null);

    let boolean = choice((
        just("true").to(Token::Boolean(true)),
        just("false").to(Token::Boolean(false)),
    ));

    let number = text::int(10)
        .chain::<char, _, _>(just('.').chain(text::digits(10)).or_not().flatten())
        .collect::<String>()
        .map(Token::Number);

    let escape = just('\\').ignore_then(
        just('\\')
            .or(just('/'))
            .or(just('"'))
            .or(just('b').to('\x08'))
            .or(just('f').to('\x0C'))
            .or(just('n').to('\n'))
            .or(just('r').to('\r'))
            .or(just('t').to('\t')),
    );

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
    ));

    let control = choice((
        just(',').to(Token::Comma),
        just(':').to(Token::Colon),
        just('.').to(Token::Dot),
    ));

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
    ));

    let token = choice((null, boolean, number, string, delimiter, control, operator))
        .recover_with(skip_then_retry_until([]));

    token
        .map_with_span(|tok, span| (tok, span))
        .padded()
        .repeated()
}
