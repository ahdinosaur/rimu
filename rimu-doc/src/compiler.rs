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
    recursive(|expr| expr)
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use map_macro::btree_map;
    use pretty_assertions::assert_eq;
    use rimu_report::{SourceId, Span, Spanned};

    use crate::{
        compiler::Doc,
        lexer::{tokenize, Token},
    };

    use super::{compile, CompilerError, SpannedDoc};

    fn span(range: Range<usize>) -> Span {
        Span::new(SourceId::empty(), range.start, range.end)
    }

    fn test(tokens: Vec<Token>) -> Result<SpannedDoc, Vec<CompilerError>> {
        compile(tokens, SourceId::empty())
    }

    #[test]
    fn something() {
        let actual = test(vec![
            Token::Key("a".into()),
            Token::Indent,
            Token::Key("b".into()),
            /*
            a:
              b:
                - c
                - d
                - e: f
              g: h
            */
        ]);

        let expected = Ok(Spanned::new(
            Doc::Object(btree_map! {
                "a".into() => Spanned::new(Doc::Object(btree_map! {
                    "b".into() => Spanned::new(Doc::List(vec![
                        Spanned::new(Doc::Expression("c".into()), span(0..0)),
                        Spanned::new(Doc::Expression("d".into()), span(0..0)),
                        Spanned::new(Doc::Object(btree_map! {
                            "e".into() => Spanned::new(Doc::Expression("f".into()), span(0..0))
                        }), span(0..0))
                    ]), span(0..0)),
                }), span(0..0)),
            }),
            span(0..0),
        ));

        assert_eq!(actual, expected);
    }
}
