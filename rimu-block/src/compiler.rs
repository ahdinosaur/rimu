use chumsky::prelude::*;

use std::collections::BTreeMap;

use rimu_report::{Span, Spanned};

use crate::lexer::Token;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Block {
    Object(BTreeMap<String, SpannedBlock>),
    List(Vec<SpannedBlock>),
    Expression(String),
}

pub type SpannedBlock = Spanned<Block>;

pub type CompilerError = Simple<Token, Span>;

pub(crate) trait Compiler<T>:
    Parser<Token, T, Error = CompilerError> + Sized + Clone
{
}
impl<P, T> Compiler<T> for P where P: Parser<Token, T, Error = CompilerError> + Clone {}

pub(crate) fn compile(
    tokens: Vec<Spanned<Token>>,
    eoi: Span,
) -> (Option<SpannedBlock>, Vec<CompilerError>) {
    compiler_parser().parse_recovery(chumsky::Stream::from_iter(
        eoi,
        tokens.into_iter().map(|token| token.take()),
    ))
}

fn compiler_parser() -> impl Compiler<SpannedBlock> {
    recursive(|doc| {
        let eol = just(Token::EndOfLine); // .repeated().at_least(1);

        let expr =
            select! { Token::Value(value) => Block::Expression(value) }.then_ignore(eol.clone());

        let list_item = just(Token::ListItem).ignore_then(doc.clone());
        let list = list_item.repeated().at_least(1).map(Block::List);

        let key = select! { Token::Key(key) => key };
        let value_simple = expr.clone().map_with_span(Spanned::new);
        let value_complex = eol
            .then(just(Token::Indent))
            .then(doc.clone())
            .map(|(_, d)| d);
        let value = value_simple.or(value_complex);
        let entry = key.then(value);
        let entries = entry.repeated().at_least(1);
        let object = entries
            .then_ignore(just(Token::Dedent).to(()).or(end()))
            .map(|entries| Block::Object(BTreeMap::from_iter(entries.into_iter())));

        expr.or(list).or(object).map_with_span(Spanned::new)
    })
    .then_ignore(end())
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use chumsky::Parser;
    use map_macro::btree_map;
    use pretty_assertions::assert_eq;
    use rimu_report::{SourceId, Span, Spanned};

    use crate::{compiler::Block, lexer::Token};

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
            Token::ListItem,
            Token::Value("a".into()),
            Token::EndOfLine,
            Token::ListItem,
            Token::Value("b".into()),
            Token::EndOfLine,
            Token::ListItem,
            Token::Value("c".into()),
            Token::EndOfLine,
        ]);

        let expected = Ok(Spanned::new(
            Block::List(vec![
                Spanned::new(Block::Expression("a".into()), span(1..3)),
                Spanned::new(Block::Expression("b".into()), span(4..6)),
                Spanned::new(Block::Expression("c".into()), span(7..9)),
            ]),
            span(0..9),
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn object_simple() {
        let actual = test(vec![
            Token::Key("a".into()),
            Token::Value("b".into()),
            Token::EndOfLine,
            Token::Key("c".into()),
            Token::Value("d".into()),
            Token::EndOfLine,
            Token::Key("e".into()),
            Token::Value("f".into()),
            Token::EndOfLine,
        ]);

        let expected = Ok(Spanned::new(
            Block::Object(btree_map! {
                "a".into() => Spanned::new(Block::Expression("b".into()), span(1..3)),
                "c".into() => Spanned::new(Block::Expression("d".into()), span(4..6)),
                "e".into() => Spanned::new(Block::Expression("f".into()), span(7..9)),
            }),
            span(0..9),
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn something() {
        let actual = test(vec![
            Token::Key("a".into()),
            Token::EndOfLine,
            Token::Indent,
            Token::Key("b".into()),
            Token::EndOfLine,
            Token::Indent,
            Token::ListItem,
            Token::Value("c".into()),
            Token::EndOfLine,
            Token::ListItem,
            Token::Value("d".into()),
            Token::EndOfLine,
            Token::ListItem,
            Token::Key("e".into()),
            Token::Value("f".into()),
            Token::EndOfLine,
            Token::Dedent,
            Token::Key("g".into()),
            Token::Value("h".into()),
            Token::EndOfLine,
            Token::Dedent,
        ]);

        let expected = Ok(Spanned::new(
            Block::Object(btree_map! {
                "a".into() => Spanned::new(Block::Object(btree_map! {
                    "b".into() => Spanned::new(Block::List(vec![
                        Spanned::new(Block::Expression("c".into()), span(7..9)),
                        Spanned::new(Block::Expression("d".into()), span(10..12)),
                        Spanned::new(Block::Object(btree_map! {
                            "e".into() => Spanned::new(Block::Expression("f".into()), span(14..16)),
                        }), span(13..17))
                    ]), span(6..17)),
                    "g".into() => Spanned::new(Block::Expression("h".into()), span(18..20))
                }), span(3..21)),
            }),
            span(0..21),
        ));

        assert_eq!(actual, expected);
    }
}
