use compiler::compile;
use lexer::tokenize;
use rimu_report::{SourceId, Span};

mod block;
mod compiler;
mod error;
mod lexer;
mod operation;

pub use crate::block::{Block, SpannedBlock};
pub use crate::error::Error;
pub use crate::operation::Operation;

pub fn parse(code: &str, source: SourceId) -> (Option<SpannedBlock>, Vec<Error>) {
    let mut errors = Vec::new();

    let len = code.chars().count();
    let eoi = Span::new(source.clone(), len, len);

    let (tokens, lex_errors) = tokenize(code, source.clone());
    errors.append(&mut lex_errors.into_iter().map(Error::Lexer).collect());

    let Some(tokens) = tokens else {
        return (None, errors);
    };

    let (output, compile_errors) = compile(tokens, eoi);
    errors.append(&mut compile_errors.into_iter().map(Error::Compiler).collect());

    (output, errors)
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use pretty_assertions::assert_eq;
    use rimu_report::{SourceId, Span, Spanned};

    use crate::{
        block::{Block, SpannedBlock},
        parse, Error,
    };
    use rimu_expr::{BinaryOperator, Expression};

    fn span(range: Range<usize>) -> Span {
        Span::new(SourceId::empty(), range.start, range.end)
    }

    fn test(code: &str) -> (Option<SpannedBlock>, Vec<Error>) {
        parse(code, SourceId::empty())
    }

    #[test]
    fn misc() {
        let (actual_expr, errors) = test(
            "
a:
  b:
    - c + d
    - e: f
  g: h
",
        );

        let expected_expr = Some(Spanned::new(
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
                                span(13..32),
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

        assert_eq!(actual_expr, expected_expr);
        assert_eq!(errors.len(), 0);
    }
}
