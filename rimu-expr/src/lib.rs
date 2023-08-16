// with help from :
// - https://github.com/zesterer/tao/blob/6e7be425ba98cb36582b9c836b3b5b120d13194a/syntax/src/lib.rs

mod compiler;
mod expression;
mod lexer;
mod operator;
mod token;

use std::ops::Range;

use chumsky::Parser;

pub use self::compiler::{compile, compiler_parser, CompilerError};
pub use self::expression::{Expression, SpannedExpression};
pub use self::lexer::{lexer_parser, tokenize, LexerError};
pub use self::operator::{BinaryOperator, UnaryOperator};
pub use self::token::Token;

pub type Span = Range<usize>;

pub fn parse(
    code: &str,
) -> (
    Option<SpannedExpression>,
    Vec<LexerError>,
    Option<Vec<CompilerError>>,
) {
    let lexer = lexer_parser();
    let compiler = compiler_parser();

    let len = code.chars().count();
    let eoi = len..len;

    let (tokens, lex_errors) = lexer.parse_recovery(chumsky::Stream::from_iter(
        eoi.clone(),
        code.chars().enumerate().map(|(i, c)| (c, i..i + 1)),
    ));

    let tokens = if let Some(tokens) = tokens {
        tokens
    } else {
        return (None, lex_errors, None);
    };

    let (output, compile_errors) =
        compiler.parse_recovery(chumsky::Stream::from_iter(eoi.clone(), tokens.into_iter()));

    (output, lex_errors, Some(compile_errors))
}
