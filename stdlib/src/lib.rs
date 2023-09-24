use std::{cell::RefCell, rc::Rc};

use rimu_meta::Spanned;
use rimu_value::{Environment, EvalError, Function, FunctionBody, NativeFunction, Object, Value};

pub fn create_stdlib() -> Object {
    let mut lib = Object::new();
    lib.insert("length".into(), length().into());
    lib
}

fn empty_env() -> Rc<RefCell<Environment>> {
    Rc::new(RefCell::new(Environment::new()))
}

pub fn length_function(args: &[Spanned<Value>]) -> Result<Value, EvalError> {
    let (arg, arg_span) = &args[0].clone().take();
    match arg {
        Value::List(list) => Ok(list.len().into()),
        Value::String(string) => Ok(string.len().into()),
        _ => Err(EvalError::TypeError {
            span: arg_span.clone(),
            expected: "list | string".into(),
            got: arg.clone(),
        }),
    }
}

pub fn length() -> Function {
    Function {
        args: vec!["arg".into()],
        env: empty_env(),
        body: FunctionBody::Native(NativeFunction::new(length_function)),
    }
}

pub fn map_function(args: &[Spanned<Value>]) -> Result<Value, EvalError> {
    let (arg, arg_span) = &args[0].clone().take();
    match arg {
        Value::Object(object) => {
            let list = object.get("list");
            let mapper = object.get("item");
            match (list, mapper) {
                (Some(Value::List(list)), Some(Value::Function(mapper))) => {
                    // call(span, function, args).map_err(NativeFunctionError::Eval)
                    Ok(Value::Null)
                }
                _ => Err(EvalError::TypeError {
                    span: arg_span.clone(),
                    expected: "{ list: list, item: (item) => next }".into(),
                    got: arg.clone(),
                }),
            }
        }
        _ => Err(EvalError::TypeError {
            span: arg_span.clone(),
            expected: "object".into(),
            got: arg.clone(),
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
