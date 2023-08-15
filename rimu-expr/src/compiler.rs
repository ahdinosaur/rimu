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
    recursive(|_expr| {
        let atom = literal_parser()
            .or(identifier_parser())
            .map_with_span(|e, s| (e, s));
        atom
    })
}

/*
pub fn compiler() -> impl Compiler<Expression> {
    recursive(|expr| {
        let raw_expr = recursive(|raw_expr| {
            // A list of expressions
            let items = expr
                .clone()
                .separated_by(just(Token::Comma))
                .allow_trailing();

            let list = items
                .clone()
                .delimited_by(just(Token::LeftBrack), just(Token::RightBrack))
                .map(Expression::List);

            // 'Atoms' are expressions that contain no ambiguity
            let atom = primitive()
                .or(identifier())
                .or(list)
                .map_with_span(|expr, span| (expr, span))
                // Atoms can also just be normal expressions, but surrounded with parentheses
                .or(expr
                    .clone()
                    .delimited_by(just(Token::LeftParen), just(Token::RightParen)))
                // Attempt to recover anything that looks like a parenthesised expression but contains errors
                .recover_with(nested_delimiters(
                    Token::LeftParen,
                    Token::RightParen,
                    [
                        (Token::LeftBrack, Token::RightBrack),
                        (Token::LeftBrace, Token::RightBrace),
                    ],
                    |span| (Expression::Error, span),
                ))
                // Attempt to recover anything that looks like a list but contains errors
                .recover_with(nested_delimiters(
                    Token::LeftBrack,
                    Token::RightBrack,
                    [
                        (Token::LeftParen, Token::RightParen),
                        (Token::LeftBrace, Token::RightBrace),
                    ],
                    |span| (Expression::Error, span),
                ));

            // Function calls have very high precedence so we prioritise them
            let call = atom
                .then(
                    items
                        .delimited_by(just(Token::LeftParen), just(Token::RightParen))
                        .map_with_span(|args, span: Span| (args, span))
                        .repeated(),
                )
                .foldl(|f, args| {
                    let span = f.1.start..args.1.end;
                    (
                        Expression::Call {
                            function: Box::new(f),
                            args: args.0,
                        },
                        span,
                    )
                });

            // Product ops (multiply and divide) have equal precedence
            let op = just(Token::Op("*".to_string()))
                .to(BinaryOp::Mul)
                .or(just(Token::Op("/".to_string())).to(BinaryOp::Div));
            let product = call
                .clone()
                .then(op.then(call).repeated())
                .foldl(|a, (op, b)| {
                    let span = a.1.start..b.1.end;
                    (Expression::Binary(Box::new(a), op, Box::new(b)), span)
                });

            // Sum ops (add and subtract) have equal precedence
            let op = just(Token::Op("+".to_string()))
                .to(BinaryOp::Add)
                .or(just(Token::Op("-".to_string())).to(BinaryOp::Sub));
            let sum = product
                .clone()
                .then(op.then(product).repeated())
                .foldl(|a, (op, b)| {
                    let span = a.1.start..b.1.end;
                    (Expression::Binary(Box::new(a), op, Box::new(b)), span)
                });

            // Comparison ops (equal, not-equal) have equal precedence
            let op = just(Token::Op("==".to_string()))
                .to(BinaryOp::Eq)
                .or(just(Token::Op("!=".to_string())).to(BinaryOp::NotEq));
            let compare = sum
                .clone()
                .then(op.then(sum).repeated())
                .foldl(|a, (op, b)| {
                    let span = a.1.start..b.1.end;
                    (Expression::Binary(Box::new(a), op, Box::new(b)), span)
                });

            compare
        });

        // Blocks are expressions but delimited with braces
        let block = expr
            .clone()
            .delimited_by(just(Token::Ctrl('{')), just(Token::Ctrl('}')))
            // Attempt to recover anything that looks like a block but contains errors
            .recover_with(nested_delimiters(
                Token::Ctrl('{'),
                Token::Ctrl('}'),
                [
                    (Token::Ctrl('('), Token::Ctrl(')')),
                    (Token::Ctrl('['), Token::Ctrl(']')),
                ],
                |span| (Expression::Error, span),
            ));

        let if_ = recursive(|if_| {
            just(Token::If)
                .ignore_then(expr.clone())
                .then(block.clone())
                .then(
                    just(Token::Else)
                        .ignore_then(block.clone().or(if_))
                        .or_not(),
                )
                .map_with_span(|((cond, a), b), span: Span| {
                    (
                        Expression::If(
                            Box::new(cond),
                            Box::new(a),
                            Box::new(match b {
                                Some(b) => b,
                                // If an `if` expression has no trailing `else` block, we magic up one that just produces null
                                None => (Expression::Value(Value::Null), span.clone()),
                            }),
                        ),
                        span,
                    )
                })
        });

        // Both blocks and `if` are 'block expressions' and can appear in the place of statements
        let block_expr = block.or(if_).labelled("block");

        let block_chain = block_expr
            .clone()
            .then(block_expr.clone().repeated())
            .foldl(|a, b| {
                let span = a.1.start..b.1.end;
                (Expression::Then(Box::new(a), Box::new(b)), span)
            });

        block_chain
            // Expressionessions, chained by semicolons, are statements
            .or(raw_expr.clone())
            .then(just(Token::Ctrl(';')).ignore_then(expr.or_not()).repeated())
            .foldl(|a, b| {
                // This allows creating a span that covers the entire Then expression.
                // b_end is the end of b if it exists, otherwise it is the end of a.
                let a_start = a.1.start;
                let b_end = b.as_ref().map(|b| b.1.end).unwrap_or(a.1.end);
                (
                    Expression::Then(
                        Box::new(a),
                        Box::new(match b {
                            Some(b) => b,
                            // Since there is no b expression then its span is empty.
                            None => (Expression::Value(Value::Null), b_end..b_end),
                        }),
                    ),
                    a_start..b_end,
                )
            })
    })
}
*/

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
    select! { Token::Identifier(identifier) => Expression::Identifier { name: identifier } }
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
    fn basic_operation() {
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
