//! Cross-boundary invariants for typed `Value` variants in `Environment`.

use std::{cell::RefCell, path::PathBuf, rc::Rc};

use pretty_assertions::assert_eq;
use rimu::{
    create_stdlib, evaluate, evaluate_expression, parse, parse_expression, Environment, SourceId,
    Value,
};
use typed_path::Utf8TypedPathBuf;

/// Evaluate a block-shaped Rimu program with the stdlib in scope and a source
/// id whose parent dir is `/tmp` so `host_path("./x")` resolves to `/tmp/x`.
fn eval_block(code: &str) -> Value {
    let source = SourceId::from("/tmp/test.rimu".to_string());
    let (block, errors) = parse(code, source);
    assert!(errors.is_empty(), "parse errors: {errors:?}");
    let block = block.expect("block parsed");

    let mut env = Environment::new();
    for (key, value) in create_stdlib() {
        env.insert(key, value);
    }
    let env = Rc::new(RefCell::new(env));

    evaluate(&block, env)
        .expect("evaluation succeeded")
        .into_inner()
}

/// Evaluate an expression-shaped Rimu program (same env as `eval_block`).
fn eval_expr(code: &str) -> Value {
    let source = SourceId::from("/tmp/test.rimu".to_string());
    let (expr, errors) = parse_expression(code, source);
    assert!(errors.is_empty(), "parse errors: {errors:?}");
    let expr = expr.expect("expression parsed");

    let mut env = Environment::new();
    for (key, value) in create_stdlib() {
        env.insert(key, value);
    }
    let env = Rc::new(RefCell::new(env));

    evaluate_expression(&expr, env)
        .expect("evaluation succeeded")
        .into_inner()
}

#[test]
fn host_path_survives_function_arg() {
    // The actual lusid bug fix: a typed `Value::HostPath` passed as a
    // function argument must arrive in the body env still typed.
    let actual = eval_expr("((p) => p)(host_path(\"./x\"))");
    assert_eq!(actual, Value::HostPath(PathBuf::from("/tmp/./x")));
}

#[test]
fn target_path_survives_function_arg() {
    let actual = eval_expr("((p) => p)(target_path(\"/etc/foo\"))");
    assert_eq!(
        actual,
        Value::TargetPath(Utf8TypedPathBuf::from_unix("/etc/foo"))
    );
}

#[test]
fn host_path_survives_let_binding() {
    // The other binding boundary: `let p: <typed> in p` must preserve type.
    let code = "
out:
  let
    p: host_path(\"./x\")
  in
    p
";
    let actual = eval_block(code);
    let inner = match actual {
        Value::Object(obj) => obj.get("out").expect("out present").clone().into_inner(),
        other => panic!("expected object, got {other:?}"),
    };
    assert_eq!(inner, Value::HostPath(PathBuf::from("/tmp/./x")));
}

#[test]
fn host_path_in_object_field_survives_let() {
    // Recursive case: typed leaf inside a nested object inside a let binding.
    // Exercises the `let_` rewrite, the env round-trip, and the per-element
    // span propagation through `Value::Object`.
    let code = "
out:
  let
    o:
      p: host_path(\"./y\")
  in
    o.p
";
    let actual = eval_block(code);
    let inner = match actual {
        Value::Object(obj) => obj.get("out").expect("out present").clone().into_inner(),
        other => panic!("expected object, got {other:?}"),
    };
    assert_eq!(inner, Value::HostPath(PathBuf::from("/tmp/./y")));
}

#[test]
fn host_path_survives_map_callback() {
    // `stdlib::map` calls user functions via `eval::call` — same binding
    // path as a sub-plan call. Catches loss in the iteration site.
    let actual =
        eval_expr("map({ list: [host_path(\"./a\"), host_path(\"./b\")], each: (item) => item })");
    let Value::List(items) = actual else {
        panic!("expected list, got {actual:?}");
    };
    let inners: Vec<_> = items.into_iter().map(|i| i.into_inner()).collect();
    assert_eq!(
        inners,
        vec![
            Value::HostPath(PathBuf::from("/tmp/./a")),
            Value::HostPath(PathBuf::from("/tmp/./b")),
        ]
    );
}
