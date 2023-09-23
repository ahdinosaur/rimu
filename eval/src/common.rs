use std::{cell::RefCell, rc::Rc};

use rimu_meta::Span;
use rimu_value::{Environment, Function, FunctionBody, NativeFunctionError, Value};

use crate::{evaluate_block, evaluate_expression, EvalError, Result};

pub(crate) fn call(function: Function, function_span: Span, args: &[Value]) -> Result<Value> {
    if let FunctionBody::Native(native) = function.body {
        return match native.call(&args) {
            Ok(value) => Ok(value),
            Err(error) => match error {
                NativeFunctionError::MissingArgument { index } => Err(EvalError::MissingArgument {
                    span: function_span,
                    index,
                }),
                NativeFunctionError::TypeError { got, expected } => Err(EvalError::TypeError {
                    span: function_span,
                    expected,
                    got: *got,
                }),
            },
        };
    }

    let function_env = function.env.clone();
    let mut body_env = Environment::new_with_parent(function_env);

    for index in 0..function.args.len() {
        let arg_name = function.args[index].clone();
        let arg_value = args
            .get(index)
            .map(ToOwned::to_owned)
            // TODO missing arg error or missing context error
            .unwrap_or_else(|| Value::Null);
        body_env.insert(arg_name, arg_value);
    }

    let body_env = Rc::new(RefCell::new(body_env));

    match &function.body {
        FunctionBody::Expression(expression) => evaluate_expression(expression, body_env),
        FunctionBody::Block(block) => evaluate_block(block, body_env),
        _ => unreachable!(),
    }
}
