use std::{cell::RefCell, rc::Rc};

use rimu_value::{
    Environment, Function, FunctionBody, NativeFunction, NativeFunctionError, Object, Value,
};

pub fn create_stdlib() -> Object {
    let mut lib = Object::new();
    lib.insert("length".into(), length().into());
    lib
}

fn env() -> Rc<RefCell<Environment>> {
    Rc::new(RefCell::new(Environment::new()))
}

pub fn length() -> Function {
    let args = vec!["value".into()];
    let function = |args: &[Value]| -> Result<Value, NativeFunctionError> {
        let Some(arg) = args.get(0) else {
            return Err(NativeFunctionError::MissingArgument { index: 0 });
        };

        match arg {
            Value::List(list) => Ok(list.len().into()),
            Value::String(string) => Ok(string.len().into()),
            _ => Err(NativeFunctionError::TypeError {
                expected: "list | string".into(),
                got: Box::new(arg.clone()),
            }),
        }
    };

    Function {
        args,
        env: env(),
        body: FunctionBody::Native(NativeFunction::new(function)),
    }
}
