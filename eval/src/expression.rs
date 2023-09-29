// with help from
// - https://github.com/DennisPrediger/SLAC/blob/main/src/interpreter.rs

use rimu_ast::{BinaryOperator, Expression, SpannedExpression, UnaryOperator};
use rimu_meta::{Span, Spanned};
use rimu_value::{
    convert_value_object_to_serde_value_object, Environment, Function, FunctionBody, Number,
    SpannedValue, Value, ValueObject,
};
use rust_decimal::prelude::ToPrimitive;
use std::{cell::RefCell, ops::Deref, rc::Rc};

use crate::{common, EvalError, Result};

pub fn evaluate(
    expression: &SpannedExpression,
    env: Rc<RefCell<Environment>>,
) -> Result<SpannedValue> {
    Evaluator::new(env).expression(expression)
}

/// A tree walking interpreter which given an [`Environment`] and an [`Expression`]
/// recursivly walks the tree and computes a single [`Value`].
struct Evaluator {
    env: Rc<RefCell<Environment>>,
}

impl Evaluator {
    fn new(env: Rc<RefCell<Environment>>) -> Self {
        Self { env }
    }

    fn expression(&self, expr: &SpannedExpression) -> Result<SpannedValue> {
        let span = expr.span();
        match expr.inner() {
            Expression::Null => Ok(Spanned::new(Value::Null, span)),

            Expression::Boolean(boolean) => Ok(Spanned::new(Value::Boolean(*boolean), span)),

            Expression::String(string) => self.string(span, string),

            Expression::Number(decimal) => Ok(Spanned::new(
                Value::Number(Into::<Number>::into(*decimal)),
                span,
            )),

            Expression::List(ref items) => self.list(span, items),

            Expression::Object(ref entries) => self.object(span, entries),

            Expression::Function { ref args, ref body } => self.function(span, args, body),

            Expression::Identifier(var) => self.variable(span, var),

            Expression::Unary {
                ref right,
                ref operator,
            } => self.unary(span, right, operator),

            Expression::Binary {
                ref left,
                ref right,
                ref operator,
            } => self.binary(span, left, operator, right),

            Expression::Call {
                ref function,
                ref args,
            } => self.call(span, function, args),

            Expression::GetIndex { container, index } => self.get_index(span, container, index),

            Expression::GetKey { container, key } => self.get_key(span, container, key),

            Expression::GetSlice {
                container,
                ref start,
                ref end,
            } => self.get_slice(
                span,
                container,
                start.as_ref().map(|s| s.deref()),
                end.as_ref().map(|e| e.deref()),
            ),

            Expression::Error => Err(EvalError::ErrorExpression { span }),
        }
    }

    fn string(&self, span: Span, string: &str) -> Result<SpannedValue> {
        // TODO handle string interpolations
        let value = Value::String(string.to_string());
        Ok(Spanned::new(value, span))
    }

    fn function(
        &self,
        span: Span,
        args: &[Spanned<String>],
        body: &SpannedExpression,
    ) -> Result<SpannedValue> {
        let args: Vec<String> = args.iter().map(|a| a.inner()).cloned().collect();
        let body = FunctionBody::Expression(body.clone());
        let env = self.env.clone();
        let value = Value::Function(Function { args, body, env });
        Ok(Spanned::new(value, span))
    }

    fn unary(
        &self,
        span: Span,
        right: &SpannedExpression,
        operator: &UnaryOperator,
    ) -> Result<SpannedValue> {
        let (right, right_span) = self.expression(right)?.take();
        let value = match operator {
            UnaryOperator::Negate => match right.clone() {
                Value::Number(number) => Ok(Value::Number(-number)),
                _ => Err(EvalError::TypeError {
                    span: right_span,
                    expected: "number".into(),
                    got: right.clone().into(),
                }),
            },
            UnaryOperator::Not => {
                let boolean: bool = right.into();
                Ok(Value::Boolean(!boolean))
            }
        }?;
        Ok(Spanned::new(value, span))
    }

    fn binary(
        &self,
        span: Span,
        left: &SpannedExpression,
        operator: &BinaryOperator,
        right: &SpannedExpression,
    ) -> Result<SpannedValue> {
        let (left, left_span) = self.expression(left)?.take();
        match operator {
            BinaryOperator::And => self.boolean(span, left, right, true),
            BinaryOperator::Or => self.boolean(span, left, right, false),
            _ => {
                let (right, right_span) = self.expression(right)?.take();
                let value = match operator {
                    BinaryOperator::Or => unreachable!(),
                    BinaryOperator::And => unreachable!(),
                    BinaryOperator::Add => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left + right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "number".into(),
                            got: right.into(),
                        }),
                        (Value::String(left), Value::String(right)) => {
                            Ok(Value::String([left, right].join("")))
                        }
                        (Value::String(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "string".into(),
                            got: right.into(),
                        }),
                        (Value::List(left), Value::List(right)) => {
                            Ok(Value::List([left, right].concat()))
                        }
                        (Value::List(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "list".into(),
                            got: right.into(),
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "number | string | list".into(),
                            got: left.into(),
                        }),
                    },
                    BinaryOperator::Subtract => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left - right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "number".into(),
                            got: right.into(),
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "number".into(),
                            got: left.into(),
                        }),
                    },
                    BinaryOperator::Multiply => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left * right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "number".into(),
                            got: right.into(),
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "number".into(),
                            got: left.into(),
                        }),
                    },
                    BinaryOperator::Divide => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left / right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "number".into(),
                            got: right.into(),
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "number".into(),
                            got: left.into(),
                        }),
                    },
                    BinaryOperator::Rem => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left % right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "number".into(),
                            got: right.into(),
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "number".into(),
                            got: left.into(),
                        }),
                    },
                    BinaryOperator::Xor => match (left.clone(), right.clone()) {
                        (Value::Boolean(left), Value::Boolean(right)) => {
                            Ok(Value::Boolean(left ^ right))
                        }
                        (Value::Boolean(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "boolean".into(),
                            got: right.into(),
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "boolean".into(),
                            got: left.into(),
                        }),
                    },
                    BinaryOperator::Greater => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Boolean(left > right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "number".into(),
                            got: right.into(),
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "number".into(),
                            got: left.into(),
                        }),
                    },
                    BinaryOperator::GreaterEqual => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Boolean(left >= right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "number".into(),
                            got: right.into(),
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "number".into(),
                            got: left.into(),
                        }),
                    },
                    BinaryOperator::Less => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Boolean(left < right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "number".into(),
                            got: right.into(),
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "number".into(),
                            got: left.into(),
                        }),
                    },
                    BinaryOperator::LessEqual => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Boolean(left <= right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "number".into(),
                            got: right.into(),
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "number".into(),
                            got: left.into(),
                        }),
                    },
                    BinaryOperator::Equal => Ok(Value::Boolean(left == right)),
                    BinaryOperator::NotEqual => Ok(Value::Boolean(left != right)),
                }?;
                Ok(Spanned::new(value, span))
            }
        }
    }

    fn boolean(
        &self,
        span: Span,
        left: Value,
        right: &SpannedExpression,
        full_evaluate_on: bool,
    ) -> Result<SpannedValue> {
        let left: bool = left.into();
        let value = if left == full_evaluate_on {
            // if `left` is not the result we need, evaluate `right`
            let right = self.expression(right)?.into_inner();
            let right: bool = right.into();
            Value::Boolean(right)
        } else {
            Value::Boolean(left) // short circuit
        };
        Ok(Spanned::new(value, span))
    }

    fn list(&self, span: Span, items: &Vec<SpannedExpression>) -> Result<SpannedValue> {
        let mut next_items = Vec::with_capacity(items.len());
        for item in items {
            let next_item = self.expression(item)?;
            next_items.push(next_item);
        }
        let value = Value::List(next_items);
        Ok(Spanned::new(value, span))
    }

    fn object(
        &self,
        span: Span,
        entries: &[(Spanned<String>, SpannedExpression)],
    ) -> Result<SpannedValue> {
        let mut object = ValueObject::new();
        for (key, value) in entries.iter() {
            let key = key.clone().into_inner();
            let value = self.expression(value)?;
            object.insert(key, value);
        }
        let value = Value::Object(object);
        Ok(Spanned::new(value, span))
    }

    fn variable(&self, span: Span, var: &str) -> Result<SpannedValue> {
        let value = self
            .env
            .borrow()
            .get(var)
            .ok_or_else(|| EvalError::MissingVariable {
                span: span.clone(),
                var: var.to_string(),
            })?;
        Ok(value.with_span(span))
    }

    fn call(
        &self,
        span: Span,
        function: &SpannedExpression,
        args: &[SpannedExpression],
    ) -> Result<SpannedValue> {
        let Value::Function(function) = self.expression(function)?.into_inner() else {
            return Err(EvalError::CallNonFunction {
                span: function.span(),
                expr: function.clone().into_inner(),
            });
        };

        let args: Vec<SpannedValue> = args
            .iter()
            .map(|expression| self.expression(expression))
            .collect::<Result<Vec<SpannedValue>>>()?;

        common::call(span, function, &args)
    }

    fn get_index(
        &self,
        span: Span,
        container: &SpannedExpression,
        index: &SpannedExpression,
    ) -> Result<SpannedValue> {
        let (container, container_span) = self.expression(container)?.take();
        let (index, index_span) = self.expression(index)?.take();

        let value = match (container.clone(), index.clone()) {
            (Value::List(list), index_value) => {
                let index = get_index(container_span, index_span, index_value, list.len(), false)?;
                list[index as usize].clone()
            }
            (Value::String(string), index_value) => {
                let index =
                    get_index(container_span, index_span, index_value, string.len(), false)?;
                let ch = string[index as usize..].chars().next().unwrap();
                Spanned::new(Value::String(ch.into()), span.clone())
            }
            (Value::Object(object), Value::String(key)) => object
                .get(&key)
                .map(Clone::clone)
                .ok_or_else(|| EvalError::KeyNotFound {
                    object_span: container_span,
                    object: convert_value_object_to_serde_value_object(object),
                    key_span: index_span,
                    key: key.clone(),
                })?,
            (Value::Object(_list), _) => {
                return Err(EvalError::TypeError {
                    span: index_span,
                    expected: "string".into(),
                    got: index.into(),
                })
            }
            _ => {
                return Err(EvalError::TypeError {
                    span: container_span,
                    expected: "list | string | object".into(),
                    got: container.into(),
                })
            }
        };

        Ok(Spanned::new(value.into_inner(), span))
    }

    fn get_key(
        &self,
        span: Span,
        container: &SpannedExpression,
        key: &Spanned<String>,
    ) -> Result<SpannedValue> {
        let (container, container_span) = self.expression(container)?.take();

        let Value::Object(object) = container.clone() else {
            return Err(EvalError::TypeError {
                span: container_span,
                expected: "object".into(),
                got: container.into(),
            });
        };

        let value = object
            .get(key.inner())
            .ok_or_else(|| EvalError::KeyNotFound {
                object_span: container_span,
                object: convert_value_object_to_serde_value_object(object.clone()),
                key: key.clone().into_inner(),
                key_span: key.span(),
            })
            .map(Clone::clone)?;

        Ok(Spanned::new(value.into_inner(), span))
    }

    fn get_slice(
        &self,
        span: Span,
        container: &SpannedExpression,
        start: Option<&SpannedExpression>,
        end: Option<&SpannedExpression>,
    ) -> Result<SpannedValue> {
        let (container, container_span) = self.expression(container)?.take();
        let start = match start {
            Some(start) => Some(self.expression(start)?.take()),
            None => None,
        };
        let end = match end {
            Some(end) => Some(self.expression(end)?.take()),
            None => None,
        };

        let value = match container.clone() {
            Value::List(list) => {
                let length = list.len();
                match (start.clone(), end.clone()) {
                    (None, None) => Value::List(list),
                    (Some((start, start_span)), None) => {
                        let start = get_index(container_span, start_span, start, length, false)?;
                        Value::List(list[start..].to_vec())
                    }
                    (None, Some((end, end_span))) => {
                        let end = get_index(container_span, end_span, end, length, true)?;
                        Value::List(list[..end].to_vec())
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
                        Value::List(list[start..end].to_vec())
                    }
                }
            }
            Value::String(string) => {
                let length = string.len();
                match (start.clone(), end.clone()) {
                    (None, None) => Value::String(string),
                    (Some((start, start_span)), None) => {
                        let start = get_index(container_span, start_span, start, length, false)?;
                        Value::String(string[start..].to_string())
                    }
                    (None, Some((end, end_span))) => {
                        let end = get_index(container_span, end_span, end, length, true)?;
                        Value::String(string[..end].to_string())
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
                        Value::String(string[start..end].to_string())
                    }
                }
            }
            _ => {
                return Err(EvalError::TypeError {
                    span: container_span,
                    expected: "list".into(),
                    got: container.into(),
                })
            }
        };

        Ok(Spanned::new(value, span))
    }
}

fn get_index(
    container_span: Span,
    value_span: Span,
    value: Value,
    length: usize,
    is_range_end: bool,
) -> Result<usize> {
    let Value::Number(number) = value else {
        return Err(EvalError::TypeError {
            span: value_span,
            expected: "number".into(),
            got: value.into(),
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
            got: value.into(),
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
    use std::{cell::RefCell, ops::Range, rc::Rc};

    use indexmap::indexmap;
    use pretty_assertions::assert_eq;
    use rimu_ast::{BinaryOperator, Expression, SpannedExpression};
    use rimu_meta::{SourceId, Span, Spanned};
    use rimu_parse::parse_expression;
    use rimu_value::{Environment, Function, FunctionBody, SerdeValue};
    use rust_decimal_macros::dec;

    use super::{evaluate, EvalError};

    fn span(range: Range<usize>) -> Span {
        Span::new(SourceId::empty(), range.start, range.end)
    }

    fn test_expression(
        expr: SpannedExpression,
        env_object: Option<IndexMap<String, SerdeValue>>,
    ) -> Result<SerdeValue, EvalError> {
        let mut env = Environment::new();
        if let Some(env_object) = env_object {
            for (key, value) in env_object.into_iter() {
                env.insert(key, value);
            }
        }

        let value = evaluate(&expr, Rc::new(RefCell::new(env)))?;
        let value: SerdeValue = value.into();
        Ok(value)
    }

    fn test_code(
        code: &str,
        env_object: Option<IndexMap<String, SerdeValue>>,
    ) -> Result<SerdeValue, EvalError> {
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

        let expected = Ok(SerdeValue::Null);

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_bool() {
        let expr = Spanned::new(Expression::Boolean(false), span(0..1));
        let actual = test_expression(expr, None);

        let expected = Ok(SerdeValue::Boolean(false));

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_number() {
        let number = dec!(9001);
        let expr = Spanned::new(Expression::Number(number), span(0..1));
        let actual = test_expression(expr, None);

        let expected = Ok(SerdeValue::Number(number.into()));

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

        let expected = Ok(SerdeValue::List(vec![
            SerdeValue::String("hello".into()),
            SerdeValue::Boolean(true),
            SerdeValue::String("world".into()),
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

        let expected = Ok(SerdeValue::Object(indexmap! {
            "a".into() => "hello".into(),
            "b".into() => "world".into(),
        }));

        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_function_call() {
        let env = indexmap! {
            "add".into() => SerdeValue::Function(Function {
                args: vec!["a".into(), "b".into()],
                body: FunctionBody::Expression(Spanned::new(
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
                )),
                env: Rc::new(RefCell::new(Environment::new())),
            }),
            "one".into() => SerdeValue::Number(dec!(1).into()),
            "two".into() => SerdeValue::Number(dec!(2).into()),
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

        let expected = Ok(SerdeValue::Number(dec!(3).into()));

        assert_eq!(actual, expected);
    }

    #[test]
    fn arithmetic() {
        let env = indexmap! {
            "x".into() => SerdeValue::Number(dec!(10).into()),
            "y".into() => SerdeValue::Number(dec!(20).into()),
            "z".into() => SerdeValue::Number(dec!(40).into()),
            "w".into() => SerdeValue::Number(dec!(80).into()),
        };
        let actual = test_code("x + y * (z / w)", Some(env));

        let expected = Ok(SerdeValue::Number(dec!(20).into()));

        assert_eq!(actual, expected);
    }

    #[test]
    fn get_list_index() {
        let env = indexmap! {
            "list".into() => SerdeValue::List(vec![
                SerdeValue::String("a".into()),
                SerdeValue::String("b".into()),
                SerdeValue::String("c".into()),
                SerdeValue::String("d".into()),
            ]),
            // "index".into() => Value::Number(dec!(2).into()),
        };
        // let actual = test_code("list[index]", Some(env));
        let actual = test_code("list[2]", Some(env));

        let expected = Ok(SerdeValue::String("c".into()));

        assert_eq!(actual, expected);
    }

    #[test]
    fn get_list_index_negative() {
        let env = indexmap! {
            "list".into() => SerdeValue::List(vec![
                SerdeValue::String("a".into()),
                SerdeValue::String("b".into()),
                SerdeValue::String("c".into()),
                SerdeValue::String("d".into()),
            ]),
            // "index".into() => Value::Number(dec!(-2).into()),
        };
        // let actual = test_code("list[index]", Some(env));
        let actual = test_code("list[-2]", Some(env));

        let expected = Ok(SerdeValue::String("c".into()));

        assert_eq!(actual, expected);
    }

    #[test]
    fn get_key() {
        let env = indexmap! {
            "object".into() => SerdeValue::Object(indexmap! {
                "a".into() => SerdeValue::String("apple".into()),
                "b".into() => SerdeValue::String("bear".into()),
                "c".into() => SerdeValue::String("cranberry".into()),
                "d".into() => SerdeValue::String("dog".into()),
            }),
        };
        let actual = test_code("object.a", Some(env));

        let expected = Ok(SerdeValue::String("apple".into()));

        assert_eq!(actual, expected);
    }

    #[test]
    fn get_slice_start_end() {
        let env = indexmap! {
            "list".into() => SerdeValue::List(vec![
                SerdeValue::String("a".into()),
                SerdeValue::String("b".into()),
                SerdeValue::String("c".into()),
                SerdeValue::String("d".into()),
                SerdeValue::String("e".into()),
            ]),
            "start".into() => SerdeValue::Number(dec!(1).into()),
            "end".into() => SerdeValue::Number(dec!(3).into()),
        };
        let actual = test_code("list[start:end]", Some(env));

        let expected = Ok(SerdeValue::List(vec![
            SerdeValue::String("b".into()),
            SerdeValue::String("c".into()),
        ]));

        assert_eq!(actual, expected);
    }
}
