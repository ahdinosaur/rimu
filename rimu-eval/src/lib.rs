// with help from
// - https://github.com/DennisPrediger/SLAC/blob/main/src/interpreter.rs

use rimu_env::{Environment, EnvironmentError};
use rimu_expr::{BinaryOperator, Expression, SpannedExpression, UnaryOperator};
use rimu_value::{Number, Object, Value};
use rust_decimal::prelude::ToPrimitive;
use std::ops::Deref;

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
    #[error("type error, expected: {expected}, got: {got}")]
    TypeError { expected: String, got: Value },
    #[error("index out of bounds, index: {index}, length: {length}")]
    IndexOutOfBounds { index: isize, length: usize },
    #[error("key error, key: {key}, object: {object:?}")]
    KeyNotFound { key: String, object: Object },
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

            Expression::Unary {
                ref right,
                ref operator,
            } => self.unary(right, operator)?,

            Expression::Binary {
                ref left,
                ref right,
                ref operator,
            } => self.binary(left, operator, right)?,

            Expression::Call {
                ref function,
                ref args,
            } => self.call(function, args)?,

            Expression::GetIndex { container, index } => self.get_index(container, index)?,

            Expression::GetKey { container, key } => self.get_key(container, key)?,

            Expression::GetSlice {
                container,
                ref start,
                ref end,
            } => self.get_slice(
                container,
                &start.as_ref().map(|s| s.deref().clone()),
                &end.as_ref().map(|e| e.deref().clone()),
            )?,

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
            UnaryOperator::Negative => match right.clone() {
                Value::Number(number) => Ok(Value::Number(-number)),
                _ => Err(EvalError::TypeError {
                    expected: "number".into(),
                    got: right.clone(),
                }),
            },
            UnaryOperator::Not => {
                let boolean: bool = right.into();
                Ok(Value::Boolean(!boolean))
            }
        }?;
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
                    BinaryOperator::Add => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left + right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            expected: "number".into(),
                            got: right,
                        }),
                        (Value::String(left), Value::String(right)) => {
                            Ok(Value::String([left, right].join("")))
                        }
                        (Value::String(_left), right) => Err(EvalError::TypeError {
                            expected: "string".into(),
                            got: right,
                        }),
                        (Value::List(left), Value::List(right)) => {
                            Ok(Value::List([left, right].concat()))
                        }
                        (Value::List(_left), right) => Err(EvalError::TypeError {
                            expected: "list".into(),
                            got: right,
                        }),
                        _ => Err(EvalError::TypeError {
                            expected: "number | string | list".into(),
                            got: left,
                        }),
                    },
                    BinaryOperator::Subtract => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left - right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            expected: "number".into(),
                            got: right,
                        }),
                        _ => Err(EvalError::TypeError {
                            expected: "number".into(),
                            got: left,
                        }),
                    },
                    BinaryOperator::Multiply => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left * right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            expected: "number".into(),
                            got: right,
                        }),
                        _ => Err(EvalError::TypeError {
                            expected: "number".into(),
                            got: left,
                        }),
                    },
                    BinaryOperator::Divide => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left / right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            expected: "number".into(),
                            got: right,
                        }),
                        _ => Err(EvalError::TypeError {
                            expected: "number".into(),
                            got: left,
                        }),
                    },
                    BinaryOperator::Mod => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left % right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            expected: "number".into(),
                            got: right,
                        }),
                        _ => Err(EvalError::TypeError {
                            expected: "number".into(),
                            got: left,
                        }),
                    },
                    BinaryOperator::Xor => match (left.clone(), right.clone()) {
                        (Value::Boolean(left), Value::Boolean(right)) => {
                            Ok(Value::Boolean(left ^ right))
                        }
                        (Value::Boolean(_left), right) => Err(EvalError::TypeError {
                            expected: "boolean".into(),
                            got: right,
                        }),
                        _ => Err(EvalError::TypeError {
                            expected: "boolean".into(),
                            got: left,
                        }),
                    },
                    BinaryOperator::Greater => Ok(Value::Boolean(left > right)),
                    BinaryOperator::GreaterEqual => Ok(Value::Boolean(left >= right)),
                    BinaryOperator::Less => Ok(Value::Boolean(left < right)),
                    BinaryOperator::LessEqual => Ok(Value::Boolean(left <= right)),
                    BinaryOperator::Equal => Ok(Value::Boolean(left == right)),
                    BinaryOperator::NotEqual => Ok(Value::Boolean(left != right)),
                }?
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
        let left: bool = left.into();
        let value = if left == full_evaluate_on {
            // if `left` is not the result we need, evaluate `right`
            let right: bool = self.expression(right)?.into();
            Value::Boolean(right)
        } else {
            Value::Boolean(left) // short circuit
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
                // TODO missing arg error or missing context error
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
        let index = self.expression(index)?;

        match (container.clone(), index.clone()) {
            (Value::List(list), Value::Number(number)) => {
                let Some(mut index) = number.to_isize() else {
                    return Err(EvalError::TypeError {
                        expected: "integer".into(),
                        got: index.clone(),
                    });
                };
                let length = list.len();
                if index < 0 {
                    // handle negative indices
                    index = length as isize + index
                }
                if index < 0 {
                    return Err(EvalError::IndexOutOfBounds { index, length });
                }
                let Some(item) = list.get(index as usize) else {
                    return Err(EvalError::IndexOutOfBounds { index, length });
                };
                Ok(item.clone())
            }
            (Value::List(_list), _) => {
                return Err(EvalError::TypeError {
                    expected: "number".into(),
                    got: index.clone(),
                });
            }
            (Value::String(string), Value::Number(number)) => {
                let Some(mut index) = number.to_isize() else {
                    return Err(EvalError::TypeError {
                        expected: "integer".into(),
                        got: index,
                    });
                };
                let length = string.len();
                if index < 0 {
                    // handle negative indices
                    index = length as isize + index
                }
                if index < 0 {
                    return Err(EvalError::IndexOutOfBounds { index, length });
                }
                let Some(substring) = string.get(index as usize..) else {
                    return Err(EvalError::IndexOutOfBounds { index, length });
                };
                let Some(ch) = substring.chars().next() else {
                    return Err(EvalError::IndexOutOfBounds { index, length });
                };
                Ok(Value::String(ch.into()))
            }
            (Value::String(_list), _) => {
                return Err(EvalError::TypeError {
                    expected: "number".into(),
                    got: index,
                });
            }
            (Value::Object(object), Value::String(key)) => object
                .get(&key)
                .map(Clone::clone)
                .ok_or_else(|| EvalError::KeyNotFound {
                    key: key.clone(),
                    object: object.clone(),
                }),
            (Value::Object(_list), _) => {
                return Err(EvalError::TypeError {
                    expected: "string".into(),
                    got: index,
                });
            }
            _ => Err(EvalError::TypeError {
                expected: "list | string | object".into(),
                got: container,
            }),
        }
    }

    fn get_key(
        &self,
        container: &SpannedExpression,
        key: &SpannedExpression,
    ) -> Result<Value, EvalError> {
        Ok(Value::Null)
    }

    fn get_slice(
        &self,
        container: &SpannedExpression,
        start: &Option<SpannedExpression>,
        end: &Option<SpannedExpression>,
    ) -> Result<Value, EvalError> {
        Ok(Value::Null)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, ops::Range};

    use map_macro::btree_map;
    use pretty_assertions::assert_eq;
    use rimu_env::Environment;
    use rimu_expr::{parse, BinaryOperator, Expression, SpannedExpression};
    use rimu_report::{SourceId, Span, Spanned};
    use rimu_value::{Function, Number, Object, Value};
    use rust_decimal_macros::dec;

    use super::{EvalError, Evaluator};

    fn span(range: Range<usize>) -> Span {
        Span::new(SourceId::empty(), range.start, range.end)
    }

    fn test_expr(
        expr: SpannedExpression,
        env_object: Option<BTreeMap<String, Value>>,
    ) -> Result<Value, EvalError> {
        let mut env = Environment::new();
        if let Some(env_object) = env_object {
            for (key, value) in env_object.into_iter() {
                env.insert(key, value);
            }
        }

        Evaluator::evaluate(&expr, &env)
    }

    fn test_code(
        code: &str,
        env_object: Option<BTreeMap<String, Value>>,
    ) -> Result<Value, EvalError> {
        let (Some(expr), errors) = parse(code, SourceId::empty()) else {
            panic!()
        };
        assert_eq!(errors.len(), 0);
        test_expr(expr, env_object)
    }

    #[test]
    fn simple_null() {
        let expr = Spanned::new(Expression::Null, span(0..1));
        let actual = test_expr(expr, None);

        let expected = Ok(Value::Null);

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_bool() {
        let expr = Spanned::new(Expression::Boolean(false), span(0..1));
        let actual = test_expr(expr, None);

        let expected = Ok(Value::Boolean(false));

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_number() {
        let number = dec!(9001).into();
        let expr = Spanned::new(Expression::Number(number), span(0..1));
        let actual = test_expr(expr, None);

        let expected = Ok(Value::Number(number.into()));

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_list() {
        let expr = Spanned::new(
            Expression::List(vec![
                Spanned::new(Expression::String("hello".into()), span(1..2)),
                Spanned::new(Expression::Boolean(true), span(3..4)),
                Spanned::new(Expression::String("world".into()), span(5..6)),
            ]),
            span(0..8),
        );
        let actual = test_expr(expr, None);

        let expected = Ok(Value::List(vec![
            Value::String("hello".into()),
            Value::Boolean(true),
            Value::String("world".into()),
        ]));

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_object() {
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
        let actual = test_expr(expr, None);

        let expected = Ok(Value::Object(btree_map! {
            "a".into() => "hello".into(),
            "b".into() => "world".into(),
        }));

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_function_call() {
        let env = btree_map! {
            "add".into() => Value::Function(Function {
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
            "one".into() => Value::Number(dec!(1).into()),
            "two".into() => Value::Number(dec!(2).into()),
        };

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

        let actual = test_expr(expr, Some(env));

        let expected = Ok(Value::Number(dec!(3).into()));

        assert_eq!(actual, expected);
    }

    #[test]
    fn arithmetic() {
        let env = btree_map! {
            "x".into() => Value::Number(dec!(10).into()),
            "y".into() => Value::Number(dec!(20).into()),
            "z".into() => Value::Number(dec!(40).into()),
            "w".into() => Value::Number(dec!(80).into()),
        };
        let actual = test_code("x + y * (z / w)", Some(env));

        let expected = Ok(Value::Number(dec!(20).into()));

        assert_eq!(actual, expected);
    }

    #[test]
    fn get_list_index() {
        let env = btree_map! {
            "list".into() => Value::List(vec![
                Value::String("a".into()),
                Value::String("b".into()),
                Value::String("c".into()),
                Value::String("d".into()),
            ]),
            "index".into() => Value::Number(dec!(2).into()),
        };
        let actual = test_code("list[index]", Some(env));

        let expected = Ok(Value::String("c".into()));

        assert_eq!(actual, expected);
    }

    #[test]
    fn get_list_index_negative() {
        let env = btree_map! {
            "list".into() => Value::List(vec![
                Value::String("a".into()),
                Value::String("b".into()),
                Value::String("c".into()),
                Value::String("d".into()),
            ]),
            "index".into() => Value::Number(dec!(-2).into()),
        };
        let actual = test_code("list[index]", Some(env));

        let expected = Ok(Value::String("c".into()));

        assert_eq!(actual, expected);
    }

    /*
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
