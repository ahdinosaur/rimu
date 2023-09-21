use chumsky::prelude::*;

use rimu_ast::{Block, SpannedBlock};
use rimu_meta::{Span, Spanned};

use crate::token::{SpannedToken, Token};

use super::{expression, Compiler, CompilerError};

pub(crate) fn compile_block(
    tokens: Vec<SpannedToken>,
    eoi: Span,
) -> (Option<SpannedBlock>, Vec<CompilerError>) {
    block_parser().parse_recovery(chumsky::Stream::from_iter(
        eoi,
        tokens.into_iter().map(|token| token.take()),
    ))
}

fn block_parser() -> impl Compiler<SpannedBlock> {
    recursive(|block| {
        let expr = expression_parser();
        let nested_block = nested_block_parser(block.clone());
        let object = object_parser(block.clone());
        let list = list_parser(block.clone());
        let function = function_parser(block.clone());
        let call = call_parser(block.clone());
        let if_ = if_parser(block.clone());
        let let_ = let_parser(block.clone());

        object
            .or(list)
            .or(if_)
            .or(let_)
            .or(function)
            .or(call)
            .or(nested_block)
            .or(expr)
            .boxed()
    })
    .then_ignore(end())
}

fn expression_parser<'a>() -> impl Compiler<SpannedBlock> + 'a {
    expression::expression_parser()
        .map(|expr| {
            let (expr, span) = expr.take();
            let block = Block::Expression(expr);
            Spanned::new(block, span)
        })
        .then_ignore(just(Token::EndOfLine))
        .boxed()
}

fn nested_block_parser<'a>(
    block: impl Compiler<SpannedBlock> + 'a,
) -> impl Compiler<SpannedBlock> + 'a {
    just(Token::EndOfLine)
        .ignore_then(just(Token::Indent))
        .ignore_then(block.clone())
        .then_ignore(just(Token::Dedent).to(()).or(end()))
        .boxed()
}

fn entry_parser<'a>(
    block: impl Compiler<SpannedBlock> + 'a,
) -> impl Compiler<(Spanned<String>, SpannedBlock)> + 'a {
    let key = select! {
        Token::String(key) => key,
        Token::Identifier(key) => key
    }
    .map_with_span(Spanned::new)
    .then_ignore(just(Token::Colon));
    let value = block;
    let entry = key.then(value);
    entry.boxed()
}

fn object_parser<'a>(block: impl Compiler<SpannedBlock> + 'a) -> impl Compiler<SpannedBlock> + 'a {
    let entry = entry_parser(block);
    let entries = entry.clone().repeated().at_least(1);
    let object = entries
        .clone()
        .map(Block::Object)
        .map_with_span(Spanned::new);
    object.boxed()
}

fn list_parser<'a>(block: impl Compiler<SpannedBlock> + 'a) -> impl Compiler<SpannedBlock> + 'a {
    let list_item = just(Token::Dash)
        .ignore_then(just(Token::Indent))
        .ignore_then(block)
        .then_ignore(just(Token::Dedent).to(()).or(end()))
        .boxed();
    let list = list_item
        .repeated()
        .at_least(1)
        .map(Block::List)
        .map_with_span(Spanned::new);
    list.boxed()
}

fn function_parser<'a>(
    block: impl Compiler<SpannedBlock> + 'a,
) -> impl Compiler<SpannedBlock> + 'a {
    let arg_name = select! {
        Token::Identifier(arg_name) => arg_name
    }
    .map_with_span(Spanned::new);
    let arg_items = arg_name
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .boxed();
    let args = arg_items.delimited_by(just(Token::LeftParen), just(Token::RightParen));
    let function = args
        .then_ignore(just(Token::FatArrow))
        .then_ignore(just(Token::EndOfLine))
        .then_ignore(just(Token::Indent))
        .then(block)
        .then_ignore(just(Token::Dedent).to(()).or(end()))
        .map(|(args, body)| Block::Function {
            args,
            body: Box::new(body),
        })
        .map_with_span(Spanned::new);

    function.boxed()
}

fn call_parser<'a>(block: impl Compiler<SpannedBlock> + 'a) -> impl Compiler<SpannedBlock> + 'a {
    let function = expression::expression_parser();

    let call = function
        .then_ignore(just(Token::EndOfLine))
        .then_ignore(just(Token::Indent))
        .then(block)
        .then_ignore(just(Token::Dedent).to(()).or(end()))
        .map(|(function, args)| Block::Call {
            function: Box::new(function),
            args: Box::new(args),
        })
        .map_with_span(Spanned::new);

    call.boxed()
}

fn if_parser<'a>(block: impl Compiler<SpannedBlock> + 'a) -> impl Compiler<SpannedBlock> + 'a {
    let value = block;

    let if_then = just(Token::If)
        .ignore_then(value.clone())
        .then_ignore(just(Token::Then))
        .then(value.clone())
        .map(|(condition, consequent)| Block::If {
            condition: Box::new(condition),
            consequent: Some(Box::new(consequent)),
            alternative: None,
        });
    let if_else = just(Token::If)
        .ignore_then(value.clone())
        .then_ignore(just(Token::Else))
        .then(value.clone())
        .map(|(condition, alternative)| Block::If {
            condition: Box::new(condition),
            consequent: None,
            alternative: Some(Box::new(alternative)),
        });
    let if_then_else = just(Token::If)
        .ignore_then(value.clone())
        .then_ignore(just(Token::Then))
        .then(value.clone())
        .then_ignore(just(Token::Else))
        .then(value.clone())
        .map(|((condition, consequent), alternative)| Block::If {
            condition: Box::new(condition),
            consequent: Some(Box::new(consequent)),
            alternative: Some(Box::new(alternative)),
        });

    let if_ = if_then_else
        .or(if_then)
        .or(if_else)
        .map_with_span(Spanned::new);
    if_.boxed()
}

fn let_parser<'a>(block: impl Compiler<SpannedBlock> + 'a) -> impl Compiler<SpannedBlock> + 'a {
    let entry = entry_parser(block.clone());
    let entries = entry.repeated().at_least(1);

    let value = block.clone();

    let let_ = just(Token::Let)
        .ignore_then(just(Token::EndOfLine))
        .ignore_then(just(Token::Indent))
        .ignore_then(entries)
        .then_ignore(just(Token::Dedent))
        .then_ignore(just(Token::In))
        .then(value)
        .map(|(variables, body)| Block::Let {
            variables,
            body: Box::new(body),
        })
        .map_with_span(Spanned::new);

    let_.boxed()
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use chumsky::Parser;
    use pretty_assertions::assert_eq;
    use rimu_ast::{Block, Expression};
    use rimu_meta::{SourceId, Span, Spanned};

    use crate::Token;

    use super::{block_parser, CompilerError, SpannedBlock};

    fn span(range: Range<usize>) -> Span {
        Span::new(SourceId::empty(), range.start, range.end)
    }

    fn test(tokens: Vec<Token>) -> Result<SpannedBlock, Vec<CompilerError>> {
        let source = SourceId::empty();
        let len = tokens.len();
        let eoi = Span::new(source.clone(), len, len);
        block_parser().parse(chumsky::Stream::from_iter(
            eoi,
            tokens
                .into_iter()
                .enumerate()
                .map(|(i, c)| (c, Span::new(source.clone(), i, i + 1))),
        ))
    }

    #[test]
    fn list_simple() {
        //
        // - a
        // - b
        // - c
        //
        let actual = test(vec![
            Token::Dash,
            Token::Indent,
            Token::Identifier("a".into()),
            Token::EndOfLine,
            Token::Dedent,
            Token::Dash,
            Token::Indent,
            Token::Identifier("b".into()),
            Token::EndOfLine,
            Token::Dedent,
            Token::Dash,
            Token::Indent,
            Token::Identifier("c".into()),
            Token::EndOfLine,
            Token::Dedent,
        ]);

        let expected = Ok(Spanned::new(
            Block::List(vec![
                Spanned::new(
                    Block::Expression(Expression::Identifier("a".into())),
                    span(2..3),
                ),
                Spanned::new(
                    Block::Expression(Expression::Identifier("b".into())),
                    span(7..8),
                ),
                Spanned::new(
                    Block::Expression(Expression::Identifier("c".into())),
                    span(12..13),
                ),
            ]),
            span(0..15),
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn object_simple() {
        //
        // a: b
        // c: d
        // e: f
        //
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
            Block::Object(vec![
                (
                    Spanned::new("a".into(), span(0..1)),
                    Spanned::new(
                        Block::Expression(Expression::Identifier("b".into())),
                        span(2..3),
                    ),
                ),
                (
                    Spanned::new("c".into(), span(4..5)),
                    Spanned::new(
                        Block::Expression(Expression::Identifier("d".into())),
                        span(6..7),
                    ),
                ),
                (
                    Spanned::new("e".into(), span(8..9)),
                    Spanned::new(
                        Block::Expression(Expression::Identifier("f".into())),
                        span(10..11),
                    ),
                ),
            ]),
            span(0..12),
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn object_hanging_value() {
        //
        // a:
        //   b:
        //     c
        // d: e
        //
        let actual = test(vec![
            Token::Identifier("a".into()),
            Token::Colon,
            Token::EndOfLine,
            Token::Indent,
            Token::Identifier("b".into()),
            Token::Colon,
            Token::EndOfLine,
            Token::Indent,
            Token::Identifier("c".into()),
            Token::EndOfLine,
            Token::Dedent,
            Token::Dedent,
            Token::Identifier("d".into()),
            Token::Colon,
            Token::Identifier("e".into()),
            Token::EndOfLine,
        ]);

        let expected = Ok(Spanned::new(
            Block::Object(vec![
                (
                    Spanned::new("a".into(), span(0..1)),
                    Spanned::new(
                        Block::Object(vec![(
                            Spanned::new("b".into(), span(4..5)),
                            Spanned::new(
                                Block::Expression(Expression::Identifier("c".into())),
                                span(8..9),
                            ),
                        )]),
                        span(4..11),
                    ),
                ),
                (
                    Spanned::new("d".into(), span(12..13)),
                    Spanned::new(
                        Block::Expression(Expression::Identifier("e".into())),
                        span(14..15),
                    ),
                ),
            ]),
            span(0..16),
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn misc() {
        //
        // a:
        //   b:
        //     - c
        //     - d
        //     - e: f
        //   g: h
        //
        let actual = test(vec![
            Token::Identifier("a".into()),
            Token::Colon,
            Token::EndOfLine,
            Token::Indent,
            Token::Identifier("b".into()),
            Token::Colon,
            Token::EndOfLine,
            Token::Indent,
            Token::Dash,
            Token::Indent,
            Token::Identifier("c".into()),
            Token::EndOfLine,
            Token::Dedent,
            Token::Dash,
            Token::Indent,
            Token::Identifier("d".into()),
            Token::EndOfLine,
            Token::Dedent,
            Token::Dash,
            Token::Indent,
            Token::Identifier("e".into()),
            Token::Colon,
            Token::Identifier("f".into()),
            Token::EndOfLine,
            Token::Dedent,
            Token::Dedent,
            Token::Identifier("g".into()),
            Token::Colon,
            Token::Identifier("h".into()),
            Token::EndOfLine,
            Token::Dedent,
        ]);

        let expected = Ok(Spanned::new(
            Block::Object(vec![(
                Spanned::new("a".into(), span(0..1)),
                Spanned::new(
                    Block::Object(vec![
                        (
                            Spanned::new("b".into(), span(4..5)),
                            Spanned::new(
                                Block::List(vec![
                                    Spanned::new(
                                        Block::Expression(Expression::Identifier("c".into())),
                                        span(10..11),
                                    ),
                                    Spanned::new(
                                        Block::Expression(Expression::Identifier("d".into())),
                                        span(15..16),
                                    ),
                                    Spanned::new(
                                        Block::Object(vec![(
                                            Spanned::new("e".into(), span(20..21)),
                                            Spanned::new(
                                                Block::Expression(Expression::Identifier(
                                                    "f".into(),
                                                )),
                                                span(22..23),
                                            ),
                                        )]),
                                        span(20..24),
                                    ),
                                ]),
                                span(8..25),
                            ),
                        ),
                        (
                            Spanned::new("g".into(), span(26..27)),
                            Spanned::new(
                                Block::Expression(Expression::Identifier("h".into())),
                                span(28..29),
                            ),
                        ),
                    ]),
                    span(4..30),
                ),
            )]),
            span(0..31),
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn multi_entry_object_in_list() {
        //
        // - a: b
        //   c: d
        // - e: f
        //   g: h
        //
        let actual = test(vec![
            Token::Dash,
            Token::Indent,
            Token::Identifier("a".into()),
            Token::Colon,
            Token::Identifier("b".into()),
            Token::EndOfLine,
            Token::Identifier("c".into()),
            Token::Colon,
            Token::Identifier("d".into()),
            Token::EndOfLine,
            Token::Dedent,
            Token::Dash,
            Token::Indent,
            Token::Identifier("e".into()),
            Token::Colon,
            Token::Identifier("f".into()),
            Token::EndOfLine,
            Token::Identifier("g".into()),
            Token::Colon,
            Token::Identifier("h".into()),
            Token::EndOfLine,
            Token::Dedent,
        ]);

        let expected = Ok(Spanned::new(
            Block::List(vec![
                Spanned::new(
                    Block::Object(vec![
                        (
                            Spanned::new("a".into(), span(2..3)),
                            Spanned::new(
                                Block::Expression(Expression::Identifier("b".into())),
                                span(4..5),
                            ),
                        ),
                        (
                            Spanned::new("c".into(), span(6..7)),
                            Spanned::new(
                                Block::Expression(Expression::Identifier("d".into())),
                                span(8..9),
                            ),
                        ),
                    ]),
                    span(2..10),
                ),
                Spanned::new(
                    Block::Object(vec![
                        (
                            Spanned::new("e".into(), span(13..14)),
                            Spanned::new(
                                Block::Expression(Expression::Identifier("f".into())),
                                span(15..16),
                            ),
                        ),
                        (
                            Spanned::new("g".into(), span(17..18)),
                            Spanned::new(
                                Block::Expression(Expression::Identifier("h".into())),
                                span(19..20),
                            ),
                        ),
                    ]),
                    span(13..21),
                ),
            ]),
            span(0..22),
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn multi_entry_object_in_list_in_object() {
        //
        // a:
        //   - b: c
        //     d: e
        //   - f: g
        //     h: i
        // j: k
        //
        let actual = test(vec![
            Token::Identifier("a".into()),
            Token::Colon,
            Token::EndOfLine,
            Token::Indent,
            Token::Dash,
            Token::Indent,
            Token::Identifier("b".into()),
            Token::Colon,
            Token::Identifier("c".into()),
            Token::EndOfLine,
            Token::Identifier("d".into()),
            Token::Colon,
            Token::Identifier("e".into()),
            Token::EndOfLine,
            Token::Dedent,
            Token::Dash,
            Token::Indent,
            Token::Identifier("f".into()),
            Token::Colon,
            Token::Identifier("g".into()),
            Token::EndOfLine,
            Token::Identifier("h".into()),
            Token::Colon,
            Token::Identifier("i".into()),
            Token::EndOfLine,
            Token::Dedent,
            Token::Dedent,
            Token::Identifier("j".into()),
            Token::Colon,
            Token::Identifier("k".into()),
            Token::EndOfLine,
        ]);

        let expected = Ok(Spanned::new(
            Block::Object(vec![
                (
                    Spanned::new("a".into(), span(0..1)),
                    Spanned::new(
                        Block::List(vec![
                            Spanned::new(
                                Block::Object(vec![
                                    (
                                        Spanned::new("b".into(), span(6..7)),
                                        Spanned::new(
                                            Block::Expression(Expression::Identifier("c".into())),
                                            span(8..9),
                                        ),
                                    ),
                                    (
                                        Spanned::new("d".into(), span(10..11)),
                                        Spanned::new(
                                            Block::Expression(Expression::Identifier("e".into())),
                                            span(12..13),
                                        ),
                                    ),
                                ]),
                                span(6..14),
                            ),
                            Spanned::new(
                                Block::Object(vec![
                                    (
                                        Spanned::new("f".into(), span(17..18)),
                                        Spanned::new(
                                            Block::Expression(Expression::Identifier("g".into())),
                                            span(19..20),
                                        ),
                                    ),
                                    (
                                        Spanned::new("h".into(), span(21..22)),
                                        Spanned::new(
                                            Block::Expression(Expression::Identifier("i".into())),
                                            span(23..24),
                                        ),
                                    ),
                                ]),
                                span(17..25),
                            ),
                        ]),
                        span(4..26),
                    ),
                ),
                (
                    Spanned::new("j".into(), span(27..28)),
                    Spanned::new(
                        Block::Expression(Expression::Identifier("k".into())),
                        span(29..30),
                    ),
                ),
            ]),
            span(0..31),
        ));

        assert_eq!(actual, expected);
    }

    // TODO:
    //
    // a:
    //   - b:
    //       d: e
    //   - f:
    //       h: i
    // j: k

    #[test]
    fn operation_if() {
        // if ready
        // then go
        // else stay

        let actual = test(vec![
            Token::If,
            Token::Identifier("ready".into()),
            Token::EndOfLine,
            Token::Then,
            Token::Identifier("go".into()),
            Token::EndOfLine,
            Token::Else,
            Token::Identifier("stay".into()),
            Token::EndOfLine,
        ]);

        let expected = Ok(Spanned::new(
            Block::If {
                condition: Box::new(Spanned::new(
                    Block::Expression(Expression::Identifier("ready".into())),
                    span(1..2),
                )),
                consequent: Some(Box::new(Spanned::new(
                    Block::Expression(Expression::Identifier("go".into())),
                    span(4..5),
                ))),
                alternative: Some(Box::new(Spanned::new(
                    Block::Expression(Expression::Identifier("stay".into())),
                    span(7..8),
                ))),
            },
            span(0..9),
        ));

        assert_eq!(actual, expected);
    }
}
