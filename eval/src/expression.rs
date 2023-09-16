// with help from
// - https://github.com/DennisPrediger/SLAC/blob/main/src/interpreter.rs

use rimu_ast::{BinaryOperator, Expression, SpannedExpression, UnaryOperator};
use rimu_meta::{Span, Spanned};
use rimu_value::{Number, Object, Value};
use rust_decimal::prelude::ToPrimitive;
use std::ops::Deref;

use crate::{Environment, EvalError};

pub fn evaluate<'a>(
    expression: &SpannedExpression,
    env: &'a Environment<'a>,
) -> Result<Value, EvalError> {
    let (value, _span) = Evaluator::new(env).expression(expression)?;
    Ok(value)
}

/// A tree walking interpreter which given an [`Environment`] and an [`Expression`]
/// recursivly walks the tree and computes a single [`Value`].
struct Evaluator<'a> {
    env: &'a Environment<'a>,
}

impl<'a> Evaluator<'a> {
    fn new(env: &'a Environment) -> Self {
        Self { env }
    }

    fn expression(&self, expr: &SpannedExpression) -> Result<(Value, Span), EvalError> {
        let span = expr.span();
        let span_ret = span.clone();

        let value = match expr.inner() {
            Expression::Null => Value::Null,

            Expression::Boolean(boolean) => Value::Boolean(*boolean),

            Expression::String(string) => self.string(span, string)?,

            Expression::Number(decimal) => Value::Number(Into::<Number>::into(*decimal)),

            Expression::List(ref items) => self.list(span, items)?,

            Expression::Object(ref entries) => self.object(span, entries)?,

            Expression::Identifier(var) => self.variable(span, var)?,

            Expression::Unary {
                ref right,
                ref operator,
            } => self.unary(span, right, operator)?,

            Expression::Binary {
                ref left,
                ref right,
                ref operator,
            } => self.binary(span, left, operator, right)?,

            Expression::Call {
                ref function,
                ref args,
            } => self.call(span, function, args)?,

            Expression::GetIndex { container, index } => self.get_index(span, container, index)?,

            Expression::GetKey { container, key } => self.get_key(span, container, key)?,

            Expression::GetSlice {
                container,
                ref start,
                ref end,
            } => self.get_slice(
                span,
                container,
                start.as_ref().map(|s| s.deref()),
                end.as_ref().map(|e| e.deref()),
            )?,

            Expression::Error => Err(EvalError::ErrorExpression { span })?,
        };

        Ok((value, span_ret))
    }

    fn string(&self, _span: Span, string: &str) -> Result<Value, EvalError> {
        // TODO handle string interpolations
        Ok(Value::String(string.to_string()))
    }

    fn unary(
        &self,
        _span: Span,
        right: &SpannedExpression,
        operator: &UnaryOperator,
    ) -> Result<Value, EvalError> {
        let (right, right_span) = self.expression(right)?;
        let value = match operator {
            UnaryOperator::Negate => match right.clone() {
                Value::Number(number) => Ok(Value::Number(-number)),
                _ => Err(EvalError::TypeError {
                    span: right_span,
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

    fn binary(
        &self,
        span: Span,
        left: &SpannedExpression,
        operator: &BinaryOperator,
        right: &SpannedExpression,
    ) -> Result<Value, EvalError> {
        let (left, left_span) = self.expression(left)?;
        let value = match operator {
            BinaryOperator::And => self.boolean(span, left, right, true)?,
            BinaryOperator::Or => self.boolean(span, left, right, false)?,
            _ => {
                let (right, right_span) = self.expression(&*right)?;
                match operator {
                    BinaryOperator::Or => unreachable!(),
                    BinaryOperator::And => unreachable!(),
                    BinaryOperator::Add => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left + right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "number".into(),
                            got: right,
                        }),
                        (Value::String(left), Value::String(right)) => {
                            Ok(Value::String([left, right].join("")))
                        }
                        (Value::String(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "string".into(),
                            got: right,
                        }),
                        (Value::List(left), Value::List(right)) => {
                            Ok(Value::List([left, right].concat()))
                        }
                        (Value::List(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "list".into(),
                            got: right,
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "number | string | list".into(),
                            got: left,
                        }),
                    },
                    BinaryOperator::Subtract => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left - right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "number".into(),
                            got: right,
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "number".into(),
                            got: left,
                        }),
                    },
                    BinaryOperator::Multiply => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left * right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "number".into(),
                            got: right,
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "number".into(),
                            got: left,
                        }),
                    },
                    BinaryOperator::Divide => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left / right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "number".into(),
                            got: right,
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "number".into(),
                            got: left,
                        }),
                    },
                    BinaryOperator::Rem => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left % right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "number".into(),
                            got: right,
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "number".into(),
                            got: left,
                        }),
                    },
                    BinaryOperator::Xor => match (left.clone(), right.clone()) {
                        (Value::Boolean(left), Value::Boolean(right)) => {
                            Ok(Value::Boolean(left ^ right))
                        }
                        (Value::Boolean(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "boolean".into(),
                            got: right,
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "boolean".into(),
                            got: left,
                        }),
                    },
                    BinaryOperator::Greater => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Boolean(left > right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "number".into(),
                            got: right,
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "number".into(),
                            got: left,
                        }),
                    },
                    BinaryOperator::GreaterEqual => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Boolean(left >= right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "number".into(),
                            got: right,
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "number".into(),
                            got: left,
                        }),
                    },
                    BinaryOperator::Less => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Boolean(left < right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "number".into(),
                            got: right,
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "number".into(),
                            got: left,
                        }),
                    },
                    BinaryOperator::LessEqual => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Boolean(left <= right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "number".into(),
                            got: right,
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "number".into(),
                            got: left,
                        }),
                    },
                    BinaryOperator::Equal => Ok(Value::Boolean(left == right)),
                    BinaryOperator::NotEqual => Ok(Value::Boolean(left != right)),
                }?
            }
        };
        Ok(value)
    }

    fn boolean(
        &self,
        _span: Span,
        left: Value,
        right: &SpannedExpression,
        full_evaluate_on: bool,
    ) -> Result<Value, EvalError> {
        let left: bool = left.into();
        let value = if left == full_evaluate_on {
            // if `left` is not the result we need, evaluate `right`
            let (right, _right_span) = self.expression(right)?;
            let right: bool = right.into();
            Value::Boolean(right)
        } else {
            Value::Boolean(left) // short circuit
        };
        Ok(value)
    }

    fn list(&self, _span: Span, items: &Vec<SpannedExpression>) -> Result<Value, EvalError> {
        let mut next_items = Vec::with_capacity(items.len());
        for item in items {
            let (next_item, _next_item_span) = self.expression(item)?;
            next_items.push(next_item);
        }
        Ok(Value::List(next_items))
    }

    fn object(
        &self,
        _span: Span,
        entries: &Vec<(Spanned<String>, SpannedExpression)>,
    ) -> Result<Value, EvalError> {
        let mut object = Object::new();
        for (key, value) in entries.into_iter() {
            let key = key.clone().into_inner();
            let (value, _value_span) = self.expression(&value)?;
            object.insert(key, value);
        }
        Ok(Value::Object(object))
    }

    fn variable(&self, span: Span, var: &str) -> Result<Value, EvalError> {
        self.env
            .get(&var)
            .map(Clone::clone)
            .ok_or_else(|| EvalError::MissingVariable {
                span,
                var: var.to_string(),
            })
    }

    fn call(
        &self,
        span: Span,
        function: &SpannedExpression,
        args: &[SpannedExpression],
    ) -> Result<Value, EvalError> {
        let (Value::Function(function), _function_span) = self.expression(function)? else {
            return Err(EvalError::CallNonFunction {
                span,
                expr: function.clone().into_inner(),
            });
        };

        let args: Vec<Value> = args
            .iter()
            .map(|expression| self.expression(expression))
            .map(|result| result.map(|(value, _span)| value))
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

        evaluate(&function.body, &function_env)
    }

    fn get_index(
        &self,
        _span: Span,
        container: &SpannedExpression,
        index: &SpannedExpression,
    ) -> Result<Value, EvalError> {
        let (container, container_span) = self.expression(container)?;
        let (index, index_span) = self.expression(index)?;

        match (container.clone(), index.clone()) {
            (Value::List(list), index_value) => {
                let index = get_index(container_span, index_span, index_value, list.len(), false)?;
                Ok(list[index as usize].clone())
            }
            (Value::String(string), index_value) => {
                let index =
                    get_index(container_span, index_span, index_value, string.len(), false)?;
                let ch = string[index as usize..].chars().next().unwrap();
                Ok(Value::String(ch.into()))
            }
            (Value::Object(object), Value::String(key)) => object
                .get(&key)
                .map(Clone::clone)
                .ok_or_else(|| EvalError::KeyNotFound {
                    object_span: container_span,
                    object: object.clone(),
                    key_span: index_span,
                    key: key.clone(),
                }),
            (Value::Object(_list), _) => {
                return Err(EvalError::TypeError {
                    span: index_span,
                    expected: "string".into(),
                    got: index,
                });
            }
            _ => Err(EvalError::TypeError {
                span: container_span,
                expected: "list | string | object".into(),
                got: container,
            }),
        }
    }

    fn get_key(
        &self,
        _span: Span,
        container: &SpannedExpression,
        key: &Spanned<String>,
    ) -> Result<Value, EvalError> {
        let (container, container_span) = self.expression(container)?;

        let Value::Object(object) = container.clone() else {
            return Err(EvalError::TypeError {
                span: container_span,
                expected: "object".into(),
                got: container,
            });
        };

        object
            .get(key.inner())
            .ok_or_else(|| EvalError::KeyNotFound {
                object_span: container_span,
                object: object.clone(),
                key: key.clone().into_inner(),
                key_span: key.span(),
            })
            .map(Clone::clone)
    }

    fn get_slice(
        &self,
        span: Span,
        container: &SpannedExpression,
        start: Option<&SpannedExpression>,
        end: Option<&SpannedExpression>,
    ) -> Result<Value, EvalError> {
        let (container, container_span) = self.expression(container)?;
        let start = match start {
            Some(start) => Some(self.expression(start)?),
            None => None,
        };
        let end = match end {
            Some(end) => Some(self.expression(end)?),
            None => None,
        };

        match container.clone() {
            Value::List(list) => {
                let length = list.len();
                match (start.clone(), end.clone()) {
                    (None, None) => Ok(Value::List(list)),
                    (Some((start, start_span)), None) => {
                        let start = get_index(container_span, start_span, start, length, false)?;
                        Ok(Value::List(list[start..].to_vec()))
                    }
                    (None, Some((end, end_span))) => {
                        let end = get_index(container_span, end_span, end, length, true)?;
                        Ok(Value::List(list[..end].to_vec()))
                    }
                    (Some((start, start_span)), Some((end, end_span))) => {
                        let start =
                            get_index(container_span.clone(), start_span, start, length, false)?;
                        let end = get_index(container_span.clone(), end_span, end, length, true)?;
                        if start >= end {
                            return Err(EvalError::RangeStartGreaterThanOrEqualToEnd {
                                span,
                                start,
                                end,
                            });
                        }
                        Ok(Value::List(list[start..end].to_vec()))
                    }
                }
            }
            Value::String(string) => {
                let length = string.len();
                match (start.clone(), end.clone()) {
                    (None, None) => Ok(Value::String(string)),
                    (Some((start, start_span)), None) => {
                        let start = get_index(container_span, start_span, start, length, false)?;
                        Ok(Value::String(string[start..].to_string()))
                    }
                    (None, Some((end, end_span))) => {
                        let end = get_index(container_span, end_span, end, length, true)?;
                        Ok(Value::String(string[..end].to_string()))
                    }
                    (Some((start, start_span)), Some((end, end_span))) => {
                        let start =
                            get_index(container_span.clone(), start_span, start, length, false)?;
                        let end = get_index(container_span.clone(), end_span, end, length, true)?;
                        if start >= end {
                            return Err(EvalError::RangeStartGreaterThanOrEqualToEnd {
                                span,
                                start,
                                end,
                            });
                        }
                        Ok(Value::String(string[start..end].to_string()))
                    }
                }
            }
            _ => {
                return Err(EvalError::TypeError {
                    span: container_span,
                    expected: "list".into(),
                    got: container,
                });
            }
        }
    }
}

fn get_index(
    container_span: Span,
    value_span: Span,
    value: Value,
    length: usize,
    is_range_end: bool,
) -> Result<usize, EvalError> {
    let Value::Number(number) = value else {
        return Err(EvalError::TypeError {
            span: value_span,
            expected: "number".into(),
            got: value,
        });
    };
    let number = if number.is_integer() {
        number.to_isize()
    } else {
        None
    };
    let Some(mut index) = number else {
        return Err(EvalError::TypeError {
            span: value_span,
            expected: "integer".into(),
            got: value,
        });
    };
    let is_under = index <= -(length as isize);
    let is_over = if !is_range_end {
        index >= length as isize
    } else {
        index > length as isize
    };
    if is_under || is_over {
        return Err(EvalError::IndexOutOfBounds {
            container_span,
            index_span: value_span,
            index,
            length,
        });
    }
    // handle negative indices
    if index < 0 {
        index += length as isize
    }
    Ok(index as usize)
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use std::ops::Range;

    use crate::Environment;
    use indexmap::indexmap;
    use pretty_assertions::assert_eq;
    use rimu_ast::{BinaryOperator, Expression, SpannedExpression};
    use rimu_meta::{SourceId, Span, Spanned};
    use rimu_parse::parse_expression;
    use rimu_value::{Function, Value};
    use rust_decimal_macros::dec;

    use super::{evaluate, EvalError};

    fn span(range: Range<usize>) -> Span {
        Span::new(SourceId::empty(), range.start, range.end)
    }

    fn test_expression(
        expr: SpannedExpression,
        env_object: Option<IndexMap<String, Value>>,
    ) -> Result<Value, EvalError> {
        let mut env = Environment::new();
        if let Some(env_object) = env_object {
            for (key, value) in env_object.into_iter() {
                env.insert(key, value);
            }
        }

        evaluate(&expr, &env)
    }

    fn test_code(
        code: &str,
        env_object: Option<IndexMap<String, Value>>,
    ) -> Result<Value, EvalError> {
        let (Some(expr), errors) = parse_expression(code, SourceId::empty()) else {
            panic!()
        };
        assert_eq!(errors.len(), 0);
        test_expression(expr, env_object)
    }

    #[test]
    fn simple_null() {
        let expr = Spanned::new(Expression::Null, span(0..1));
        let actual = test_expression(expr, None);

        let expected = Ok(Value::Null);

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_bool() {
        let expr = Spanned::new(Expression::Boolean(false), span(0..1));
        let actual = test_expression(expr, None);

        let expected = Ok(Value::Boolean(false));

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_number() {
        let number = dec!(9001).into();
        let expr = Spanned::new(Expression::Number(number), span(0..1));
        let actual = test_expression(expr, None);

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
        let actual = test_expression(expr, None);

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
                    Spanned::new("a".into(), span(1..2)),
                    Spanned::new(Expression::String("hello".into()), span(3..4)),
                ),
                (
                    Spanned::new("b".into(), span(5..6)),
                    Spanned::new(Expression::String("world".into()), span(7..8)),
                ),
            ]),
            span(0..10),
        );
        let actual = test_expression(expr, None);

        let expected = Ok(Value::Object(indexmap! {
            "a".into() => "hello".into(),
            "b".into() => "world".into(),
        }));

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_function_call() {
        let env = indexmap! {
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

        let actual = test_expression(expr, Some(env));

        let expected = Ok(Value::Number(dec!(3).into()));

        assert_eq!(actual, expected);
    }

    #[test]
    fn arithmetic() {
        let env = indexmap! {
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
        let env = indexmap! {
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
        let env = indexmap! {
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

    #[test]
    fn get_key() {
        let env = indexmap! {
            "object".into() => Value::Object(indexmap! {
                "a".into() => Value::String("apple".into()),
                "b".into() => Value::String("bear".into()),
                "c".into() => Value::String("cranberry".into()),
                "d".into() => Value::String("dog".into()),
            }),
        };
        let actual = test_code("object.a", Some(env));

        let expected = Ok(Value::String("apple".into()));

        assert_eq!(actual, expected);
    }

    #[test]
    fn get_slice_start_end() {
        let env = indexmap! {
            "list".into() => Value::List(vec![
                Value::String("a".into()),
                Value::String("b".into()),
                Value::String("c".into()),
                Value::String("d".into()),
                Value::String("e".into()),
            ]),
            "start".into() => Value::Number(dec!(1).into()),
            "end".into() => Value::Number(dec!(3).into()),
        };
        let actual = test_code("list[start:end]", Some(env));

        let expected = Ok(Value::List(vec![
            Value::String("b".into()),
            Value::String("c".into()),
        ]));

        assert_eq!(actual, expected);
    }
}
