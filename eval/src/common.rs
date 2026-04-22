use std::{cell::RefCell, rc::Rc};

use rimu_meta::{Span, Spanned};
use rimu_value::{
    merge_tag_metas, peel_tag, rewrap_tag, Environment, Function, FunctionBody, SpannedValue,
    TagMeta,
};

use crate::{evaluate_block, evaluate_expression, EvalError, Result};

pub fn call(span: Span, function: Function, args: &[SpannedValue]) -> Result<SpannedValue> {
    let (args, tag_meta) = peel_call_args(args)?;

    let value = if let FunctionBody::Native(native) = function.body {
        for index in 0..function.args.len() {
            if args.get(index).is_none() {
                return Err(EvalError::MissingArgument {
                    span: span.clone(),
                    index,
                });
            }
        }
        native.call(span.clone(), &args)?
    } else {
        let function_env = function.env.clone();
        let mut body_env = Environment::new_with_parent(function_env);

        for index in 0..function.args.len() {
            let arg_name = function.args[index].clone();
            let arg_value = args
                .get(index)
                .map(|a| a.clone().into_inner())
                .map_or_else(
                    || {
                        Err(EvalError::MissingArgument {
                            span: span.clone(),
                            index,
                        })
                    },
                    Ok,
                )?;
            body_env.insert(arg_name, arg_value);
        }

        let body_env = Rc::new(RefCell::new(body_env));

        match &function.body {
            FunctionBody::Expression(expression) => evaluate_expression(expression, body_env)?,
            FunctionBody::Block(block) => evaluate_block(block, body_env)?,
            _ => unreachable!(),
        }
    };

    Ok(rewrap_tag(span, value, tag_meta))
}

/// Peels tags off each argument (so the callee sees raw values) and folds
/// the tags into a single tag + merged meta to re-wrap onto the result.
fn peel_call_args(args: &[SpannedValue]) -> Result<(Vec<SpannedValue>, Option<TagMeta>)> {
    let mut peeled = Vec::with_capacity(args.len());
    let mut acc: Option<(TagMeta, Span)> = None;
    for arg in args {
        let (value, span) = arg.clone().take();
        let (inner, tag_meta) = peel_tag(value);
        peeled.push(Spanned::new(inner, span.clone()));
        if let Some(current) = tag_meta {
            acc = Some(match acc {
                None => (current, span),
                Some((prev, prev_span)) => {
                    let merged = merge_tag_metas(
                        Some(prev),
                        prev_span.clone(),
                        Some(current),
                        span.clone(),
                    )?
                    .expect("both sides Some -> merge_tag_metas returns Some");
                    (merged, prev_span)
                }
            });
        }
    }
    Ok((peeled, acc.map(|(tm, _)| tm)))
}
