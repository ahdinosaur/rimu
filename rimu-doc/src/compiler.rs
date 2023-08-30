use chumsky::prelude::*;

use std::collections::BTreeMap;

use rimu_report::{SourceId, Span, Spanned};

use crate::lexer::Token;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Doc {
    Object(BTreeMap<String, SpannedDoc>),
    List(Vec<SpannedDoc>),
    Expression(String),
}

pub type SpannedDoc = Spanned<Doc>;

pub type CompilerError = Simple<Token, Span>;

pub trait Compiler<T>: Parser<Token, T, Error = CompilerError> + Sized + Clone {}
impl<P, T> Compiler<T> for P where P: Parser<Token, T, Error = CompilerError> + Clone {}

pub fn compile(tokens: Vec<Token>, source: SourceId) -> Result<SpannedDoc, Vec<CompilerError>> {
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

pub fn compiler_parser() -> impl Compiler<SpannedDoc> {
    recursive(|doc| {
        let eol = just(Token::EndOfLine); // .repeated().at_least(1);

        let expr =
            select! { Token::Value(value) => Doc::Expression(value) }.then_ignore(eol.clone());

        let list_item = just(Token::ListItem).ignore_then(doc.clone());
        let list = list_item.repeated().at_least(1).map(Doc::List);

        let key = select! { Token::Key(key) => key };
        let value_simple = expr.clone().map_with_span(Spanned::new);
        let value_complex = eol
            .clone()
            .then(just(Token::Indent))
            .then(doc.clone())
            .map(|(_, d)| d);
        let value = value_simple.or(value_complex);
        let entry = key.then(value);
        let entries = entry.repeated().at_least(1);
        let object = entries
            .then_ignore(just(Token::Dedent).or(just(Token::EndOfInput)))
            .map(|entries| Doc::Object(BTreeMap::from_iter(entries.into_iter())));

        expr.or(list).or(object).map_with_span(Spanned::new)
    })
    // TODO fix list test with this enabled
    // .then_ignore(end())
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use map_macro::btree_map;
    use pretty_assertions::assert_eq;
    use rimu_report::{SourceId, Span, Spanned};

    use crate::{compiler::Doc, lexer::Token};

    use super::{compile, CompilerError, SpannedDoc};

    fn span(range: Range<usize>) -> Span {
        Span::new(SourceId::empty(), range.start, range.end)
    }

    fn test(tokens: Vec<Token>) -> Result<SpannedDoc, Vec<CompilerError>> {
        compile(tokens, SourceId::empty())
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
            Token::EndOfInput,
        ]);

        let expected = Ok(Spanned::new(
            Doc::List(vec![
                Spanned::new(Doc::Expression("a".into()), span(1..3)),
                Spanned::new(Doc::Expression("b".into()), span(4..6)),
                Spanned::new(Doc::Expression("c".into()), span(7..9)),
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
            Token::EndOfInput,
        ]);

        let expected = Ok(Spanned::new(
            Doc::Object(btree_map! {
                "a".into() => Spanned::new(Doc::Expression("b".into()), span(1..3)),
                "c".into() => Spanned::new(Doc::Expression("d".into()), span(4..6)),
                "e".into() => Spanned::new(Doc::Expression("f".into()), span(7..9)),
            }),
            span(0..10),
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
            Token::EndOfInput,
        ]);

        let expected = Ok(Spanned::new(
            Doc::Object(btree_map! {
                "a".into() => Spanned::new(Doc::Object(btree_map! {
                    "b".into() => Spanned::new(Doc::List(vec![
                        Spanned::new(Doc::Expression("c".into()), span(7..9)),
                        Spanned::new(Doc::Expression("d".into()), span(10..12)),
                        Spanned::new(Doc::Object(btree_map! {
                            "e".into() => Spanned::new(Doc::Expression("f".into()), span(14..16)),
                        }), span(13..17))
                    ]), span(6..17)),
                    "g".into() => Spanned::new(Doc::Expression("h".into()), span(18..20))
                }), span(3..21)),
            }),
            span(0..22),
        ));

        assert_eq!(actual, expected);
    }
}
