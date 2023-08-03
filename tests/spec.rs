use rimu::{Context, Engine, Template, Value};
use std::error::Error;

#[track_caller]
fn test_spec(template: &str, context: &str, output: &str) -> Result<(), Box<dyn Error>> {
    let template: Template = serde_yaml::from_str(template)?;
    let context_val: Value = serde_yaml::from_str(context)?;
    let output: Value = serde_yaml::from_str(output)?;

    let engine = Engine::default();

    let mut context = Context::new();
    let Value::Object(context_obj) = context_val else {
        panic!("context.yml must be object");
    };
    for (key, value) in context_obj.into_iter() {
        context.insert(key, value);
    }

    let actual: Value = engine.render(&template, &context)?;

    assert_eq!(output, actual);

    Ok(())
}

#[test]
fn eval_identity() -> Result<(), Box<dyn Error>> {
    test_spec(
        include_str!("./spec/identity/template.yml"),
        include_str!("./spec/identity/context.yml"),
        include_str!("./spec/identity/output.yml"),
    )?;
    Ok(())
}

#[test]
fn eval() -> Result<(), Box<dyn Error>> {
    test_spec(
        include_str!("./spec/eval/template.yml"),
        include_str!("./spec/eval/context.yml"),
        include_str!("./spec/eval/output.yml"),
    )?;
    Ok(())
}

#[test]
fn eval_no_context() -> Result<(), Box<dyn Error>> {
    test_spec(
        include_str!("./spec/eval-no-context/template.yml"),
        include_str!("./spec/eval-no-context/context.yml"),
        include_str!("./spec/eval-no-context/output.yml"),
    )?;
    Ok(())
}
