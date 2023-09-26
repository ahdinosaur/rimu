use std::{cell::RefCell, ops::Range, rc::Rc};

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

pub fn length_function(span: Span, args: &[Spanned<Value>]) -> Result<SpannedValue, EvalError> {
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
}

pub fn length() -> Function {
    Function {
        args: vec!["arg".into()],
        env: empty_env(),
        body: FunctionBody::Native(NativeFunction::new(length_function)),
    }
}

pub fn map_function(span: Span, args: &[Spanned<Value>]) -> Result<SpannedValue, EvalError> {
    let (arg, arg_span) = &args[0].clone().take();
    match arg {
        Value::Object(object) => {
            let list_arg = object.get("list");
            let mapper_arg = object.get("item");
            let (Some(list_arg), Some(mapper_arg)) = (list_arg, mapper_arg) else {
                return Err(EvalError::TypeError {
                    span: arg_span.clone(),
                    expected: "{ list: list, item: (item) => next }".into(),
                    got: arg.clone().into(),
                });
            };
            match (list_arg.inner(), mapper_arg.inner()) {
                (Value::List(list), Value::Function(mapper)) => {
                    let next_list = list
                        .iter()
                        .map(|item| call(span.clone(), mapper.clone(), &[item.clone()]))
                        .collect::<Result<Vec<SpannedValue>, EvalError>>()?;
                    Ok(Spanned::new(Value::List(next_list), list_arg.span()))
                }
                _ => Err(EvalError::TypeError {
                    span: arg_span.clone(),
                    expected: "{ list: list, item: (item) => next }".into(),
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
}

pub fn map() -> Function {
    Function {
        args: vec!["arg".into()],
        env: empty_env(),
        body: FunctionBody::Native(NativeFunction::new(map_function)),
    }
}

pub fn range_function(span: Span, args: &[Spanned<Value>]) -> Result<SpannedValue, EvalError> {
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
                    let start: usize = 0;
                    let end = end.to_usize().ok_or_else(|| EvalError::TypeError {
                        span: end_span,
                        expected: "zero or positive integer".into(),
                        got: SerdeValue::Number(end),
                    })?;
                    Ok(SerdeValue::List((start..end).map(Into::into).collect()).with_span(span))
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
                    Ok(SerdeValue::List((start..end).map(Into::into).collect()).with_span(span))
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
}

pub fn range() -> Function {
    Function {
        args: vec!["arg".into()],
        env: empty_env(),
        body: FunctionBody::Native(NativeFunction::new(range_function)),
    }
}
