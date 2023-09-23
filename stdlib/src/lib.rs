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
        _ => Err(NativeFunctionError::TypeError {
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
            // TODO validate object shape
        }
        _ => Err(NativeFunctionError::TypeError {
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
