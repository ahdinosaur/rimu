use std::{cell::RefCell, rc::Rc};

use rimu_meta::Span;
use rimu_value::{Environment, Function, FunctionBody, SpannedValue};

use crate::{evaluate_block, evaluate_expression, EvalError, Result};

pub fn call(span: Span, function: Function, args: &[SpannedValue]) -> Result<SpannedValue> {
    if let FunctionBody::Native(native) = function.body {
        for index in 0..function.args.len() {
            if args.get(index).is_none() {
                return Err(EvalError::MissingArgument {
                    span: span.clone(),
                    index,
                });
            }
        }
        return native.call(span, args);
    }

    let function_env = function.env.clone();
    let mut body_env = Environment::new_with_parent(function_env);

    // Insert each call-site argument into the body env *with its span intact*.
    // The previous code unzipped (Value, Span) and dropped the spans, which
    // (a) lost call-site context for diagnostics inside the body, and
    // (b) flattened typed `Value::HostPath` / `TargetPath` to strings via
    //     `Into<SerdeValue>` — the bug this typed-env refactor exists to fix.
    for index in 0..function.args.len() {
        let arg_name = function.args[index].clone();
        let arg = args
            .get(index)
            .cloned()
            .ok_or_else(|| EvalError::MissingArgument {
                span: span.clone(),
                index,
            })?;
        body_env.insert(arg_name, arg);
    }

    let body_env = Rc::new(RefCell::new(body_env));

    let value = match &function.body {
        FunctionBody::Expression(expression) => evaluate_expression(expression, body_env)?,
        FunctionBody::Block(block) => evaluate_block(block, body_env)?,
        _ => unreachable!(),
    };
    Ok(value)
}
