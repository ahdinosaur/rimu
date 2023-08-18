// with help from :
// - https://github.com/zesterer/tao/blob/6e7be425ba98cb36582b9c836b3b5b120d13194a/syntax/src/lib.rs

mod compiler;
mod error;
mod expression;
mod lexer;
mod operator;
mod token;

use chumsky::Parser;
pub use rimu_report::{SourceId, Span, Spanned};

pub use self::compiler::{compile, compiler_parser, CompilerError};
pub use self::error::Error;
pub use self::expression::{Expression, SpannedExpression};
pub use self::lexer::{lexer_parser, tokenize, LexerError};
pub use self::operator::{BinaryOperator, UnaryOperator};
pub use self::token::Token;

pub fn parse(code: &str, source: SourceId) -> (Option<SpannedExpression>, Vec<Error>) {
    let lexer = lexer_parser();
    let compiler = compiler_parser();

    let mut errors = Vec::new();

    let len = code.chars().count();
    let eoi = Span::new(source.clone(), len, len);

    let (tokens, lex_errors) = lexer.parse_recovery(chumsky::Stream::from_iter(
        eoi.clone(),
        code.chars()
            .enumerate()
            .map(|(i, c)| (c, Span::new(source.clone(), i, i + 1))),
    ));
    errors.append(&mut lex_errors.into_iter().map(Error::Lexer).collect());

    let tokens = if let Some(tokens) = tokens {
        tokens
    } else {
        return (None, errors);
    };

    let (output, compile_errors) =
        compiler.parse_recovery(chumsky::Stream::from_iter(eoi.clone(), tokens.into_iter()));
    errors.append(&mut compile_errors.into_iter().map(Error::Compiler).collect());

    (output, errors)
}
