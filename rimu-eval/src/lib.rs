// with help from
// - https://github.com/DennisPrediger/SLAC/blob/main/src/interpreter.rs

use rimu_env::{Environment, EnvironmentError};
use rimu_expr::{BinaryOperator, Expression, SpannedExpression, UnaryOperator};
use rimu_value::{Number, Object, Value};

#[derive(Debug, thiserror::Error, Clone, PartialEq, PartialOrd)]
pub enum EvalError {
    #[error("environment error: {0}")]
    Environment(#[source] EnvironmentError),
    #[error("missing context: {var}")]
    MissingEnvironment { var: String },
    #[error("tried to call non-function")]
    CallNonFunction, // { expr: SpannedExpression },
    #[error("error expression")]
    ErrorExpression,
}

/// A tree walking interpreter which given an [`Environment`] and an [`Expression`]
/// recursivly walks the tree and computes a single [`Value`].
pub struct Evaluator<'a> {
    env: &'a Environment<'a>,
}

impl<'a> Evaluator<'a> {
    pub fn new(env: &'a Environment) -> Self {
        Self { env }
    }

    pub fn evaluate(
        expression: &SpannedExpression,
        env: &'a Environment<'a>,
    ) -> Result<Value, EvalError> {
        Self::new(env).expression(expression)
    }

    pub fn expression(&self, expr: &SpannedExpression) -> Result<Value, EvalError> {
        let _span = expr.span();
        let value = match expr.inner() {
            Expression::Null => Value::Null,

            Expression::Boolean(boolean) => Value::Boolean(*boolean),

            Expression::String(string) => Value::String(string.clone()),

            Expression::Number(decimal) => Value::Number(Into::<Number>::into(*decimal)),

            Expression::List(items) => self.list(&items)?,

            Expression::Object(entries) => self.object(&entries)?,

            Expression::Identifier(var) => self.variable(var)?,

            Expression::Unary { right, operator } => self.unary(right.as_ref(), &operator)?,

            Expression::Binary {
                left,
                right,
                operator,
            } => self.binary(&left, &operator, &right)?,

            Expression::Call { function, args } => self.call(function.as_ref(), &args)?,

            Expression::GetIndex { container, index } => todo!(),

            Expression::GetKey { container, key } => todo!(),

            Expression::GetSlice {
                container,
                start,
                end,
            } => todo!(),

            Expression::Error => todo!(),
        };

        Ok(value)
    }

    pub fn unary(
        &self,
        right: &SpannedExpression,
        operator: &UnaryOperator,
    ) -> Result<Value, EvalError> {
        let right = self.expression(right)?;
        let value = match operator {
            UnaryOperator::Negative => -right,
            UnaryOperator::Not => !right,
        };
        Ok(value)
    }

    pub fn binary(
        &self,
        left: &SpannedExpression,
        operator: &BinaryOperator,
        right: &SpannedExpression,
    ) -> Result<Value, EvalError> {
        let left = self.expression(&*left)?;
        let value = match operator {
            BinaryOperator::And => self.boolean(left, right, true)?,
            BinaryOperator::Or => self.boolean(left, right, false)?,
            _ => {
                let right = self.expression(&*right)?;
                match operator {
                    BinaryOperator::Or => unreachable!(),
                    BinaryOperator::And => unreachable!(),
                    BinaryOperator::Add => left + right,
                    BinaryOperator::Subtract => left - right,
                    BinaryOperator::Multiply => left * right,
                    BinaryOperator::Divide => left / right,
                    BinaryOperator::Mod => left % right,
                    BinaryOperator::Xor => left ^ right,
                    BinaryOperator::Greater => Value::Boolean(left > right),
                    BinaryOperator::GreaterEqual => Value::Boolean(left >= right),
                    BinaryOperator::Less => Value::Boolean(left < right),
                    BinaryOperator::LessEqual => Value::Boolean(left <= right),
                    BinaryOperator::Equal => Value::Boolean(left == right),
                    BinaryOperator::NotEqual => Value::Boolean(left != right),
                }
            }
        };
        Ok(value)
    }

    fn boolean(
        &self,
        left: Value,
        right: &SpannedExpression,
        full_evaluate_on: bool,
    ) -> Result<Value, EvalError> {
        let value = match left {
            Value::Boolean(left) => {
                if left == full_evaluate_on {
                    // if `left` is not the result we need, evaluate `right`
                    match self.expression(right)? {
                        Value::Boolean(right) => Value::Boolean(right),
                        _ => Value::Null,
                    }
                } else {
                    Value::Boolean(left) // short circuit
                }
            }
            _ => Value::Null,
        };
        Ok(value)
    }

    fn list(&self, items: &Vec<SpannedExpression>) -> Result<Value, EvalError> {
        Ok(Value::List(
            items
                .iter()
                .map(|item| self.expression(item))
                .collect::<Result<Vec<Value>, EvalError>>()?,
        ))
    }

    fn object(
        &self,
        entries: &Vec<(SpannedExpression, SpannedExpression)>,
    ) -> Result<Value, EvalError> {
        let mut object = Object::new();
        for (key, value) in entries.into_iter() {
            let Expression::Identifier(key) = key.inner() else {
                    panic!();
                };
            let key = key.clone();
            let value = self.expression(&value)?;
            object.insert(key, value);
        }
        Ok(Value::Object(object))
    }

    fn variable(&self, var: &str) -> Result<Value, EvalError> {
        self.env
            .get(&var)
            .map(Clone::clone)
            // .map_or(Value::Null, |v| (*v).clone())
            .ok_or_else(|| EvalError::MissingEnvironment {
                var: var.to_string(),
            })
    }

    fn call(
        &self,
        function: &SpannedExpression,
        args: &[SpannedExpression],
    ) -> Result<Value, EvalError> {
        let Expression::Identifier(var) = function.inner() else {
            return Err(EvalError::CallNonFunction)
        };
        let Value::Function(function) = self.variable(var)? else {
            return Err(EvalError::CallNonFunction);
        };

        let args: Vec<Value> = args
            .iter()
            .map(|expression| self.expression(expression))
            .collect::<Result<Vec<Value>, EvalError>>()?;

        let mut function_env = self.env.child();

        for index in 0..function.args.len() {
            let arg_name = function.args[index].clone();
            let arg_value = args
                .get(index)
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| Value::Null);
            function_env.insert(arg_name, arg_value);
        }

        Evaluator::evaluate(&function.body, &function_env)
    }

    fn get_index(
        &self,
        container: &SpannedExpression,
        index: &SpannedExpression,
    ) -> Result<Value, EvalError> {
        let container = self.expression(container)?;

        Ok(Value::Null)
    }

    fn get_key(
        &self,
        container: &SpannedExpression,
        index: &SpannedExpression,
    ) -> Result<Value, EvalError> {
        Ok(Value::Null)
    }

    fn get_slice(
        &self,
        container: &SpannedExpression,
        start: &SpannedExpression,
        end: &SpannedExpression,
    ) -> Result<Value, EvalError> {
        Ok(Value::Null)
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use map_macro::btree_map;
    use pretty_assertions::assert_eq;
    use rimu_env::Environment;
    use rimu_expr::{BinaryOperator, Expression, SpannedExpression};
    use rimu_report::{SourceId, Span, Spanned};
    use rimu_value::{Function, Number, Value};
    use rust_decimal_macros::dec;

    use super::{EvalError, Evaluator};

    fn span(range: Range<usize>) -> Span {
        Span::new(SourceId::empty(), range.start, range.end)
    }

    fn test(expr: SpannedExpression, env: Environment) -> Result<Value, EvalError> {
        Evaluator::evaluate(&expr, &env)
    }

    #[test]
    fn simple_null() {
        let env = Environment::new();
        let expr = Spanned::new(Expression::Null, span(0..1));
        let actual = test(expr, env);

        let expected = Ok(Value::Null);

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_bool() {
        let env = Environment::new();
        let expr = Spanned::new(Expression::Boolean(false), span(0..1));
        let actual = test(expr, env);

        let expected = Ok(Value::Boolean(false));

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_number() {
        let env = Environment::new();
        let number = dec!(9001).into();
        let expr = Spanned::new(Expression::Number(number), span(0..1));
        let actual = test(expr, env);

        let expected = Ok(Value::Number(number.into()));

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_list() {
        let env = Environment::new();
        let expr = Spanned::new(
            Expression::List(vec![
                Spanned::new(Expression::String("hello".into()), span(1..2)),
                Spanned::new(Expression::Boolean(true), span(3..4)),
                Spanned::new(Expression::String("world".into()), span(5..6)),
            ]),
            span(0..8),
        );
        let actual = test(expr, env);

        let expected = Ok(Value::List(vec![
            Value::String("hello".into()),
            Value::Boolean(true),
            Value::String("world".into()),
        ]));

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_object() {
        let env = Environment::new();
        let expr = Spanned::new(
            Expression::Object(vec![
                (
                    Spanned::new(Expression::Identifier("a".into()), span(1..2)),
                    Spanned::new(Expression::String("hello".into()), span(3..4)),
                ),
                (
                    Spanned::new(Expression::Identifier("b".into()), span(5..6)),
                    Spanned::new(Expression::String("world".into()), span(7..8)),
                ),
            ]),
            span(0..10),
        );
        let actual = test(expr, env);

        let expected = Ok(Value::Object(btree_map! {
            "a".into() => "hello".into(),
            "b".into() => "world".into(),
        }));

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_function_call() {
        let mut env = Environment::new();
        env.insert(
            "add",
            Value::Function(Function {
                name: "add".into(),
                args: vec!["a".into(), "b".into()],
                body: Spanned::new(
                    Expression::Binary {
                        left: Box::new(Spanned::new(
                            Expression::Identifier("a".into()),
                            span(0..1),
                        )),
                        operator: BinaryOperator::Add,
                        right: Box::new(Spanned::new(
                            Expression::Identifier("b".into()),
                            span(2..3),
                        )),
                    },
                    span(0..3),
                ),
            }),
        );
        env.insert("one", Value::Number(dec!(1).into()));
        env.insert("two", Value::Number(dec!(2).into()));

        let expr = Spanned::new(
            Expression::Call {
                function: Box::new(Spanned::new(
                    Expression::Identifier("add".into()),
                    span(0..1),
                )),
                args: vec![
                    Spanned::new(Expression::Identifier("one".into()), span(2..3)),
                    Spanned::new(Expression::Identifier("two".into()), span(4..5)),
                ],
            },
            span(0..7),
        );

        let actual = test(expr, env);

        let expected = Ok(Value::Number(dec!(3).into()));

        assert_eq!(actual, expected);
    }

    /*
    #[test]
    fn negate_number() {
        let one = Decimal::from_u8(1).unwrap();
        let tokens = vec![Token::Minus, Token::Number(one)];
        let actual = test(tokens);

        let expected = Ok(Spanned::new(
            Expression::Unary {
                operator: UnaryOperator::Negative,
                right: Box::new(Spanned::new(Expression::Number(one), span(1..2))),
            },
            span(0..2),
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn add_numbers() {
        let one = Decimal::from_u8(1).unwrap();
        let tokens = vec![Token::Number(one), Token::Plus, Token::Number(one)];
        let actual = test(tokens);

        let expected = Ok(Spanned::new(
            Expression::Binary {
                left: Box::new(Spanned::new(Expression::Number(one), span(0..1))),
                operator: BinaryOperator::Add,
                right: Box::new(Spanned::new(Expression::Number(one), span(2..3))),
            },
            span(0..3),
        ));

        assert_eq!(actual, expected);
    }

    #[test]
    fn precedence_multiply_addition() {
        let one = Decimal::from_u8(1).unwrap();
        let two = Decimal::from_u8(2).unwrap();
        let three = Decimal::from_u8(3).unwrap();

        let actual = test(vec![
            Token::Number(one),
            Token::Plus,
            Token::Number(two),
            Token::Star,
            Token::Number(three),
        ]);
        let expected = Ok(Spanned::new(
            Expression::Binary {
                left: Box::new(Spanned::new(Expression::Number(one), span(0..1))),
                operator: BinaryOperator::Add,
                right: Box::new(Spanned::new(
                    Expression::Binary {
                        left: Box::new(Spanned::new(Expression::Number(two), span(2..3))),
                        operator: BinaryOperator::Multiply,
                        right: Box::new(Spanned::new(Expression::Number(three), span(4..5))),
                    },
                    span(2..5),
                )),
            },
            span(0..5),
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
        let actual = test(tokens);

        let expected = Ok(Spanned::new(
            Expression::GetIndex {
                container: Box::new(Spanned::new(Expression::Identifier("a".into()), span(0..1))),
                index: Box::new(Spanned::new(Expression::Number(one), span(2..3))),
            },
            span(0..4),
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
        let actual = test(tokens);

        let expected = Ok(Spanned::new(
            Expression::GetKey {
                container: Box::new(Spanned::new(Expression::Identifier("a".into()), span(0..1))),
                key: Box::new(Spanned::new(Expression::Identifier("b".into()), span(2..3))),
            },
            span(0..3),
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
        let actual = test(tokens);

        let expected = Ok(Spanned::new(
            Expression::GetSlice {
                container: Box::new(Spanned::new(Expression::Identifier("a".into()), span(0..1))),
                start: Some(Box::new(Spanned::new(Expression::Number(one), span(2..3)))),
                end: Some(Box::new(Spanned::new(Expression::Number(two), span(4..5)))),
            },
            span(0..6),
        ));

        assert_eq!(actual, expected);
    }
    */
}
