use chumsky::prelude::*;

use crate::{Expression, Token};

/*
type Span = std::ops::Range<usize>;
type Spanned<T> = (T, Span);

pub fn parser() -> impl Parser<Token, Spanned<Expression>, Error = Simple<Token>> + Clone {
    recursive(|expr| {
        let raw_expr = recursive(|raw_expr| {
            let primitive_values = select! {
                Token::Null => Expression::Null,
                Token::Boolean(x) => Expression::Boolean(x),
                Token::Number(n) => Expression::Number(n.parse().unwrap()),
                Token::String(s) => Expression::String(s),
            }
            .labelled("value");

            let identifier =
                select! { Token::Identifier(identifier) => Expression::Identifier { name: identifier } }
                    .labelled("identifier");

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
            let atom = primitive_values
                .or(identifier)
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
                        .delimited_by(just(Token::Ctrl('(')), just(Token::Ctrl(')')))
                        .map_with_span(|args, span: Span| (args, span))
                        .repeated(),
                )
                .foldl(|f, args| {
                    let span = f.1.start..args.1.end;
                    (Expression::Call(Box::new(f), args.0), span)
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

pub fn nested_parser<T>(
    parser: impl Parser<Token, T, Error = Simple<Token>> + Clone,
    open: Token,
    close: Token,
    f: impl Fn(Span) -> T + Clone,
) -> impl Parser<Token, T, Error = Simple<Token>> + Clone {
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
*/
