//
// TODO
//
// - Block replaces rimu Template
// - block Token is same as expr token, plus Indent, Dedent, and EndOfLine
// - block line lexer uses expr lexer
// - block parser uses expr parser
// - handle operations in chumsky with chumsky::try_map and rimu's existing operation parse code.
// - also be sure to keep TryFrom<Value> for Block, like Template had
//
// ----
//
// OR...
//
// maybe we don't need to embed lexers and parsers. just use in the outer tokenize code.
//
//
// ---
//
// then later
//
// - rimu-eval evaluates either block or expr
//

use rimu_report::{SourceId, Span};

use crate::block::SpannedBlock;
use crate::compiler::{compile, CompilerError};
use crate::lexer::{tokenize, LexerError};

mod block;
mod compiler;
mod lexer;
mod operation;

pub enum Error {
    Lexer(LexerError),
    Compiler(CompilerError),
}

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

    use map_macro::btree_map;
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
    fn arithmetic() {
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
            Block::Object(btree_map! {
                Spanned::new("a".into(), span(1..2)) => Spanned::new(Block::Object(btree_map! {
                    Spanned::new("b".into(), span(6..7)) => Spanned::new(
                        Block::List(vec![
                            Spanned::new(Block::Expression(Expression::Binary {
                                left: Box::new(Spanned::new(Expression::Identifier("c".into()), span(15..16))),
                                right: Box::new(Spanned::new(Expression::Identifier("d".into()), span(19..20))),
                                operator: BinaryOperator::Add
                            }), span(15..20)),
                            Spanned::new(
                                Block::Object(btree_map! {
                                    Spanned::new("e".into(), span(27..28)) => {
                                        Spanned::new(Block::Expression(Expression::Identifier("f".into())), span(30..31))
                                    },
                                }),
                                span(27..34)
                            )
                        ]),
                        span(13..34)
                    ),
                    Spanned::new("g".into(), span(34..35)) => {
                        Spanned::new(Block::Expression(Expression::Identifier("h".into())), span(37..38))
                    },
                }), span(6..39)),
            }),
            span(1..39),
        ));

        assert_eq!(actual_expr, expected_expr);
        assert_eq!(errors.len(), 0);
    }
}
