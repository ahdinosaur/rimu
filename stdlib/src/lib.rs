use std::{cell::RefCell, rc::Rc, slice::from_ref};

use rimu_eval::call;
use rimu_meta::{Span, Spanned};
use rimu_value::{
    Environment, EvalError, Function, FunctionBody, NativeFunction, SerdeValue, SerdeValueObject,
    SpannedValue, Value,
};
use rust_decimal::prelude::ToPrimitive;

pub fn create_stdlib() -> SerdeValueObject {
    let mut lib = SerdeValueObject::new();
    lib.insert("length".into(), length().into());
    lib.insert("map".into(), map().into());
    lib.insert("range".into(), range().into());
    lib
}

fn empty_env() -> Rc<RefCell<Environment>> {
    Rc::new(RefCell::new(Environment::new()))
}

pub fn length() -> Function {
    let function = |span: Span, args: &[Spanned<Value>]| -> Result<SpannedValue, EvalError> {
        let (arg, arg_span) = &args[0].clone().take();
        let value = match arg {
            Value::List(list) => list.len().into(),
            Value::String(string) => string.len().into(),
            _ => {
                return Err(EvalError::TypeError {
                    span: arg_span.clone(),
                    expected: "list | string".into(),
                    got: arg.clone().into(),
                })
            }
        };
        Ok(Spanned::new(value, span))
    };
    Function {
        args: vec!["arg".into()],
        env: empty_env(),
        body: FunctionBody::Native(NativeFunction::new("length", function)),
    }
}
pub fn map() -> Function {
    let function = |span: Span, args: &[Spanned<Value>]| -> Result<SpannedValue, EvalError> {
        let (arg, arg_span) = &args[0].clone().take();
        match arg {
            Value::Object(object) => {
                let list_arg = object.get("list").map(|a| a.inner());
                let mapper_arg = object.get("each").map(|a| a.inner());
                match (list_arg, mapper_arg) {
                    (Some(Value::List(list)), Some(Value::Function(mapper))) => map_op(
                        span,
                        MapOptions {
                            list: list.clone(),
                            mapper: mapper.clone(),
                        },
                    ),
                    _ => Err(EvalError::TypeError {
                        span: arg_span.clone(),
                        expected: "{ list: list, each: (item) => next }".into(),
                        got: arg.clone().into(),
                    }),
                }
            }
            _ => Err(EvalError::TypeError {
                span: arg_span.clone(),
                expected: "object".into(),
                got: arg.clone().into(),
            }),
        }
    };

    Function {
        args: vec!["arg".into()],
        env: empty_env(),
        body: FunctionBody::Native(NativeFunction::new("map", function)),
    }
}

struct MapOptions {
    list: Vec<SpannedValue>,
    mapper: Function,
}

fn map_op(span: Span, options: MapOptions) -> Result<SpannedValue, EvalError> {
    let MapOptions { list, mapper } = options;
    let next_list = list
        .iter()
        .map(|item| call(span.clone(), mapper.clone(), from_ref(item)))
        .collect::<Result<Vec<SpannedValue>, EvalError>>()?;
    Ok(Spanned::new(Value::List(next_list), span))
}

pub fn range() -> Function {
    let function = |span: Span, args: &[Spanned<Value>]| -> Result<SpannedValue, EvalError> {
        let (arg, arg_span) = &args[0].clone().take();
        match arg {
            Value::Object(object) => {
                let start = object.get("start");
                let end = object.get("end");
                let start = start.map(|a| a.clone().take());
                let end = end.map(|a| a.clone().take());
                match (start, end) {
                    (None, None) => Ok(Spanned::new(Value::List(vec![]), span)),
                    (None, Some((Value::Number(end), end_span))) => {
                        let end = end.to_usize().ok_or_else(|| EvalError::TypeError {
                            span: end_span,
                            expected: "zero or positive integer".into(),
                            got: SerdeValue::Number(end),
                        })?;
                        range_op(span, RangeOptions { start: None, end })
                    }
                    (
                        Some((Value::Number(start), start_span)),
                        Some((Value::Number(end), end_span)),
                    ) => {
                        let start = start.to_usize().ok_or_else(|| EvalError::TypeError {
                            span: start_span,
                            expected: "zero or positive integer".into(),
                            got: SerdeValue::Number(start),
                        })?;
                        let end = end.to_usize().ok_or_else(|| EvalError::TypeError {
                            span: end_span,
                            expected: "zero or positive integer".into(),
                            got: SerdeValue::Number(end),
                        })?;
                        range_op(
                            span,
                            RangeOptions {
                                start: Some(start),
                                end,
                            },
                        )
                    }
                    _ => Err(EvalError::TypeError {
                        span: arg_span.clone(),
                        expected: "{ start?: number, end: number }".into(),
                        got: arg.clone().into(),
                    }),
                }
            }
            _ => Err(EvalError::TypeError {
                span: arg_span.clone(),
                expected: "object".into(),
                got: arg.clone().into(),
            }),
        }
    };

    Function {
        args: vec!["arg".into()],
        env: empty_env(),
        body: FunctionBody::Native(NativeFunction::new("range", function)),
    }
}

struct RangeOptions {
    start: Option<usize>,
    end: usize,
}

fn range_op(span: Span, options: RangeOptions) -> Result<SpannedValue, EvalError> {
    let RangeOptions { start, end } = options;
    let start = start.unwrap_or(0);
    let list = (start..end).map(Into::into).collect();
    Ok(SerdeValue::List(list).with_span(span))
}
