use rimu::{from_value, Context, Engine, Template, Value};
use std::{fs, path::Path};

fn test_spec(path: &Path) -> datatest_stable::Result<()> {
    let content = fs::read_to_string(path)?;
    let value: Value = serde_yaml::from_str(&content)?;

    let Value::Object(value) = value else {
        panic!("Spec should be object");
    };

    let template: Value = value
        .get("template")
        .expect("template should exist")
        .clone();
    let context_val: Value = value.get("context").expect("context should exist").clone();
    let output: Value = value.get("output").expect("output should exist").clone();

    let template: Template = from_value(template)?;

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

datatest_stable::harness!(test_spec, "./tests/spec", r"^.*/*");
