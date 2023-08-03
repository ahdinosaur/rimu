use pretty_assertions::assert_eq;
use rimu::{from_value, Context, Engine, Template, Value};
use std::{fs, path::Path};

fn test_spec(path: &Path) -> datatest_stable::Result<()> {
    let content = fs::read_to_string(path)?;
    let specs: Value = serde_yaml::from_str(&content)?;

    let Value::List(specs) = specs else {
        panic!("Specs should be list");
    };

    for spec in specs {
        let Value::Object(spec) = spec else {
            panic!("Spec should be object");
        };

        let title: Value = spec.get("title").expect("title should exist").clone();
        let template: Value = spec.get("template").expect("template should exist").clone();
        let context_val: Value = spec.get("context").expect("context should exist").clone();
        let output: Value = spec.get("output").expect("output should exist").clone();

        let title: String = from_value(title)?;
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

        assert_eq!(output, actual, "{}", title);
    }

    Ok(())
}

datatest_stable::harness!(test_spec, "./tests/spec", r"^.*/*");
