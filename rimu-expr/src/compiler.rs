// with help from
//
// - https://github.com/zesterer/chumsky/blob/40fe7d1966f375b3c676d01e04c5dca08f7615ac/examples/nano_rust.rs
// - https://github.com/zesterer/tao/blob/6e7be425ba98cb36582b9c836b3b5b120d13194a/syntax/src/parse.rs
// - https://github.com/noir-lang/noir/blob/master/crates/noirc_frontend/src/parser/parser.rs

use chumsky::prelude::*;
use rust_decimal::Decimal;
use std::{ops::Range, str::FromStr};

use crate::{Expression, SpannedExpression, Token};

type Span = Range<usize>;
type CompilerError = Simple<Token, Span>;

pub trait Compiler<T>: Parser<Token, T, Error = CompilerError> + Sized + Clone {}
impl<P, T> Compiler<T> for P where P: Parser<Token, T, Error = CompilerError> + Clone {}

pub fn compile(source: Vec<Token>) -> Result<SpannedExpression, Vec<CompilerError>> {
    compiler().parse(source)
}

pub fn compiler() -> impl Compiler<SpannedExpression> {
    recursive(|expr| {
        let literal = literal_parser();

        let identifier = identifier_parser();

        let items = expr
            .clone()
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .map(Some);

        let list = nested_parser(items.clone(), Token::LeftBrack, Token::RightBrack, |_| None)
            .map(|x| match x {
                Some(items) => Expression::List(items),
                None => Expression::Error,
            })
            .labelled("list");

        let object = nested_parser(
            identifier_parser()
                .map_with_span(|expr, span| (expr, span))
                .then(just(Token::Colon).ignore_then(expr.clone().or_not()))
                .map(|(field, value)| match value {
                    Some(value) => (field, value),
                    None => (field.clone(), field.clone()),
                })
                .separated_by(just(Token::Comma))
                .allow_trailing()
                .map(Some)
                .boxed(),
            Token::LeftBrace,
            Token::RightBrace,
            |_| None,
        )
        .map(|fields| fields.map(Expression::Object).unwrap_or(Expression::Error))
        .labelled("object");

        // Begin precedence order:

        // Highest precedence are "primary" literals
        let atom = literal
            .or(identifier)
            .or(list)
            .or(object)
            .map_with_span(|e, s| (e, s));

        // Next precedence: function "calls"
        let call = atom
            .then(
                items
                    .clone()
                    .delimited_by(just(Token::LeftParen), just(Token::RightParen))
                    .map_with_span(|args, span: Span| (args, span))
                    .repeated(),
            )
            .foldl(|f, args| {
                let span = f.1.start..args.1.end;
                (
                    Expression::Call {
                        function: Box::new(f),
                        args: args.0.unwrap_or(vec![]),
                    },
                    span,
                )
            });

        /*
        // Next precedence: "unary" operators
        let op = just(Token::Op(Op::Sub))
            .to(ast::UnaryOp::Neg)
            .or(just(Token::Op(Op::Not)).to(ast::UnaryOp::Not))
            .map_with_span(SrcNode::new);
        */

        call
    })
}

fn literal_parser() -> impl Compiler<Expression> {
    select! {
        Token::Null => Expression::Null,
        Token::Boolean(x) => Expression::Boolean(x),
        Token::Number(n) => Expression::Number(n),
        Token::String(s) => Expression::String(s),
    }
    .labelled("primitive")
}

fn identifier_parser() -> impl Compiler<Expression> {
    select! { Token::Identifier(identifier) => Expression::Identifier(identifier) }
        .labelled("identifier")
}

fn nested_parser<'a, T: 'a>(
    parser: impl Compiler<T> + 'a,
    open: Token,
    close: Token,
    f: impl Fn(Span) -> T + Clone + 'a,
) -> impl Compiler<T> + 'a {
    parser
        .delimited_by(just(open.clone()), just(close.clone()))
        .recover_with(nested_delimiters(
            open.clone(),
            close.clone(),
            [
                (Token::LeftParen, Token::RightParen),
                (Token::LeftBrack, Token::RightBrack),
                (Token::LeftBrace, Token::RightBrace),
            ],
            f,
        ))
        .boxed()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rust_decimal::{prelude::FromPrimitive, Decimal};

    use crate::Operator;

    use super::{compile, Expression, Token};

    #[test]
    fn empty_input() {
        let actual = compile(vec![]);

        assert!(actual.is_err());
    }

    #[test]
    fn simple_null() {
        let actual = compile(vec![Token::Null]);

        let expected = Ok((Expression::Null, 0..1));

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_bool() {
        let actual = compile(vec![Token::Boolean(false)]);

        let expected = Ok((Expression::Boolean(false), 0..1));

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_number() {
        let number = Decimal::from_u32(9001).unwrap();
        let actual = compile(vec![Token::Number(number)]);

        let expected = Ok((Expression::Number(number), 0..1));

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_list() {
        let actual = compile(vec![
            Token::LeftBrack,
            Token::String("hello".into()),
            Token::Comma,
            Token::Boolean(true),
            Token::Comma,
            Token::String("world".into()),
            Token::Comma,
            Token::RightBrack,
        ]);

        let expected = Ok((
            Expression::List(vec![
                (Expression::String("hello".into()), 1..2),
                (Expression::Boolean(true), 3..4),
                (Expression::String("world".into()), 5..6),
            ]),
            0..8,
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_object() {
        let actual = compile(vec![
            Token::LeftBrace,
            Token::Identifier("a".into()),
            Token::Colon,
            Token::String("hello".into()),
            Token::Comma,
            Token::Identifier("b".into()),
            Token::Colon,
            Token::String("world".into()),
            Token::Comma,
            Token::RightBrace,
        ]);

        let expected = Ok((
            Expression::Object(vec![
                (
                    (Expression::Identifier("a".into()), 1..2),
                    (Expression::String("hello".into()), 3..4),
                ),
                (
                    (Expression::Identifier("b".into()), 5..6),
                    (Expression::String("world".into()), 7..8),
                ),
            ]),
            0..10,
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_function_call() {
        let actual = compile(vec![
            Token::Identifier("add".into()),
            Token::LeftParen,
            Token::Identifier("a".into()),
            Token::Comma,
            Token::Identifier("b".into()),
            Token::Comma,
            Token::RightParen,
        ]);

        let expected = Ok((
            Expression::Call {
                function: Box::new((Expression::Identifier("add".into()), 0..1)),
                args: vec![
                    (Expression::Identifier("a".into()), 2..3),
                    (Expression::Identifier("b".into()), 4..5),
                ],
            },
            0..7,
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_operation() {
        let one = Decimal::from_u8(1).unwrap();
        let tokens = vec![Token::Number(one), Token::Plus, Token::Number(one)];
        let actual = compile(tokens);

        let expected = Ok((
            Expression::Binary {
                left: Box::new((Expression::Number(one), (0..1))),
                operator: Operator::Plus,
                right: Box::new((Expression::Number(one), (2..3))),
            },
            0..3,
        ));

        assert_eq!(actual, expected);
    }
}
