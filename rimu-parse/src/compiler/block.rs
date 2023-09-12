use indexmap::IndexMap;

use chumsky::prelude::*;

use rimu_ast::{Block, BlockOperation, SpannedBlock};
use rimu_meta::{Span, Spanned};

use crate::token::{SpannedToken, Token};

use super::{expression::expression_parser, Compiler, CompilerError};

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
        let eol = just(Token::EndOfLine);

        let expr = expression_parser()
            .map(|expr| {
                let (expr, span) = expr.take();
                let block = Block::Expression(expr);
                Spanned::new(block, span)
            })
            .then_ignore(eol.clone());

        let key = select! {
            Token::String(key) => key,
            Token::Identifier(key) => key
        }
        .map_with_span(Spanned::new)
        .then_ignore(just(Token::Colon));
        let value_simple = expr.clone();
        let value_complex = eol
            .ignore_then(just(Token::Indent))
            .ignore_then(block.clone())
            .then_ignore(just(Token::Dedent).to(()).or(end()));
        let value = value_simple.or(value_complex);
        let entry = key.then(value);
        let entries = entry.clone().repeated().at_least(1);
        let object = entries
            .clone()
            .try_map(parse_object_entries)
            .map_with_span(Spanned::new)
            .boxed();

        let list_item_object_multi = just(Token::Minus)
            .ignore_then(entry.clone())
            .then_ignore(just(Token::Indent))
            .then(entries.clone())
            .then_ignore(just(Token::Dedent).to(()).or(end()))
            .map(|(entry, mut entries)| {
                let mut ret = Vec::with_capacity(entries.len() + 1);
                ret.push(entry);
                ret.append(&mut entries);
                ret
            })
            .try_map(parse_object_entries)
            .map_with_span(Spanned::new)
            .boxed();
        let list_item_simple = just(Token::Minus).ignore_then(block.clone()).boxed();
        let list_item = list_item_object_multi.or(list_item_simple);
        let list = list_item
            .repeated()
            .at_least(1)
            .map(Block::List)
            .map_with_span(Spanned::new)
            .boxed();

        object.or(list).or(expr).boxed()
    })
    .then_ignore(end())
}

fn parse_object_entries(
    entries: Vec<(Spanned<String>, SpannedBlock)>,
    span: Span,
) -> Result<Block, CompilerError> {
    if let Some(operator) = find_block_operator(&entries) {
        return Ok(Block::Operation(Box::new(parse_block_operation(
            operator, entries, span,
        )?)));
    }

    let mut next_entries = Vec::new();
    for (key, value) in entries.into_iter() {
        let (key, key_span) = key.take();
        let key = unescape_non_block_operation_key(&key).to_owned();
        next_entries.push((Spanned::new(key, key_span), value));
    }
    Ok(Block::Object(next_entries))
}

fn find_block_operator<Value>(entries: &[(Spanned<String>, Value)]) -> Option<String> {
    for (key, _) in entries.iter() {
        let key = key.inner();
        let mut chars = key.chars();
        let is_op = chars.next() == Some('$') && chars.next() != Some('$');
        if is_op {
            return Some(key.clone());
        }
    }
    None
}

fn parse_block_operation(
    operator: String,
    entries: Vec<(Spanned<String>, Spanned<Block>)>,
    span: Span,
) -> Result<BlockOperation, CompilerError> {
    let object = IndexMap::from_iter(
        entries
            .into_iter()
            .map(|(key, value)| (key.into_inner(), value)),
    );
    let operation = match operator.as_str() {
        "$if" => {
            static KEYS: [&str; 3] = ["$if", "then", "else"];
            check_block_operation_keys(span, "$if", &KEYS, &object)?;

            let condition = object.get("$if").unwrap().to_owned();
            let consequent = object.get("then").cloned();
            let alternative = object.get("else").cloned();

            BlockOperation::If {
                condition,
                consequent,
                alternative,
            }
        }
        "$let" => {
            static KEYS: [&str; 2] = ["$let", "in"];
            check_block_operation_keys(span.clone(), "$let", &KEYS, &object)?;

            let variables = object.get("$let").unwrap().to_owned();
            let body = object
                .get("in")
                .ok_or_else(|| CompilerError::custom(span, "$let: missing field \"in\""))?
                .to_owned();
            BlockOperation::Let { variables, body }
        }
        &_ => {
            return Err(CompilerError::custom(
                span,
                format!("Unknown block operator: {}", operator.as_str()),
            ))
        }
    };
    Ok(operation)
}

fn check_block_operation_keys<Value>(
    span: Span,
    op: &str,
    keys: &[&str],
    object: &IndexMap<String, Value>,
) -> Result<(), CompilerError> {
    for key in object.keys() {
        if !keys.contains(&key.as_str()) {
            return Err(CompilerError::custom(
                span,
                format!("{}: unexpected field \"{}\"", op, key),
            ));
        }
    }
    Ok(())
}

fn unescape_non_block_operation_key(key: &str) -> &str {
    if key.starts_with("$$") {
        &key[1..]
    } else {
        key
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use chumsky::Parser;
    use pretty_assertions::assert_eq;
    use rimu_ast::{Block, BlockOperation, Expression};
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
                                        span(9..10),
                                    ),
                                    Spanned::new(
                                        Block::Expression(Expression::Identifier("d".into())),
                                        span(12..13),
                                    ),
                                    Spanned::new(
                                        Block::Object(vec![(
                                            Spanned::new("e".into(), span(15..16)),
                                            Spanned::new(
                                                Block::Expression(Expression::Identifier(
                                                    "f".into(),
                                                )),
                                                span(17..18),
                                            ),
                                        )]),
                                        span(15..19),
                                    ),
                                ]),
                                span(8..19),
                            ),
                        ),
                        (
                            Spanned::new("g".into(), span(20..21)),
                            Spanned::new(
                                Block::Expression(Expression::Identifier("h".into())),
                                span(22..23),
                            ),
                        ),
                    ]),
                    span(4..24),
                ),
            )]),
            span(0..25),
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
            Token::Minus,
            Token::Identifier("a".into()),
            Token::Colon,
            Token::Identifier("b".into()),
            Token::EndOfLine,
            Token::Indent,
            Token::Identifier("c".into()),
            Token::Colon,
            Token::Identifier("d".into()),
            Token::EndOfLine,
            Token::Dedent,
            Token::Minus,
            Token::Identifier("e".into()),
            Token::Colon,
            Token::Identifier("f".into()),
            Token::EndOfLine,
            Token::Indent,
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
                            Spanned::new("a".into(), span(1..2)),
                            Spanned::new(
                                Block::Expression(Expression::Identifier("b".into())),
                                span(3..4),
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
                    span(0..11),
                ),
                Spanned::new(
                    Block::Object(vec![
                        (
                            Spanned::new("e".into(), span(12..13)),
                            Spanned::new(
                                Block::Expression(Expression::Identifier("f".into())),
                                span(14..15),
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
                    span(11..22),
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
            Token::Minus,
            Token::Identifier("b".into()),
            Token::Colon,
            Token::Identifier("c".into()),
            Token::EndOfLine,
            Token::Indent,
            Token::Identifier("d".into()),
            Token::Colon,
            Token::Identifier("e".into()),
            Token::EndOfLine,
            Token::Dedent,
            Token::Minus,
            Token::Identifier("f".into()),
            Token::Colon,
            Token::Identifier("g".into()),
            Token::EndOfLine,
            Token::Indent,
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
                                        Spanned::new("b".into(), span(5..6)),
                                        Spanned::new(
                                            Block::Expression(Expression::Identifier("c".into())),
                                            span(7..8),
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
                                span(4..15),
                            ),
                            Spanned::new(
                                Block::Object(vec![
                                    (
                                        Spanned::new("f".into(), span(16..17)),
                                        Spanned::new(
                                            Block::Expression(Expression::Identifier("g".into())),
                                            span(18..19),
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
                                span(15..26),
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

    // a:
    //   - b:
    //       d: e
    //   - f:
    //       h: i
    // j: k
    //

    #[test]
    fn operation_if() {
        let actual = test(vec![
            Token::Identifier("$if".into()),
            Token::Colon,
            Token::Identifier("ready".into()),
            Token::EndOfLine,
            Token::Identifier("then".into()),
            Token::Colon,
            Token::Identifier("go".into()),
            Token::EndOfLine,
            Token::Identifier("else".into()),
            Token::Colon,
            Token::Identifier("stay".into()),
            Token::EndOfLine,
        ]);

        let expected = Ok(Spanned::new(
            Block::Operation(Box::new(BlockOperation::If {
                condition: Spanned::new(
                    Block::Expression(Expression::Identifier("ready".into())),
                    span(2..3),
                ),
                consequent: Some(Spanned::new(
                    Block::Expression(Expression::Identifier("go".into())),
                    span(6..7),
                )),
                alternative: Some(Spanned::new(
                    Block::Expression(Expression::Identifier("stay".into())),
                    span(10..11),
                )),
            })),
            span(0..12),
        ));

        assert_eq!(actual, expected);
    }
}
