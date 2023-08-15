/*

use rimu_span::Spanned;

use crate::{Expression, Operator, Precedence, Token};

#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum ParseError {
    #[error("expected end of expresssion: {token}")]
    ExpectedEndOfExpression { token: Token },
    #[error("expected left side of expresssion: {token}")]
    ExpectedLeftSideOfExpression { token: Token },
    #[error("invalid infix token: {token}")]
    InvalidInfixToken { token: Token },
    #[error("expected identifier: {token}")]
    ExpectedIdentifier { token: Token },
    #[error("expected some token")]
    ExpectedSomeToken,
    #[error("expected token {token}, encountered {current}")]
    ExpectedNextToken { token: Token, current: usize },
    #[error("expected token {token}, encountered end of file")]
    ExpectedNextTokenGotEndOfFile { token: Token },
}

/// From a series of [`Tokens`](Token) parse a structured [`Expression`] tree.
/// # Errors
/// Returns a [`SyntaxError`] when encountering invalid input.
pub fn parse(tokens: Vec<Spanned<Token>>) -> Result<Expression, ParseError> {
    let tokens: Vec<Token> = tokens.into_iter().map(|i| i.into_contents()).collect();
    Parser::new(tokens).parse()
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn parse(&mut self) -> Result<Expression, ParseError> {
        let expression = self.expression()?;

        match self.current() {
            Some(token) => Err(ParseError::ExpectedEndOfExpression {
                token: token.clone(),
            }),
            None => Ok(expression),
        }
    }

    fn expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_precedence(Precedence::Or)
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> Result<Expression, ParseError> {
        self.advance();
        let mut expression = self.do_prefix()?;

        while self
            .current()
            .is_some_and(|t| precedence <= Precedence::from(t))
        {
            self.advance();
            expression = self.do_infix(expression)?;
        }

        Ok(expression)
    }

    fn do_prefix(&mut self) -> Result<Expression, ParseError> {
        let previous = self.previous()?;
        match previous {
            Token::Literal(value) => Ok(Expression::Literal {
                value: value.clone(),
            }),
            Token::Identifier(name) => Ok(Expression::Variable { name: name.clone() }),
            Token::LeftParen => self.grouping(),
            Token::LeftBracket => self.array(),
            Token::Not | Token::Minus => self.unary(),
            _ => Err(ParseError::ExpectedLeftSideOfExpression { token: previous }),
        }
    }

    fn do_infix(&mut self, left: Expression) -> Result<Expression, ParseError> {
        let previous = self.previous()?;
        match previous {
            Token::Minus
            | Token::Plus
            | Token::Star
            | Token::Slash
            | Token::Div
            | Token::Mod
            | Token::Equal
            | Token::NotEqual
            | Token::Greater
            | Token::GreaterEqual
            | Token::Less
            | Token::LessEqual
            | Token::And
            | Token::Or
            | Token::Xor => self.binary(left),
            Token::LeftParen => self.call(left),
            _ => Err(ParseError::InvalidInfixToken { token: previous }),
        }
    }

    fn expression_list(&mut self, end_token: &Token) -> Result<Vec<Expression>, ParseError> {
        let mut expressions: Vec<Expression> = vec![];

        while self.current().is_some_and(|t| t != end_token) {
            expressions.push(self.expression()?);

            if self.current() == Some(&Token::Comma) {
                self.advance();
            }
        }

        self.chomp(end_token)?;

        Ok(expressions)
    }

    fn call(&mut self, left: Expression) -> Result<Expression, ParseError> {
        if let Expression::Identifier { name } = left {
            Ok(Expression::Call {
                function: name,
                params: self.expression_list(&Token::RightParen)?,
            })
        } else {
            Err(ParseError::ExpectedIdentifier {
                token: self.previous()?,
            })
        }
    }

    fn array(&mut self) -> Result<Expression, ParseError> {
        Ok(Expression::Array {
            expressions: self.expression_list(&Token::RightBracket)?,
        })
    }

    fn binary(&mut self, left: Expression) -> Result<Expression, ParseError> {
        let operator = Operator::try_from(self.previous()?)?;
        let right = self.parse_precedence(Precedence::from(self.previous()?).next())?;

        Ok(Expression::Binary {
            left: Box::new(left),
            right: Box::new(right),
            operator,
        })
    }

    fn unary(&mut self) -> Result<Expression, ParseError> {
        let operator = Operator::try_from(self.previous()?)?;
        let right = self.parse_precedence(Precedence::Unary)?;

        Ok(Expression::Unary {
            right: Box::new(right),
            operator,
        })
    }

    fn grouping(&mut self) -> Result<Expression, ParseError> {
        let expression = self.expression()?;
        self.chomp(&Token::RightParen)?;

        Ok(expression)
    }

    fn advance(&mut self) {
        if self.current < self.tokens.len() {
            self.current += 1;
        }
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn previous(&self) -> Result<&Token, ParseError> {
        self.tokens
            .get(self.current - 1)
            .ok_or(ParseError::ExpectedSomeToken)
    }

    fn chomp(&mut self, token: &Token) -> Result<(), ParseError> {
        if self.current() == Some(token) {
            self.advance();
            Ok(())
        } else {
            match self.current() {
                Some(current) => Err(SyntaxError(format!(
                    "Expected {token:?} encountered {current:?}"
                ))),
                None => Err(SyntaxError(format!(
                    "Expected {token:?} encountered end of file"
                ))),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::Expression, error::SyntaxError, operator::Operator, token::Token, value::Value,
    };

    use super::Parser;

    #[test]
    fn single_literal() {
        let ast = Parser::compile_ast(vec![Token::Literal(Value::Boolean(true))]);
        let expected = Expression::Literal {
            value: Value::Boolean(true),
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn single_variable() {
        let ast = Parser::compile_ast(vec![Token::Identifier(String::from("test"))]);
        let expected = Expression::Variable {
            name: String::from("test"),
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn expression_group() {
        let ast = Parser::compile_ast(vec![
            Token::LeftParen,
            Token::Literal(Value::Boolean(true)),
            Token::RightParen,
        ]);
        let expected = Expression::Literal {
            value: Value::Boolean(true),
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn unary_literal() {
        let ast = Parser::compile_ast(vec![Token::Minus, Token::Literal(Value::Number(42.0))]);
        let expected = Expression::Unary {
            right: Box::new(Expression::Literal {
                value: Value::Number(42.0),
            }),
            operator: Operator::Minus,
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn multiply_number() {
        let ast = Parser::compile_ast(vec![
            Token::Literal(Value::Number(3.0)),
            Token::Star,
            Token::Literal(Value::Number(2.0)),
        ]);
        let expected = Expression::Binary {
            left: Box::new(Expression::Literal {
                value: Value::Number(3.0),
            }),
            right: Box::new(Expression::Literal {
                value: Value::Number(2.0),
            }),
            operator: Operator::Multiply,
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn add_number() {
        let ast = Parser::compile_ast(vec![
            Token::Literal(Value::Number(3.0)),
            Token::Plus,
            Token::Literal(Value::Number(2.0)),
        ]);
        let expected = Expression::Binary {
            left: Box::new(Expression::Literal {
                value: Value::Number(3.0),
            }),
            right: Box::new(Expression::Literal {
                value: Value::Number(2.0),
            }),
            operator: Operator::Plus,
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn precedence_multiply_addition() {
        let ast = Parser::compile_ast(vec![
            Token::Literal(Value::Number(1.0)),
            Token::Plus,
            Token::Literal(Value::Number(2.0)),
            Token::Star,
            Token::Literal(Value::Number(3.0)),
        ]);
        let expected = Expression::Binary {
            left: Box::new(Expression::Literal {
                value: Value::Number(1.0),
            }),
            right: Box::new(Expression::Binary {
                left: Box::new(Expression::Literal {
                    value: Value::Number(2.0),
                }),
                right: Box::new(Expression::Literal {
                    value: Value::Number(3.0),
                }),
                operator: Operator::Multiply,
            }),
            operator: Operator::Plus,
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn comparison_equal() {
        let ast = Parser::compile_ast(vec![
            Token::Literal(Value::Number(5.0)),
            Token::Equal,
            Token::Literal(Value::Number(7.0)),
        ]);
        let expected = Expression::Binary {
            left: Box::new(Expression::Literal {
                value: Value::Number(5.0),
            }),
            right: Box::new(Expression::Literal {
                value: Value::Number(7.0),
            }),
            operator: Operator::Equal,
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn boolean_and() {
        let ast = Parser::compile_ast(vec![
            Token::Literal(Value::Boolean(true)),
            Token::And,
            Token::Literal(Value::Boolean(false)),
        ]);
        let expected = Expression::Binary {
            left: Box::new(Expression::Literal {
                value: Value::Boolean(true),
            }),
            right: Box::new(Expression::Literal {
                value: Value::Boolean(false),
            }),
            operator: Operator::And,
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn variable_add() {
        let ast = Parser::compile_ast(vec![
            Token::LeftParen,
            Token::Literal(Value::Number(5.0)),
            Token::Plus,
            Token::Identifier(String::from("SOME_VAR")),
            Token::RightParen,
            Token::Star,
            Token::Literal(Value::Number(4.0)),
        ]);
        let expected = Expression::Binary {
            left: Box::new(Expression::Binary {
                left: Box::new(Expression::Literal {
                    value: Value::Number(5.0),
                }),
                right: Box::new(Expression::Variable {
                    name: String::from("SOME_VAR"),
                }),
                operator: Operator::Plus,
            }),
            right: Box::new(Expression::Literal {
                value: Value::Number(4.0),
            }),
            operator: Operator::Multiply,
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn variable_mul() {
        let ast = Parser::compile_ast(vec![
            Token::Identifier(String::from("SOME_VAR")),
            Token::Star,
            Token::Literal(Value::Number(4.0)),
        ]);
        let expected = Expression::Binary {
            left: Box::new(Expression::Variable {
                name: String::from("SOME_VAR"),
            }),
            right: Box::new(Expression::Literal {
                value: Value::Number(4.0),
            }),
            operator: Operator::Multiply,
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn function_call() {
        let ast = Parser::compile_ast(vec![
            Token::Identifier(String::from("max")),
            Token::LeftParen,
            Token::Literal(Value::Number(1.0)),
            Token::Comma,
            Token::Literal(Value::Number(2.0)),
            Token::RightParen,
        ]);
        let expected = Expression::Call {
            name: String::from("max"),
            params: vec![
                Expression::Literal {
                    value: Value::Number(1.0),
                },
                Expression::Literal {
                    value: Value::Number(2.0),
                },
            ],
        };

        assert_eq!(ast, Ok(expected));
    }

    #[test]
    fn err_open_function_call() {
        let ast = Parser::compile_ast(vec![Token::Identifier("max".to_string()), Token::LeftParen]);

        let expected = SyntaxError("Expected RightParen encountered end of file".to_string());

        assert_eq!(ast, Err(expected));
    }

    #[test]
    fn err_open_array() {
        let ast = Parser::compile_ast(vec![Token::LeftBracket, Token::Literal(Value::Nil)]);

        let expected = SyntaxError("Expected RightBracket encountered end of file".to_string());
        assert_eq!(ast, Err(expected));
    }

    #[test]
    fn err_array_empty_expressions() {
        let ast = Parser::compile_ast(vec![Token::LeftBracket, Token::Comma, Token::RightBracket]);

        let expected = SyntaxError("Expected left side of expression got \"Comma\"".to_string());
        assert_eq!(ast, Err(expected));
    }
}

*/

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
