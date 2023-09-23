use std::{cell::RefCell, rc::Rc};

use rimu_meta::{Span, Spanned};
use rimu_value::{Environment, Function, FunctionBody, NativeFunctionError, Value};

use crate::{evaluate_block, evaluate_expression, EvalError, Result};

pub fn call(span: Span, function: Function, args: &[Spanned<Value>]) -> Result<Value> {
    if let FunctionBody::Native(native) = function.body {
        let (args, arg_spans): (Vec<_>, Vec<_>) = args.iter().map(|a| a.take()).unzip();
        return match native.call(&args) {
            Ok(value) => Ok(value),
            Err(error) => match error {
                NativeFunctionError::ArgTypeError {
                    index,
                    got,
                    expected,
                } => Err(EvalError::TypeError {
                    span: arg_spans[index].clone(),
                    expected,
                    got: *got,
                }),
            },
        };
    }

    let function_env = function.env.clone();
    let mut body_env = Environment::new_with_parent(function_env);

    let (args, _arg_spans): (Vec<_>, Vec<_>) = args.iter().map(|a| a.take()).unzip();
    for index in 0..function.args.len() {
        let arg_name = function.args[index].clone();
        let arg_value = args
            .get(index)
            .map(ToOwned::to_owned)
            .map_or_else(|| Err(EvalError::MissingArgument { span, index }), Ok)?;
        body_env.insert(arg_name, arg_value);
    }

    let body_env = Rc::new(RefCell::new(body_env));

    match &function.body {
        FunctionBody::Expression(expression) => evaluate_expression(expression, body_env),
        FunctionBody::Block(block) => evaluate_block(block, body_env),
        _ => unreachable!(),
    }
}
