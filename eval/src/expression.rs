// with help from
// - https://github.com/DennisPrediger/SLAC/blob/main/src/interpreter.rs

use rimu_ast::{BinaryOperator, Expression, SpannedExpression, UnaryOperator};
use rimu_meta::{Span, Spanned};
use rimu_value::{
    convert_value_object_to_serde_value_object, BothTagged, Environment, Function, FunctionBody,
    Number, SpannedValue, Value, ValueMeta, ValueObject,
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
        let (unwrapped, tag_meta) = match right {
            Value::Tagged { tag, inner, meta } => (inner.into_inner(), Some((tag, meta))),
            other => (other, None),
        };
        let value = match operator {
            UnaryOperator::Negate => match unwrapped.clone() {
                Value::Number(number) => Ok(Value::Number(-number)),
                _ => Err(EvalError::TypeError {
                    span: right_span,
                    expected: "number".into(),
                    got: Box::new(unwrapped.into()),
                }),
            },
            UnaryOperator::Not => {
                let boolean: bool = unwrapped.into();
                Ok(Value::Boolean(!boolean))
            }
        }?;
        Ok(rewrap(span, value, tag_meta))
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
            BinaryOperator::And => self.boolean(span, left, left_span, right, true),
            BinaryOperator::Or => self.boolean(span, left, left_span, right, false),
            // `PartialEq` on `Value` compares tag + meta structurally.
            BinaryOperator::Equal => {
                let right = self.expression(right)?.into_inner();
                Ok(Spanned::new(Value::Boolean(left == right), span))
            }
            BinaryOperator::NotEqual => {
                let right = self.expression(right)?.into_inner();
                Ok(Spanned::new(Value::Boolean(left != right), span))
            }
            BinaryOperator::Greater
            | BinaryOperator::GreaterEqual
            | BinaryOperator::Less
            | BinaryOperator::LessEqual => {
                if let Value::Tagged { tag, .. } = &left {
                    return Err(EvalError::TaggedOrderingComparison {
                        span: left_span,
                        tag: tag.clone(),
                    });
                }
                let (right, right_span) = self.expression(right)?.take();
                if let Value::Tagged { tag, .. } = &right {
                    return Err(EvalError::TaggedOrderingComparison {
                        span: right_span,
                        tag: tag.clone(),
                    });
                }
                let value = match operator {
                    BinaryOperator::Greater => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Boolean(left > right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "number".into(),
                            got: Box::new(right.into()),
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "number".into(),
                            got: Box::new(left.into()),
                        }),
                    },
                    BinaryOperator::GreaterEqual => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Boolean(left >= right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "number".into(),
                            got: Box::new(right.into()),
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "number".into(),
                            got: Box::new(left.into()),
                        }),
                    },
                    BinaryOperator::Less => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Boolean(left < right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "number".into(),
                            got: Box::new(right.into()),
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "number".into(),
                            got: Box::new(left.into()),
                        }),
                    },
                    BinaryOperator::LessEqual => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Boolean(left <= right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "number".into(),
                            got: Box::new(right.into()),
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "number".into(),
                            got: Box::new(left.into()),
                        }),
                    },
                    _ => unreachable!(),
                }?;
                Ok(Spanned::new(value, span))
            }
            // Arithmetic / concat: a tagged operand's tag + meta carries through to the result.
            BinaryOperator::Add
            | BinaryOperator::Subtract
            | BinaryOperator::Multiply
            | BinaryOperator::Divide
            | BinaryOperator::Rem
            | BinaryOperator::Xor => {
                let (right, right_span) = self.expression(right)?.take();
                let (left, right, tag_meta) =
                    combine_tags(left, left_span.clone(), right, right_span.clone())?;
                let value = match operator {
                    BinaryOperator::Add => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left + right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "number".into(),
                            got: Box::new(right.into()),
                        }),
                        (Value::String(left), Value::String(right)) => {
                            Ok(Value::String([left, right].join("")))
                        }
                        (Value::String(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "string".into(),
                            got: Box::new(right.into()),
                        }),
                        (Value::List(left), Value::List(right)) => {
                            Ok(Value::List([left, right].concat()))
                        }
                        (Value::List(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "list".into(),
                            got: Box::new(right.into()),
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "number | string | list".into(),
                            got: Box::new(left.into()),
                        }),
                    },
                    BinaryOperator::Subtract => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left - right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "number".into(),
                            got: Box::new(right.into()),
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "number".into(),
                            got: Box::new(left.into()),
                        }),
                    },
                    BinaryOperator::Multiply => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left * right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "number".into(),
                            got: Box::new(right.into()),
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "number".into(),
                            got: Box::new(left.into()),
                        }),
                    },
                    BinaryOperator::Divide => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left / right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "number".into(),
                            got: Box::new(right.into()),
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "number".into(),
                            got: Box::new(left.into()),
                        }),
                    },
                    BinaryOperator::Rem => match (left.clone(), right.clone()) {
                        (Value::Number(left), Value::Number(right)) => {
                            Ok(Value::Number(left % right))
                        }
                        (Value::Number(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "number".into(),
                            got: Box::new(right.into()),
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "number".into(),
                            got: Box::new(left.into()),
                        }),
                    },
                    BinaryOperator::Xor => match (left.clone(), right.clone()) {
                        (Value::Boolean(left), Value::Boolean(right)) => {
                            Ok(Value::Boolean(left ^ right))
                        }
                        (Value::Boolean(_left), right) => Err(EvalError::TypeError {
                            span: right_span,
                            expected: "boolean".into(),
                            got: Box::new(right.into()),
                        }),
                        _ => Err(EvalError::TypeError {
                            span: left_span,
                            expected: "boolean".into(),
                            got: Box::new(left.into()),
                        }),
                    },
                    _ => unreachable!(),
                }?;
                Ok(rewrap(span, value, tag_meta))
            }
        }
    }

    /// Short-circuiting `&&` / `||`. Tags propagate: an untagged side contributes
    /// nothing; same-tag sides merge metas (right-wins on collision); different
    /// tags error via [`BothTagged`]. Because right is only evaluated when left
    /// doesn't determine the result, `tagged("a", false) && tagged("b", ...)`
    /// short-circuits to `tagged("a", false)` without observing the mismatch.
    fn boolean(
        &self,
        span: Span,
        left: Value,
        left_span: Span,
        right: &SpannedExpression,
        full_evaluate_on: bool,
    ) -> Result<SpannedValue> {
        let (left_inner, left_tag_meta) = match left {
            Value::Tagged { tag, inner, meta } => (inner.into_inner(), Some((tag, meta))),
            other => (other, None),
        };
        let left_bool: bool = left_inner.into();
        if left_bool != full_evaluate_on {
            // short circuit — tag on left (if any) still carries through
            return Ok(rewrap(span, Value::Boolean(left_bool), left_tag_meta));
        }
        let (right, right_span) = self.expression(right)?.take();
        let (right_inner, right_tag_meta) = match right {
            Value::Tagged { tag, inner, meta } => (inner.into_inner(), Some((tag, meta))),
            other => (other, None),
        };
        let right_bool: bool = right_inner.into();
        let tag_meta = merge_tag_metas(left_tag_meta, left_span, right_tag_meta, right_span)?;
        Ok(rewrap(span, Value::Boolean(right_bool), tag_meta))
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
        if let Value::Tagged { tag, .. } = &container {
            return Err(EvalError::TaggedStructuralOp {
                span: container_span,
                tag: tag.clone(),
                op: "index",
            });
        }
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
            (Value::Object(object), Value::String(key)) => {
                object
                    .get(&key)
                    .cloned()
                    .ok_or_else(|| EvalError::KeyNotFound {
                        object_span: container_span,
                        object: Box::new(convert_value_object_to_serde_value_object(object)),
                        key_span: index_span,
                        key: key.clone(),
                    })?
            }
            (Value::Object(_list), _) => {
                return Err(EvalError::TypeError {
                    span: index_span,
                    expected: "string".into(),
                    got: Box::new(index.into()),
                })
            }
            _ => {
                return Err(EvalError::TypeError {
                    span: container_span,
                    expected: "list | string | object".into(),
                    got: Box::new(container.into()),
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

        if let Value::Tagged { tag, .. } = &container {
            return Err(EvalError::TaggedStructuralOp {
                span: container_span,
                tag: tag.clone(),
                op: "key access",
            });
        }

        let Value::Object(object) = container.clone() else {
            return Err(EvalError::TypeError {
                span: container_span,
                expected: "object".into(),
                got: Box::new(container.into()),
            });
        };

        let value = object
            .get(key.inner())
            .ok_or_else(|| EvalError::KeyNotFound {
                object_span: container_span,
                object: Box::new(convert_value_object_to_serde_value_object(object.clone())),
                key: key.clone().into_inner(),
                key_span: key.span(),
            })
            .cloned()?;

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
        if let Value::Tagged { tag, .. } = &container {
            return Err(EvalError::TaggedStructuralOp {
                span: container_span,
                tag: tag.clone(),
                op: "slice",
            });
        }
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
                    got: Box::new(container.into()),
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
            got: Box::new(value.into()),
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
            got: Box::new(value.into()),
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

type TagMeta = (String, ValueMeta);

/// Unwraps a tagged operand so the inner can be computed on. If both sides are
/// tagged with the same tag, their metas are merged (right-wins on key
/// collision) and the tag is carried through. Tag mismatch errors — see
/// [`Value::Tagged`].
fn combine_tags(
    left: Value,
    left_span: Span,
    right: Value,
    right_span: Span,
) -> Result<(Value, Value, Option<TagMeta>)> {
    let (left_inner, left_tag_meta) = match left {
        Value::Tagged { tag, inner, meta } => (inner.into_inner(), Some((tag, meta))),
        other => (other, None),
    };
    let (right_inner, right_tag_meta) = match right {
        Value::Tagged { tag, inner, meta } => (inner.into_inner(), Some((tag, meta))),
        other => (other, None),
    };
    let tag_meta = merge_tag_metas(left_tag_meta, left_span, right_tag_meta, right_span)?;
    Ok((left_inner, right_inner, tag_meta))
}

/// Merge two already-extracted tag+meta pairs. Same-tag merges metas
/// (right-wins on collision); different tags error; `None` contributes nothing.
/// Used by both [`combine_tags`] (which extracts from `Value::Tagged` operands)
/// and by [`Evaluator::boolean`] (which extracts eagerly for short-circuit).
fn merge_tag_metas(
    left: Option<TagMeta>,
    left_span: Span,
    right: Option<TagMeta>,
    right_span: Span,
) -> Result<Option<TagMeta>> {
    match (left, right) {
        (None, None) => Ok(None),
        (Some(lt), None) => Ok(Some(lt)),
        (None, Some(rt)) => Ok(Some(rt)),
        (Some((left_tag, mut left_meta)), Some((right_tag, right_meta))) => {
            if left_tag != right_tag {
                return Err(EvalError::BothTagged(Box::new(BothTagged {
                    left_span,
                    right_span,
                    left_tag,
                    right_tag,
                })));
            }
            for (key, value) in right_meta {
                left_meta.insert(key, value);
            }
            Ok(Some((left_tag, left_meta)))
        }
    }
}

/// Note(cc): both the outer `Spanned` and the inner `Spanned<Value>` are stamped
/// with the whole-operation `span`. A tighter source span for the unwrapped
/// inner value isn't available at this point (the op just produced a fresh
/// `Value`), and downstream consumers only surface the outer span in
/// diagnostics, so the duplication is harmless today.
fn rewrap(span: Span, value: Value, tag_meta: Option<TagMeta>) -> SpannedValue {
    match tag_meta {
        None => Spanned::new(value, span),
        Some((tag, meta)) => {
            let inner = Spanned::new(value, span.clone());
            Spanned::new(
                Value::Tagged {
                    tag,
                    inner: Box::new(inner),
                    meta,
                },
                span,
            )
        }
    }
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

    mod tagged {
        use pretty_assertions::assert_eq;
        use rimu_value::{SerdeValue, SerdeValueMeta, SerdeValueObject};
        use rust_decimal_macros::dec;

        use super::super::EvalError;
        use super::test_code;

        fn tagged(tag: &str, inner: SerdeValue, meta_pairs: &[(&str, SerdeValue)]) -> SerdeValue {
            let mut meta = SerdeValueMeta::new();
            for (key, value) in meta_pairs {
                meta.insert((*key).into(), value.clone());
            }
            SerdeValue::Tagged {
                tag: tag.into(),
                inner: Box::new(inner),
                meta,
            }
        }

        fn host_path_env(abs: &str, origin_dir: &str) -> SerdeValueObject {
            let p = tagged(
                "host_path",
                SerdeValue::String(abs.into()),
                &[("origin_dir", SerdeValue::String(origin_dir.into()))],
            );
            let mut env = SerdeValueObject::new();
            env.insert("p".into(), p);
            env
        }

        #[test]
        fn add_preserves_tag() {
            let env = host_path_env("/abs/foo.txt", "/src");
            let actual = test_code(r#"p + "/hello""#, Some(env)).unwrap();

            let expected = tagged(
                "host_path",
                SerdeValue::String("/abs/foo.txt/hello".into()),
                &[("origin_dir", SerdeValue::String("/src".into()))],
            );
            assert_eq!(actual, expected);
        }

        #[test]
        fn mismatched_tags_error() {
            let mut env = host_path_env("/abs/a", "/src");
            let q = tagged(
                "target_path",
                SerdeValue::String("/abs/b".into()),
                &[("origin_dir", SerdeValue::String("/src".into()))],
            );
            env.insert("q".into(), q);

            let actual = test_code("p + q", Some(env));
            assert!(matches!(actual, Err(EvalError::BothTagged(_))));
        }

        #[test]
        fn same_tag_merges_meta() {
            let a = tagged(
                "secret",
                SerdeValue::String("user:".into()),
                &[("source", SerdeValue::String("env".into()))],
            );
            let b = tagged(
                "secret",
                SerdeValue::String("hunter2".into()),
                &[("name", SerdeValue::String("password".into()))],
            );
            let mut env = SerdeValueObject::new();
            env.insert("a".into(), a);
            env.insert("b".into(), b);

            let actual = test_code("a + b", Some(env)).unwrap();

            let expected = tagged(
                "secret",
                SerdeValue::String("user:hunter2".into()),
                &[
                    ("source", SerdeValue::String("env".into())),
                    ("name", SerdeValue::String("password".into())),
                ],
            );
            assert_eq!(actual, expected);
        }

        #[test]
        fn same_tag_meta_right_wins_on_collision() {
            let a = tagged(
                "secret",
                SerdeValue::String("a".into()),
                &[("source", SerdeValue::String("left".into()))],
            );
            let b = tagged(
                "secret",
                SerdeValue::String("b".into()),
                &[("source", SerdeValue::String("right".into()))],
            );
            let mut env = SerdeValueObject::new();
            env.insert("a".into(), a);
            env.insert("b".into(), b);

            let actual = test_code("a + b", Some(env)).unwrap();

            let expected = tagged(
                "secret",
                SerdeValue::String("ab".into()),
                &[("source", SerdeValue::String("right".into()))],
            );
            assert_eq!(actual, expected);
        }

        #[test]
        fn ordering_on_tagged_errors() {
            let env = host_path_env("/abs/a", "/src");
            let actual = test_code(r#"p < "z""#, Some(env));
            assert!(matches!(
                actual,
                Err(EvalError::TaggedOrderingComparison { .. })
            ));
        }

        #[test]
        fn ordering_on_tagged_right_errors() {
            let env = host_path_env("/abs/a", "/src");
            let actual = test_code(r#""z" < p"#, Some(env));
            assert!(matches!(
                actual,
                Err(EvalError::TaggedOrderingComparison { .. })
            ));
        }

        #[test]
        fn unary_negate_preserves_tag() {
            let n = tagged("host_path", SerdeValue::Number(dec!(5).into()), &[]);
            let mut env = SerdeValueObject::new();
            env.insert("n".into(), n);
            let actual = test_code("-n", Some(env)).unwrap();

            let expected = tagged("host_path", SerdeValue::Number(dec!(-5).into()), &[]);
            assert_eq!(actual, expected);
        }

        #[test]
        fn unary_not_preserves_tag() {
            let env = host_path_env("/abs/a", "/src");
            let actual = test_code("!p", Some(env)).unwrap();

            let expected = tagged(
                "host_path",
                SerdeValue::Boolean(false),
                &[("origin_dir", SerdeValue::String("/src".into()))],
            );
            assert_eq!(actual, expected);
        }

        #[test]
        fn and_short_circuit_on_left_false_keeps_left_tag() {
            // tagged false short-circuits: right is never evaluated, left tag
            // survives. A right-hand tag mismatch can't surface here.
            let left = tagged(
                "host_path",
                SerdeValue::Boolean(false),
                &[("origin_dir", SerdeValue::String("/src".into()))],
            );
            let mut env = SerdeValueObject::new();
            env.insert("left".into(), left);
            let actual = test_code("left && true", Some(env)).unwrap();

            let expected = tagged(
                "host_path",
                SerdeValue::Boolean(false),
                &[("origin_dir", SerdeValue::String("/src".into()))],
            );
            assert_eq!(actual, expected);
        }

        #[test]
        fn and_evaluated_right_carries_right_tag() {
            let right = tagged(
                "host_path",
                SerdeValue::Boolean(true),
                &[("origin_dir", SerdeValue::String("/src".into()))],
            );
            let mut env = SerdeValueObject::new();
            env.insert("right".into(), right);
            let actual = test_code("true && right", Some(env)).unwrap();

            let expected = tagged(
                "host_path",
                SerdeValue::Boolean(true),
                &[("origin_dir", SerdeValue::String("/src".into()))],
            );
            assert_eq!(actual, expected);
        }

        #[test]
        fn and_merges_matching_tag_metas() {
            let left = tagged(
                "secret",
                SerdeValue::Boolean(true),
                &[("source", SerdeValue::String("env".into()))],
            );
            let right = tagged(
                "secret",
                SerdeValue::Boolean(true),
                &[("name", SerdeValue::String("password".into()))],
            );
            let mut env = SerdeValueObject::new();
            env.insert("a".into(), left);
            env.insert("b".into(), right);
            let actual = test_code("a && b", Some(env)).unwrap();

            let expected = tagged(
                "secret",
                SerdeValue::Boolean(true),
                &[
                    ("source", SerdeValue::String("env".into())),
                    ("name", SerdeValue::String("password".into())),
                ],
            );
            assert_eq!(actual, expected);
        }

        #[test]
        fn and_mismatched_tags_error_when_right_evaluated() {
            // Left is true, so right IS evaluated; mismatched tags surface.
            let left = tagged("host_path", SerdeValue::Boolean(true), &[]);
            let right = tagged("secret", SerdeValue::Boolean(true), &[]);
            let mut env = SerdeValueObject::new();
            env.insert("a".into(), left);
            env.insert("b".into(), right);
            let actual = test_code("a && b", Some(env));
            assert!(matches!(actual, Err(EvalError::BothTagged(_))));
        }

        #[test]
        fn or_short_circuit_on_left_true_keeps_left_tag() {
            let left = tagged(
                "host_path",
                SerdeValue::Boolean(true),
                &[("origin_dir", SerdeValue::String("/src".into()))],
            );
            let mut env = SerdeValueObject::new();
            env.insert("left".into(), left);
            let actual = test_code("left || false", Some(env)).unwrap();

            let expected = tagged(
                "host_path",
                SerdeValue::Boolean(true),
                &[("origin_dir", SerdeValue::String("/src".into()))],
            );
            assert_eq!(actual, expected);
        }

        #[test]
        fn slice_on_tagged_errors() {
            let env = host_path_env("/abs/a", "/src");
            let actual = test_code("p[0:2]", Some(env));
            assert!(matches!(
                actual,
                Err(EvalError::TaggedStructuralOp { op: "slice", .. })
            ));
        }

        #[test]
        fn equality_respects_meta() {
            let build = |origin: &str| -> SerdeValue {
                tagged(
                    "host_path",
                    SerdeValue::String("/same/path".into()),
                    &[("origin_dir", SerdeValue::String(origin.into()))],
                )
            };
            let mut env = SerdeValueObject::new();
            env.insert("a".into(), build("/root-a"));
            env.insert("b".into(), build("/root-b"));

            let actual = test_code("a == b", Some(env.clone())).unwrap();
            assert_eq!(actual, SerdeValue::Boolean(false));

            env.insert("c".into(), build("/root-a"));
            let actual = test_code("a == c", Some(env)).unwrap();
            assert_eq!(actual, SerdeValue::Boolean(true));
        }

        #[test]
        fn index_on_tagged_errors() {
            let env = host_path_env("/abs/a", "/src");
            let actual = test_code("p[0]", Some(env));
            assert!(matches!(
                actual,
                Err(EvalError::TaggedStructuralOp { op: "index", .. })
            ));
        }

        #[test]
        fn key_on_tagged_errors() {
            let env = host_path_env("/abs/a", "/src");
            let actual = test_code("p.foo", Some(env));
            assert!(matches!(
                actual,
                Err(EvalError::TaggedStructuralOp {
                    op: "key access",
                    ..
                })
            ));
        }

        mod call {
            use std::{cell::RefCell, rc::Rc};

            use pretty_assertions::assert_eq;
            use rimu_meta::{SourceId, Spanned};
            use rimu_parse::parse_expression;
            use rimu_value::{
                Environment, EvalError, Function, FunctionBody, NativeFunction, SerdeValue,
                SerdeValueObject, SpannedValue, Value,
            };

            use super::super::super::EvalError as TopEvalError;
            use super::super::test_code;
            use super::{host_path_env, tagged};

            fn user_fn(arg_names: &[&str], body_code: &str) -> SerdeValue {
                let (Some(body), errors) = parse_expression(body_code, SourceId::empty()) else {
                    panic!("failed to parse function body");
                };
                assert_eq!(errors.len(), 0);
                SerdeValue::Function(Function {
                    args: arg_names.iter().map(|s| (*s).to_string()).collect(),
                    body: FunctionBody::Expression(body),
                    env: Rc::new(RefCell::new(Environment::new())),
                })
            }

            #[test]
            fn user_fn_propagates_single_tagged_arg() {
                let mut env = host_path_env("/abs/a", "/src");
                env.insert("append_bang".into(), user_fn(&["x"], r#"x + "!""#));

                let actual = test_code("append_bang(p)", Some(env)).unwrap();
                let expected = tagged(
                    "host_path",
                    SerdeValue::String("/abs/a!".into()),
                    &[("origin_dir", SerdeValue::String("/src".into()))],
                );
                assert_eq!(actual, expected);
            }

            #[test]
            fn user_fn_merges_matching_tag_metas() {
                let a = tagged(
                    "secret",
                    SerdeValue::String("user:".into()),
                    &[("source", SerdeValue::String("env".into()))],
                );
                let b = tagged(
                    "secret",
                    SerdeValue::String("hunter2".into()),
                    &[("name", SerdeValue::String("password".into()))],
                );
                let mut env = SerdeValueObject::new();
                env.insert("a".into(), a);
                env.insert("b".into(), b);
                env.insert("concat".into(), user_fn(&["x", "y"], "x + y"));

                let actual = test_code("concat(a, b)", Some(env)).unwrap();

                let expected = tagged(
                    "secret",
                    SerdeValue::String("user:hunter2".into()),
                    &[
                        ("source", SerdeValue::String("env".into())),
                        ("name", SerdeValue::String("password".into())),
                    ],
                );
                assert_eq!(actual, expected);
            }

            #[test]
            fn user_fn_errors_on_mismatched_tags() {
                let a = tagged("host_path", SerdeValue::String("/a".into()), &[]);
                let b = tagged("target_path", SerdeValue::String("/b".into()), &[]);
                let mut env = SerdeValueObject::new();
                env.insert("a".into(), a);
                env.insert("b".into(), b);
                env.insert("concat".into(), user_fn(&["x", "y"], "x + y"));

                let actual = test_code("concat(a, b)", Some(env));
                assert!(matches!(actual, Err(TopEvalError::BothTagged(_))));
            }

            fn first_arg(
                span: rimu_meta::Span,
                args: &[SpannedValue],
            ) -> Result<SpannedValue, EvalError> {
                let arg = args.first().cloned().ok_or(EvalError::MissingArgument {
                    span: span.clone(),
                    index: 0,
                })?;
                Ok(Spanned::new(arg.into_inner(), span))
            }

            #[test]
            fn native_fn_propagates_tag_and_unwraps_input() {
                let native = SerdeValue::Function(Function {
                    args: vec!["x".into()],
                    body: FunctionBody::Native(NativeFunction::new("first_arg", first_arg)),
                    env: Rc::new(RefCell::new(Environment::new())),
                });
                let probe = SerdeValue::Function(Function {
                    args: vec!["x".into()],
                    body: FunctionBody::Native(NativeFunction::new(
                        "assert_untagged_string",
                        |span, args| {
                            let (arg, _) = args[0].clone().take();
                            assert!(
                                matches!(arg, Value::String(_)),
                                "native should receive unwrapped value, got {:?}",
                                arg
                            );
                            Ok(Spanned::new(arg, span))
                        },
                    )),
                    env: Rc::new(RefCell::new(Environment::new())),
                });

                let mut env = host_path_env("/abs/a", "/src");
                env.insert("id".into(), native);
                env.insert("probe".into(), probe);

                let actual = test_code("id(p)", Some(env.clone())).unwrap();
                let expected = tagged(
                    "host_path",
                    SerdeValue::String("/abs/a".into()),
                    &[("origin_dir", SerdeValue::String("/src".into()))],
                );
                assert_eq!(actual, expected);

                // Independently confirm the native callee sees the unwrapped
                // inner (not a Tagged), so it can't accidentally branch on tags.
                let probed = test_code("probe(p)", Some(env)).unwrap();
                assert_eq!(probed, expected);
            }
        }
    }
}
