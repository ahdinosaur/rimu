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
pub use self::token::{SpannedToken, Token};

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
        tokens.into_iter().map(|spanned| spanned.take())
    } else {
        return (None, errors);
    };

    let (output, compile_errors) =
        compiler.parse_recovery(chumsky::Stream::from_iter(eoi.clone(), tokens));
    errors.append(&mut compile_errors.into_iter().map(Error::Compiler).collect());

    (output, errors)
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use pretty_assertions::assert_eq;

    use crate::{
        parse, BinaryOperator, Error, Expression, SourceId, Span, Spanned, SpannedExpression,
    };

    fn span(range: Range<usize>) -> Span {
        Span::new(SourceId::empty(), range.start, range.end)
    }

    fn test(code: &str) -> (Option<SpannedExpression>, Vec<Error>) {
        parse(code, SourceId::empty())
    }

    #[test]
    fn empty_input() {
        let (actual_expr, errors) = test("");

        assert_eq!(actual_expr, None);
        // TODO improve this error
        assert!(errors.len() > 0);
    }

    #[test]
    fn arithmetic() {
        let (actual_expr, errors) = test("x + y * (z / w)");

        let expected_expr = Some(Spanned::new(
            Expression::Binary {
                left: Box::new(Spanned::new(Expression::Identifier("x".into()), span(0..1))),
                right: Box::new(Spanned::new(
                    Expression::Binary {
                        left: Box::new(Spanned::new(
                            Expression::Identifier("y".into()),
                            span(4..5),
                        )),
                        right: Box::new(Spanned::new(
                            Expression::Binary {
                                left: Box::new(Spanned::new(
                                    Expression::Identifier("z".into()),
                                    span(9..10),
                                )),
                                right: Box::new(Spanned::new(
                                    Expression::Identifier("w".into()),
                                    span(13..14),
                                )),
                                operator: BinaryOperator::Divide,
                            },
                            span(8..15),
                        )),
                        operator: BinaryOperator::Multiply,
                    },
                    span(4..15),
                )),
                operator: BinaryOperator::Add,
            },
            span(0..15),
        ));

        assert_eq!(actual_expr, expected_expr);
        assert_eq!(errors.len(), 0);
    }
}
