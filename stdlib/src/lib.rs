use std::{cell::RefCell, rc::Rc};

use rimu_value::{
    Environment, Function, FunctionBody, NativeFunction, NativeFunctionError, Object, Value,
};

pub fn create_stdlib() -> Object {
    let mut lib = Object::new();
    lib.insert("length".into(), length().into());
    lib
}

fn empty_env() -> Rc<RefCell<Environment>> {
    Rc::new(RefCell::new(Environment::new()))
}

pub fn length_function(args: &[Value]) -> Result<Value, NativeFunctionError> {
    let arg = &args[0];
    match arg {
        Value::List(list) => Ok(list.len().into()),
        Value::String(string) => Ok(string.len().into()),
        _ => Err(NativeFunctionError::ArgTypeError {
            index: 0,
            expected: "list | string".into(),
            got: Box::new(arg.clone()),
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

pub fn map_function(args: &[Value]) -> Result<Value, NativeFunctionError> {
    let arg = &args[0];
    match arg {
        Value::Object(object) => {
            let list = object.get("list");
            let mapper = object.get("item");
            match (list, mapper) {
                // NOTE: uh oh, we need to evaluate the given function.
                // - which means we need to be able to return an EvalError
                // - which means a NativeFunction needs to be able to return an EvalError
                // - which means `rimu-value` depends on `rimu-eval` which depends on `rimu-value`
                (Some(Value::List(list)), Some(Value::Function(mapper))) => eval,
                _ => Err(NativeFunctionError::ArgTypeError {
                    index: 0,
                    expected: "{ list: list, item: (item) => next }".into(),
                    got: Box::new(arg.clone()),
                }),
            }
        }
        _ => Err(NativeFunctionError::ArgTypeError {
            index: 0,
            expected: "object".into(),
            got: Box::new(arg.clone()),
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
