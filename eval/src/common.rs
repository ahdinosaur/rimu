use std::{cell::RefCell, rc::Rc};

use rimu_meta::{Span, Spanned};
use rimu_value::{BothTagged, Environment, Function, FunctionBody, SpannedValue, Value, ValueMeta};

use crate::{evaluate_block, evaluate_expression, EvalError, Result};

pub fn call(span: Span, function: Function, args: &[SpannedValue]) -> Result<SpannedValue> {
    let (args, tag_meta) = unwrap_tagged_args(args)?;

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

    Ok(rewrap_tagged(span, value, tag_meta))
}

type TagMeta = (String, ValueMeta);

/// Unwraps tagged arguments so the callee sees raw values, and collects a
/// single tag + merged meta to re-wrap onto the result. Same-tag metas are
/// merged (right-wins on key collision); differing tags error.
fn unwrap_tagged_args(args: &[SpannedValue]) -> Result<(Vec<SpannedValue>, Option<TagMeta>)> {
    let mut unwrapped = Vec::with_capacity(args.len());
    let mut combined: Option<(String, ValueMeta, Span)> = None;
    for arg in args {
        let (value, span) = arg.clone().take();
        match value {
            Value::Tagged { tag, inner, meta } => {
                combined = Some(match combined {
                    None => (tag, meta, span),
                    Some((existing_tag, mut existing_meta, existing_span)) => {
                        if existing_tag != tag {
                            return Err(EvalError::BothTagged(Box::new(BothTagged {
                                left_span: existing_span,
                                right_span: span,
                                left_tag: existing_tag,
                                right_tag: tag,
                            })));
                        }
                        for (key, value) in meta {
                            existing_meta.insert(key, value);
                        }
                        (existing_tag, existing_meta, existing_span)
                    }
                });
                unwrapped.push(*inner);
            }
            other => unwrapped.push(Spanned::new(other, span)),
        }
    }
    Ok((unwrapped, combined.map(|(tag, meta, _)| (tag, meta))))
}

fn rewrap_tagged(span: Span, value: SpannedValue, tag_meta: Option<TagMeta>) -> SpannedValue {
    match tag_meta {
        None => value,
        Some((tag, meta)) => Spanned::new(
            Value::Tagged {
                tag,
                inner: Box::new(value),
                meta,
            },
            span,
        ),
    }
}
