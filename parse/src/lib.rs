use rimu_ast::{SpannedBlock, SpannedExpression};
use rimu_meta::{SourceId, Span};

mod compiler;
mod error;
mod lexer;
mod token;

pub use crate::error::Error;
pub(crate) use compiler::{compile_block, compile_expression};
pub(crate) use lexer::{tokenize_block, tokenize_expression};
pub(crate) use token::{SpannedToken, Token};

pub fn parse_expression(code: &str, source: SourceId) -> (Option<SpannedExpression>, Vec<Error>) {
    let mut errors = Vec::new();

    let len = code.chars().count();
    let eoi = Span::new(source.clone(), len, len);

    let (tokens, lex_errors) = tokenize_expression(code, source.clone());
    errors.append(&mut lex_errors.into_iter().map(Error::Lexer).collect());

    let Some(tokens) = tokens else {
        return (None, errors);
    };

    let (output, compile_errors) = compile_expression(tokens, eoi);
    errors.append(&mut compile_errors.into_iter().map(Error::Compiler).collect());

    (output, errors)
}

pub fn parse_block(code: &str, source: SourceId) -> (Option<SpannedBlock>, Vec<Error>) {
    let mut errors = Vec::new();

    let len = code.chars().count();
    let eoi = Span::new(source.clone(), len, len);

    let (tokens, lex_errors) = tokenize_block(code, source.clone());
    errors.append(&mut lex_errors.into_iter().map(Error::Lexer).collect());

    let Some(tokens) = tokens else {
        return (None, errors);
    };

    let (output, compile_errors) = compile_block(tokens, eoi);
    errors.append(&mut compile_errors.into_iter().map(Error::Compiler).collect());

    (output, errors)
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use pretty_assertions::assert_eq;
    use rimu_ast::{BinaryOperator, Block, Expression, SpannedBlock, SpannedExpression};
    use rimu_meta::{SourceId, Span, Spanned};

    use crate::{parse_block, parse_expression, Error};

    fn span(range: Range<usize>) -> Span {
        Span::new(SourceId::empty(), range.start, range.end)
    }

    fn test_block(code: &str) -> (Option<SpannedBlock>, Vec<Error>) {
        parse_block(code, SourceId::empty())
    }

    fn test_expression(code: &str) -> (Option<SpannedExpression>, Vec<Error>) {
        parse_expression(code, SourceId::empty())
    }

    #[test]
    fn expr_arithmetic() {
        let (actual_expr, errors) = test_expression("x + y * (z / w)");

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

    #[test]
    fn block_misc() {
        let (actual_block, errors) = test_block(
            "
a:
  b:
    - c + d
    - e: f
  g: h
",
        );

        let expected_block = Some(Spanned::new(
            Block::Object(vec![(
                Spanned::new("a".into(), span(1..2)),
                Spanned::new(
                    Block::Object(vec![
                        (
                            Spanned::new("b".into(), span(6..7)),
                            Spanned::new(
                                Block::List(vec![
                                    Spanned::new(
                                        Block::Expression(Expression::Binary {
                                            left: Box::new(Spanned::new(
                                                Expression::Identifier("c".into()),
                                                span(15..16),
                                            )),
                                            right: Box::new(Spanned::new(
                                                Expression::Identifier("d".into()),
                                                span(19..20),
                                            )),
                                            operator: BinaryOperator::Add,
                                        }),
                                        span(15..20),
                                    ),
                                    Spanned::new(
                                        Block::Object(vec![(
                                            Spanned::new("e".into(), span(27..28)),
                                            Spanned::new(
                                                Block::Expression(Expression::Identifier(
                                                    "f".into(),
                                                )),
                                                span(30..31),
                                            ),
                                        )]),
                                        span(27..32),
                                    ),
                                ]),
                                span(13..34),
                            ),
                        ),
                        (
                            Spanned::new("g".into(), span(34..35)),
                            Spanned::new(
                                Block::Expression(Expression::Identifier("h".into())),
                                span(37..38),
                            ),
                        ),
                    ]),
                    span(6..39),
                ),
            )]),
            span(1..39),
        ));

        assert_eq!(actual_block, expected_block);
        assert_eq!(errors.len(), 0);
    }
}
