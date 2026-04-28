use std::{cell::RefCell, path::PathBuf, rc::Rc, slice::from_ref};

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
    lib.insert("host_path".into(), host_path().into());
    lib.insert("target_path".into(), target_path().into());
    lib.insert("to_string".into(), to_string().into());
    lib
}

fn empty_env() -> Rc<RefCell<Environment>> {
    Rc::new(RefCell::new(Environment::new()))
}

pub fn length() -> Function {
    let function = |span: Span, args: &[Spanned<Value>]| -> Result<SpannedValue, EvalError> {
        let (arg, arg_span) = &args[0].clone().take();
        let value = match arg {
            Value::List(list) => list.len().into(),
            Value::String(string) => string.len().into(),
            _ => {
                return Err(EvalError::TypeError {
                    span: arg_span.clone(),
                    expected: "list | string".into(),
                    got: Box::new(arg.clone().into()),
                })
            }
        };
        Ok(Spanned::new(value, span))
    };
    Function {
        args: vec!["arg".into()],
        env: empty_env(),
        body: FunctionBody::Native(NativeFunction::new("length", function)),
    }
}
pub fn map() -> Function {
    let function = |span: Span, args: &[Spanned<Value>]| -> Result<SpannedValue, EvalError> {
        let (arg, arg_span) = &args[0].clone().take();
        match arg {
            Value::Object(object) => {
                let list_arg = object.get("list").map(|a| a.inner());
                let mapper_arg = object.get("each").map(|a| a.inner());
                match (list_arg, mapper_arg) {
                    (Some(Value::List(list)), Some(Value::Function(mapper))) => map_op(
                        span,
                        MapOptions {
                            list: list.clone(),
                            mapper: mapper.clone(),
                        },
                    ),
                    _ => Err(EvalError::TypeError {
                        span: arg_span.clone(),
                        expected: "{ list: list, each: (item) => next }".into(),
                        got: Box::new(arg.clone().into()),
                    }),
                }
            }
            _ => Err(EvalError::TypeError {
                span: arg_span.clone(),
                expected: "object".into(),
                got: Box::new(arg.clone().into()),
            }),
        }
    };

    Function {
        args: vec!["arg".into()],
        env: empty_env(),
        body: FunctionBody::Native(NativeFunction::new("map", function)),
    }
}

struct MapOptions {
    list: Vec<SpannedValue>,
    mapper: Function,
}

fn map_op(span: Span, options: MapOptions) -> Result<SpannedValue, EvalError> {
    let MapOptions { list, mapper } = options;
    let next_list = list
        .iter()
        .map(|item| call(span.clone(), mapper.clone(), from_ref(item)))
        .collect::<Result<Vec<SpannedValue>, EvalError>>()?;
    Ok(Spanned::new(Value::List(next_list), span))
}

pub fn range() -> Function {
    let function = |span: Span, args: &[Spanned<Value>]| -> Result<SpannedValue, EvalError> {
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
                        let end = end.to_usize().ok_or_else(|| EvalError::TypeError {
                            span: end_span,
                            expected: "zero or positive integer".into(),
                            got: Box::new(SerdeValue::Number(end)),
                        })?;
                        range_op(span, RangeOptions { start: None, end })
                    }
                    (
                        Some((Value::Number(start), start_span)),
                        Some((Value::Number(end), end_span)),
                    ) => {
                        let start = start.to_usize().ok_or_else(|| EvalError::TypeError {
                            span: start_span,
                            expected: "zero or positive integer".into(),
                            got: Box::new(SerdeValue::Number(start)),
                        })?;
                        let end = end.to_usize().ok_or_else(|| EvalError::TypeError {
                            span: end_span,
                            expected: "zero or positive integer".into(),
                            got: Box::new(SerdeValue::Number(end)),
                        })?;
                        range_op(
                            span,
                            RangeOptions {
                                start: Some(start),
                                end,
                            },
                        )
                    }
                    _ => Err(EvalError::TypeError {
                        span: arg_span.clone(),
                        expected: "{ start?: number, end: number }".into(),
                        got: Box::new(arg.clone().into()),
                    }),
                }
            }
            _ => Err(EvalError::TypeError {
                span: arg_span.clone(),
                expected: "object".into(),
                got: Box::new(arg.clone().into()),
            }),
        }
    };

    Function {
        args: vec!["arg".into()],
        env: empty_env(),
        body: FunctionBody::Native(NativeFunction::new("range", function)),
    }
}

struct RangeOptions {
    start: Option<usize>,
    end: usize,
}

fn range_op(span: Span, options: RangeOptions) -> Result<SpannedValue, EvalError> {
    let RangeOptions { start, end } = options;
    let start = start.unwrap_or(0);
    let list = (start..end).map(Into::into).collect();
    Ok(SerdeValue::List(list).with_span(span))
}

/// Construct a [`Value::HostPath`] from a relative string, resolved against
/// the directory of the source file the call appears in. The resolved path is
/// absolute when the source id is itself an absolute path, so it can be
/// forwarded across sub-plan boundaries without losing its anchor.
///
/// Errors when the source id has no parent directory (e.g. an empty source
/// id). Sources without a directory component (e.g. `"test.rimu"`) resolve
/// against the current working directory.
pub fn host_path() -> Function {
    let function = |span: Span, args: &[Spanned<Value>]| -> Result<SpannedValue, EvalError> {
        let (arg, arg_span) = &args[0].clone().take();
        let Value::String(rel) = arg else {
            return Err(EvalError::TypeError {
                span: arg_span.clone(),
                expected: "string".into(),
                got: Box::new(arg.clone().into()),
            });
        };
        let source_path = PathBuf::from(span.source().as_str());
        // Note(cc): "no parent" is reported via TypeError because no other
        // EvalError variant fits today. The user-facing message is meaningful
        // even if the variant is a stretch — consider a dedicated variant
        // (e.g. `MissingSourceContext`) if more stdlib functions need it.
        let Some(source_dir) = source_path.parent() else {
            return Err(EvalError::TypeError {
                span: span.clone(),
                expected: "source id with a parent directory (host_path needs a directory to resolve against)".into(),
                got: Box::new(SerdeValue::String(span.source().as_str().to_string())),
            });
        };
        let path = source_dir.join(rel);
        Ok(Spanned::new(Value::HostPath(path), span))
    };
    Function {
        args: vec!["arg".into()],
        env: empty_env(),
        body: FunctionBody::Native(NativeFunction::new("host_path", function)),
    }
}

/// Construct a [`Value::TargetPath`] from an absolute unix path string. Errors
/// if the input doesn't start with `/` — the type's contract is "absolute path
/// on a remote unix host", and a relative path here is almost always a bug at
/// the call site.
pub fn target_path() -> Function {
    let function = |span: Span, args: &[Spanned<Value>]| -> Result<SpannedValue, EvalError> {
        let (arg, arg_span) = &args[0].clone().take();
        let Value::String(s) = arg else {
            return Err(EvalError::TypeError {
                span: arg_span.clone(),
                expected: "string".into(),
                got: Box::new(arg.clone().into()),
            });
        };
        if !s.starts_with('/') {
            return Err(EvalError::TypeError {
                span: arg_span.clone(),
                expected: "absolute unix path (starts with '/')".into(),
                got: Box::new(arg.clone().into()),
            });
        }
        Ok(Spanned::new(Value::TargetPath(s.clone()), span))
    };
    Function {
        args: vec!["arg".into()],
        env: empty_env(),
        body: FunctionBody::Native(NativeFunction::new("target_path", function)),
    }
}

/// Render a value as a [`Value::String`]. The escape hatch for paths: a plan
/// that wants to embed a path inside a string (e.g. a podman volume entry
/// `<host>:/<container>`) can call `to_string` to drop the path type and
/// continue with plain string concatenation.
pub fn to_string() -> Function {
    let function = |span: Span, args: &[Spanned<Value>]| -> Result<SpannedValue, EvalError> {
        let (arg, arg_span) = &args[0].clone().take();
        let s = match arg {
            Value::String(s) => s.clone(),
            Value::HostPath(p) => p.display().to_string(),
            Value::TargetPath(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Boolean(b) => b.to_string(),
            _ => {
                return Err(EvalError::TypeError {
                    span: arg_span.clone(),
                    expected: "string | host-path | target-path | number | boolean".into(),
                    got: Box::new(arg.clone().into()),
                });
            }
        };
        Ok(Spanned::new(Value::String(s), span))
    };
    Function {
        args: vec!["arg".into()],
        env: empty_env(),
        body: FunctionBody::Native(NativeFunction::new("to_string", function)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rimu_meta::SourceId;
    use rimu_parse::parse_expression;

    fn span_with_source(source: SourceId) -> Span {
        Span::new(source, 0, 0)
    }

    /// Run `code` with the stdlib in scope and a source id whose parent is
    /// `/tmp` — `host_path("./x")` resolves to `/tmp/x`.
    fn eval_with_stdlib(code: &str) -> Result<Value, EvalError> {
        let source = SourceId::from("/tmp/test.rimu".to_string());
        let (Some(expr), errors) = parse_expression(code, source.clone()) else {
            panic!("parse failed");
        };
        assert_eq!(errors.len(), 0, "parse errors: {:?}", errors);

        let mut env = Environment::new();
        for (key, value) in create_stdlib() {
            env.insert(key, value);
        }
        let env = Rc::new(RefCell::new(env));

        rimu_eval::evaluate_expression(&expr, env).map(|v| v.into_inner())
    }

    #[test]
    fn host_path_resolves_against_source_dir() {
        let path_fn = host_path();
        let args = vec![Spanned::new(
            Value::String("./gitconfig".into()),
            span_with_source(SourceId::empty()),
        )];
        let span = span_with_source(SourceId::from("/plans/parent.rimu".to_string()));
        let result = rimu_eval::call(span, path_fn, &args).unwrap();
        let Value::HostPath(path) = result.into_inner() else {
            panic!("expected HostPath");
        };
        assert_eq!(path, PathBuf::from("/plans/gitconfig"));
    }

    #[test]
    fn host_path_errors_when_source_has_no_parent() {
        let path_fn = host_path();
        let args = vec![Spanned::new(
            Value::String("./foo".into()),
            span_with_source(SourceId::empty()),
        )];
        // SourceId::empty() is "" — PathBuf::from("").parent() is None.
        let span = span_with_source(SourceId::empty());
        let err = rimu_eval::call(span, path_fn, &args).unwrap_err();
        assert!(matches!(err, EvalError::TypeError { .. }));
    }

    #[test]
    fn target_path_accepts_absolute() {
        let path_fn = target_path();
        let args = vec![Spanned::new(
            Value::String("/etc/foo".into()),
            span_with_source(SourceId::empty()),
        )];
        let result = rimu_eval::call(span_with_source(SourceId::empty()), path_fn, &args).unwrap();
        assert_eq!(result.into_inner(), Value::TargetPath("/etc/foo".into()));
    }

    #[test]
    fn target_path_rejects_relative() {
        let path_fn = target_path();
        let args = vec![Spanned::new(
            Value::String("etc/foo".into()),
            span_with_source(SourceId::empty()),
        )];
        let err = rimu_eval::call(span_with_source(SourceId::empty()), path_fn, &args).unwrap_err();
        assert!(matches!(err, EvalError::TypeError { .. }));
    }

    #[test]
    fn host_path_plus_absolute_string_replaces_base() {
        // PathBuf::join: a leading "/" on the right replaces the base. That's
        // Rust convention; users wanting to extend in-place should pass "sub"
        // (no leading slash). This test pins the replacement behaviour so we
        // notice if PathBuf::join semantics change, or we decide to reject
        // absolute strings on the right (see TODO(cc) in eval/src/expression.rs).
        let actual = eval_with_stdlib(r#"host_path("./gitconfig") + "/sub""#).unwrap();
        assert_eq!(actual, Value::HostPath(PathBuf::from("/sub")));
    }

    #[test]
    fn host_path_plus_relative_string_extends_in_place() {
        // Note: PathBuf::join does not normalize "./" — the result keeps the
        // leading "./" from the input. Matches existing lusid semantics.
        let actual = eval_with_stdlib(r#"host_path("./dir") + "sub""#).unwrap();
        assert_eq!(actual, Value::HostPath(PathBuf::from("/tmp/./dir/sub")));
    }

    #[test]
    fn target_path_plus_string_concats() {
        let actual = eval_with_stdlib(r#"target_path("/etc") + "/foo""#).unwrap();
        assert_eq!(actual, Value::TargetPath("/etc/foo".into()));
    }

    #[test]
    fn string_plus_target_path_drops_to_plain_string() {
        // The TODO note from wormfarm: a podman volume entry like
        // "<host>:/<container>" should not inherit the target-path type.
        let actual = eval_with_stdlib(r#""prefix:" + target_path("/etc")"#).unwrap();
        assert_eq!(actual, Value::String("prefix:/etc".into()));
    }

    #[test]
    fn to_string_renders_host_path() {
        let actual = eval_with_stdlib(r#"to_string(host_path("./foo"))"#).unwrap();
        assert_eq!(actual, Value::String("/tmp/./foo".into()));
    }

    #[test]
    fn to_string_renders_target_path() {
        let actual = eval_with_stdlib(r#"to_string(target_path("/etc"))"#).unwrap();
        assert_eq!(actual, Value::String("/etc".into()));
    }

    #[test]
    fn path_plus_path_errors() {
        let err = eval_with_stdlib(r#"host_path("./a") + target_path("/b")"#).unwrap_err();
        assert!(matches!(err, EvalError::TypeError { .. }));
    }
}
