// with help from
//
// - https://github.com/zesterer/chumsky/blob/40fe7d1966f375b3c676d01e04c5dca08f7615ac/examples/nano_rust.rs
// - https://github.com/zesterer/tao/blob/6e7be425ba98cb36582b9c836b3b5b120d13194a/syntax/src/parse.rs
// - https://github.com/noir-lang/noir/blob/master/crates/noirc_frontend/src/parser/parser.rs
// - https://github.com/DennisPrediger/SLAC/blob/main/src/compiler.rs

use chumsky::prelude::*;

use crate::{BinaryOperator, Expression, Span, SpannedExpression, Token, UnaryOperator};

pub type CompilerError = Simple<Token, Span>;

pub trait Compiler<T>: Parser<Token, T, Error = CompilerError> + Sized + Clone {}
impl<P, T> Compiler<T> for P where P: Parser<Token, T, Error = CompilerError> + Clone {}

pub fn compile(source: Vec<Token>) -> Result<SpannedExpression, Vec<CompilerError>> {
    compiler_parser().parse(source)
}

pub fn compiler_parser() -> impl Compiler<SpannedExpression> {
    recursive(|expr| {
        // Begin precedence order:

        // Highest precedence are "primary" atoms
        let atom = atom_parser(expr.clone());

        // Next precedence: right unary (function calls or member get)
        let right_unary = right_unary_parser(expr, atom);

        // Next precedence: "left unary" operators ("-", "not")
        let op = just(Token::Minus)
            .to(UnaryOperator::Negative)
            .or(just(Token::Not).to(UnaryOperator::Not))
            .map_with_span(|expr, span| (expr, span))
            .labelled("unary operator");
        let left_unary = op
            .repeated()
            .then(right_unary.labelled("unary right operand"))
            .clone()
            .foldr(|op, expr| {
                let span = union(op.1, expr.1.clone());
                (
                    Expression::Unary {
                        operator: op.0,
                        right: Box::new(expr),
                    },
                    span,
                )
            })
            .boxed();

        // Next precedence: "factor" operators: "*", "/", "div", "mod"
        let op = just(Token::Star)
            .to(BinaryOperator::Multiply)
            .or(just(Token::Slash).to(BinaryOperator::Divide))
            .or(just(Token::Div).to(BinaryOperator::Div))
            .or(just(Token::Mod).to(BinaryOperator::Mod))
            .labelled("binary (factor) operator");
        let factor = binary_operator_parser(left_unary, op);

        // Next precedence: "term" operators: "+", "-"
        let op = just(Token::Plus)
            .to(BinaryOperator::Add)
            .or(just(Token::Minus).to(BinaryOperator::Subtract))
            .labelled("binary (term) operator");
        let term = binary_operator_parser(factor, op);

        // Next precedence: "comparison" operators: ">", ">=", "<", "<="
        let op = just(Token::Less)
            .to(BinaryOperator::Less)
            .or(just(Token::LessEqual).to(BinaryOperator::LessEqual))
            .or(just(Token::Greater).to(BinaryOperator::Greater))
            .or(just(Token::GreaterEqual).to(BinaryOperator::GreaterEqual))
            .labelled("binary (comparison) operator");
        let comparison = binary_operator_parser(term, op);

        // Next precedence: "equality" operators: "=", "!="
        let op = just(Token::Equal)
            .to(BinaryOperator::Equal)
            .or(just(Token::NotEqual).to(BinaryOperator::NotEqual))
            .labelled("binary (equality) operator");
        let equality = binary_operator_parser(comparison, op);

        // Next precedence: "xor" operator
        let op = just(Token::Xor)
            .to(BinaryOperator::Xor)
            .labelled("binary (xor) operator");
        let xor = binary_operator_parser(equality, op);

        // Next precedence: "and" operator
        let op = just(Token::And)
            .to(BinaryOperator::And)
            .labelled("binary (and) operator");
        let and = binary_operator_parser(xor, op);

        // Next precedence: "or" operator
        let op = just(Token::Or)
            .to(BinaryOperator::Or)
            .labelled("binary (or) operator");
        let or = binary_operator_parser(and, op);

        or
    })
}

fn atom_parser<'a>(
    expr: impl Compiler<SpannedExpression> + 'a,
) -> impl Compiler<SpannedExpression> + 'a {
    let scalar = scalar_parser();
    let identifier = identifier_parser();
    let list = list_parser(expr.clone());
    let object = object_parser(expr.clone());

    let nested_expr = nested_parser(
        expr.clone().map(|spanned| spanned.0),
        Token::LeftParen,
        Token::RightParen,
        |_| Expression::Error,
    );

    scalar
        .or(identifier)
        .or(list)
        .or(object)
        .or(nested_expr)
        .map_with_span(|e, s| (e, s))
        .boxed()
}

fn scalar_parser() -> impl Compiler<Expression> {
    select! {
        Token::Null => Expression::Null,
        Token::Boolean(x) => Expression::Boolean(x),
        Token::Number(n) => Expression::Number(n),
        Token::String(s) => Expression::String(s),
    }
    .labelled("scalar")
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

fn items_parser<'a>(
    expr: impl Compiler<SpannedExpression> + 'a,
) -> impl Compiler<Option<Vec<SpannedExpression>>> + 'a {
    expr.separated_by(just(Token::Comma))
        .allow_trailing()
        .map(Some)
        .boxed()
}

fn list_parser<'a>(expr: impl Compiler<SpannedExpression> + 'a) -> impl Compiler<Expression> + 'a {
    let items = items_parser(expr);
    nested_parser(items, Token::LeftBrack, Token::RightBrack, |_| None)
        .map(|x| match x {
            Some(items) => Expression::List(items),
            None => Expression::Error,
        })
        .labelled("list")
}

fn object_parser<'a>(
    expr: impl Compiler<SpannedExpression> + 'a,
) -> impl Compiler<Expression> + 'a {
    nested_parser(
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
    .labelled("object")
}

fn right_unary_parser<'a>(
    expr: impl Compiler<SpannedExpression> + 'a,
    atom: impl Compiler<SpannedExpression> + 'a,
) -> impl Compiler<SpannedExpression> + 'a {
    let items = items_parser(expr);
    enum RightUnary {
        Call(Vec<SpannedExpression>),
        GetIndex(SpannedExpression),
        GetKey(SpannedExpression),
        GetSlice(Option<SpannedExpression>, Option<SpannedExpression>),
    }
    let call = items
        .clone()
        .delimited_by(just(Token::LeftParen), just(Token::RightParen))
        .map(|expr| RightUnary::Call(expr.unwrap_or(vec![])));
    let get_index = atom
        .clone()
        .delimited_by(just(Token::LeftBrack), just(Token::RightBrack))
        .map(RightUnary::GetIndex);
    let get_key = just(Token::Dot)
        .then(identifier_parser().map_with_span(|expr, span| (expr, span)))
        .map(|(_, expr)| RightUnary::GetKey(expr));
    let get_slice = atom
        .clone()
        .or_not()
        .then(just(Token::Colon))
        .then(atom.clone().or_not())
        .delimited_by(just(Token::LeftBrack), just(Token::RightBrack))
        .map(|((start, _), end)| RightUnary::GetSlice(start, end));

    atom.then(
        call.or(get_index)
            .or(get_key)
            .or(get_slice)
            .map_with_span(|args, span: Span| (args, span))
            .repeated(),
    )
    .foldl(|left, right| {
        let span = union(left.1.clone(), right.1);
        match right.0 {
            RightUnary::Call(args) => (
                Expression::Call {
                    function: Box::new(left),
                    args,
                },
                span,
            ),
            RightUnary::GetIndex(index) => (
                Expression::GetIndex {
                    container: Box::new(left),
                    index: Box::new(index),
                },
                span,
            ),
            RightUnary::GetKey(key) => (
                Expression::GetKey {
                    container: Box::new(left),
                    key: Box::new(key),
                },
                span,
            ),
            RightUnary::GetSlice(start, end) => (
                Expression::GetSlice {
                    container: Box::new(left),
                    start: start.map(Box::new),
                    end: end.map(Box::new),
                },
                span,
            ),
        }
    })
    .boxed()
}

fn binary_operator_parser<'a>(
    prev: impl Compiler<SpannedExpression> + 'a,
    op: impl Compiler<BinaryOperator> + 'a,
) -> impl Compiler<SpannedExpression> + 'a {
    prev.clone()
        .labelled("left operand")
        .then(op.then(prev.clone().labelled("right operand")).repeated())
        .foldl(|left, (op, right)| {
            let span = union(left.1.clone(), right.1.clone());
            (
                Expression::Binary {
                    left: Box::new(left),
                    operator: op,
                    right: Box::new(right),
                },
                span,
            )
        })
        .boxed()
}

fn union(a: Span, b: Span) -> Span {
    a.start..b.end
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rust_decimal::{prelude::FromPrimitive, Decimal};

    use crate::{BinaryOperator, UnaryOperator};

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
    fn expression_group() {
        let actual = compile(vec![
            Token::LeftParen,
            Token::Boolean(true),
            Token::RightParen,
        ]);
        let expected = Ok((Expression::Boolean(true), 0..3));

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
    fn negate_number() {
        let one = Decimal::from_u8(1).unwrap();
        let tokens = vec![Token::Minus, Token::Number(one)];
        let actual = compile(tokens);

        let expected = Ok((
            Expression::Unary {
                operator: UnaryOperator::Negative,
                right: Box::new((Expression::Number(one), (1..2))),
            },
            0..2,
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn add_numbers() {
        let one = Decimal::from_u8(1).unwrap();
        let tokens = vec![Token::Number(one), Token::Plus, Token::Number(one)];
        let actual = compile(tokens);

        let expected = Ok((
            Expression::Binary {
                left: Box::new((Expression::Number(one), (0..1))),
                operator: BinaryOperator::Add,
                right: Box::new((Expression::Number(one), (2..3))),
            },
            0..3,
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn precedence_multiply_addition() {
        let one = Decimal::from_u8(1).unwrap();
        let two = Decimal::from_u8(2).unwrap();
        let three = Decimal::from_u8(3).unwrap();

        let actual = compile(vec![
            Token::Number(one),
            Token::Plus,
            Token::Number(two),
            Token::Star,
            Token::Number(three),
        ]);
        let expected = Ok((
            Expression::Binary {
                left: Box::new((Expression::Number(one), 0..1)),
                operator: BinaryOperator::Add,
                right: Box::new((
                    Expression::Binary {
                        left: Box::new((Expression::Number(two), 2..3)),
                        operator: BinaryOperator::Multiply,
                        right: Box::new((Expression::Number(three), 4..5)),
                    },
                    2..5,
                )),
            },
            0..5,
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn get_index() {
        let one = Decimal::from_u8(1).unwrap();
        let tokens = vec![
            Token::Identifier("a".into()),
            Token::LeftBrack,
            Token::Number(one),
            Token::RightBrack,
        ];
        let actual = compile(tokens);

        let expected = Ok((
            Expression::GetIndex {
                container: Box::new((Expression::Identifier("a".into()), 0..1)),
                index: Box::new((Expression::Number(one), 2..3)),
            },
            0..4,
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn get_key() {
        let tokens = vec![
            Token::Identifier("a".into()),
            Token::Dot,
            Token::Identifier("b".into()),
        ];
        let actual = compile(tokens);

        let expected = Ok((
            Expression::GetKey {
                container: Box::new((Expression::Identifier("a".into()), 0..1)),
                key: Box::new((Expression::Identifier("b".into()), 2..3)),
            },
            0..3,
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn get_slice() {
        let one = Decimal::from_u8(1).unwrap();
        let two = Decimal::from_u8(2).unwrap();
        let tokens = vec![
            Token::Identifier("a".into()),
            Token::LeftBrack,
            Token::Number(one),
            Token::Colon,
            Token::Number(two),
            Token::RightBrack,
        ];
        let actual = compile(tokens);

        let expected = Ok((
            Expression::GetSlice {
                container: Box::new((Expression::Identifier("a".into()), 0..1)),
                start: Some(Box::new((Expression::Number(one), 2..3))),
                end: Some(Box::new((Expression::Number(two), 4..5))),
            },
            0..6,
        ));

        assert_eq!(actual, expected);
    }
}
