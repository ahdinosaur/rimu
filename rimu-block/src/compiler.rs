use chumsky::prelude::*;

use std::collections::BTreeMap;

use rimu_expr::{compiler_parser as expr_compiler_parser, CompilerError as ExprCompilerError};
use rimu_report::{Span, Spanned};
use rimu_token::{SpannedToken, Token};

use crate::block::{Block, SpannedBlock};

pub type CompilerError = Simple<Token, Span>;

pub(crate) trait Compiler<T>:
    Parser<Token, T, Error = CompilerError> + Sized + Clone
{
}
impl<P, T> Compiler<T> for P where P: Parser<Token, T, Error = CompilerError> + Clone {}

pub(crate) fn compile(
    tokens: Vec<SpannedToken>,
    eoi: Span,
) -> (Option<SpannedBlock>, Vec<CompilerError>) {
    compiler_parser().parse_recovery(chumsky::Stream::from_iter(
        eoi,
        tokens.into_iter().map(|token| token.take()),
    ))
}

fn compiler_parser() -> impl Compiler<SpannedBlock> {
    recursive(|block| {
        let eol = just(Token::EndOfLine); // .repeated().at_least(1);

        let expr = expr_compiler_parser()
            .map(|expr| {
                let (expr, span) = expr.take();
                let block = Block::Expression(expr);
                Spanned::new(block, span)
            })
            .then_ignore(eol.clone());

        let list_item = just(Token::Minus).ignore_then(block.clone());
        let list = list_item.repeated().at_least(1).map(Block::List).boxed();

        let key = select! {
            Token::String(key) => key,
            Token::Identifier(key) => key
        }
        .map_with_span(Spanned::new)
        .then_ignore(just(Token::Colon));
        let value_simple = expr.clone();
        let value_complex = eol
            .then(just(Token::Indent))
            .then(block.clone())
            .map(|(_, d)| d);
        let value = value_simple.or(value_complex);
        let entry = key.then(value);
        let entries = entry.repeated().at_least(1);
        let object = entries
            .then_ignore(just(Token::Dedent).to(()).or(end()))
            .map(|entries| {
                /*
                let unspanned_keys = vec![];
                let spanned_keys = vec![];
                let obj = Block::Object(BTreeMap::from_iter(entries.into_iter()))
                */
                Block::Object(BTreeMap::from_iter(entries.into_iter()))
            })
            .boxed();

        list.or(object).map_with_span(Spanned::new).or(expr).boxed()
    })
    .then_ignore(end())
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use chumsky::Parser;
    use map_macro::btree_map;
    use pretty_assertions::assert_eq;
    use rimu_expr::Expression;
    use rimu_report::{SourceId, Span, Spanned};
    use rimu_token::Token;

    use crate::compiler::Block;

    use super::{compiler_parser, CompilerError, SpannedBlock};

    fn span(range: Range<usize>) -> Span {
        Span::new(SourceId::empty(), range.start, range.end)
    }

    fn test(tokens: Vec<Token>) -> Result<SpannedBlock, Vec<CompilerError>> {
        let source = SourceId::empty();
        let len = tokens.len();
        let eoi = Span::new(source.clone(), len, len);
        compiler_parser().parse(chumsky::Stream::from_iter(
            eoi,
            tokens
                .into_iter()
                .enumerate()
                .map(|(i, c)| (c, Span::new(source.clone(), i, i + 1))),
        ))
    }

    #[test]
    fn list_simple() {
        let actual = test(vec![
            Token::Minus,
            Token::Identifier("a".into()),
            Token::EndOfLine,
            Token::Minus,
            Token::Identifier("b".into()),
            Token::EndOfLine,
            Token::Minus,
            Token::Identifier("c".into()),
            Token::EndOfLine,
        ]);

        let expected = Ok(Spanned::new(
            Block::List(vec![
                Spanned::new(
                    Block::Expression(Expression::Identifier("a".into())),
                    span(1..2),
                ),
                Spanned::new(
                    Block::Expression(Expression::Identifier("b".into())),
                    span(4..5),
                ),
                Spanned::new(
                    Block::Expression(Expression::Identifier("c".into())),
                    span(7..8),
                ),
            ]),
            span(0..9),
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn object_simple() {
        let actual = test(vec![
            Token::Identifier("a".into()),
            Token::Colon,
            Token::Identifier("b".into()),
            Token::EndOfLine,
            Token::Identifier("c".into()),
            Token::Colon,
            Token::Identifier("d".into()),
            Token::EndOfLine,
            Token::Identifier("e".into()),
            Token::Colon,
            Token::Identifier("f".into()),
            Token::EndOfLine,
        ]);

        let expected = Ok(Spanned::new(
            Block::Object(btree_map! {
                Spanned::new("a".into(), span(0..1)) => {
                    Spanned::new(Block::Expression(Expression::Identifier("b".into())), span(2..3))
                },
                Spanned::new("c".into(), span(4..5)) => {
                    Spanned::new(Block::Expression(Expression::Identifier("d".into())), span(6..7))
                },
                Spanned::new("e".into(), span(8..9)) => {
                    Spanned::new(Block::Expression(Expression::Identifier("f".into())), span(10..11))
                },
            }),
            span(0..12),
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn something() {
        let actual = test(vec![
            Token::Identifier("a".into()),
            Token::Colon,
            Token::EndOfLine,
            Token::Indent,
            Token::Identifier("b".into()),
            Token::Colon,
            Token::EndOfLine,
            Token::Indent,
            Token::Minus,
            Token::Identifier("c".into()),
            Token::EndOfLine,
            Token::Minus,
            Token::Identifier("d".into()),
            Token::EndOfLine,
            Token::Minus,
            Token::Identifier("e".into()),
            Token::Colon,
            Token::Identifier("f".into()),
            Token::EndOfLine,
            Token::Dedent,
            Token::Identifier("g".into()),
            Token::Colon,
            Token::Identifier("h".into()),
            Token::EndOfLine,
            Token::Dedent,
        ]);

        let expected = Ok(Spanned::new(
            Block::Object(btree_map! {
                Spanned::new("a".into(), span(0..1)) => Spanned::new(Block::Object(btree_map! {
                    Spanned::new("b".into(), span(4..5)) => Spanned::new(
                        Block::List(vec![
                            Spanned::new(Block::Expression(Expression::Identifier("c".into())), span(9..10)),
                            Spanned::new(Block::Expression(Expression::Identifier("d".into())), span(12..13)),
                            Spanned::new(
                                Block::Object(btree_map! {
                                    Spanned::new("e".into(), span(15..16)) => {
                                        Spanned::new(Block::Expression(Expression::Identifier("f".into())), span(17..18))
                                    },
                                }),
                                span(15..20)
                            )
                        ]),
                        span(8..20)
                    ),
                    Spanned::new("g".into(), span(20..21)) => {
                        Spanned::new(Block::Expression(Expression::Identifier("h".into())), span(22..23))
                    },
                }), span(4..25)),
            }),
            span(0..25),
        ));

        assert_eq!(actual, expected);
    }
}
