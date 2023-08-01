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

        let title: Value = spec.get("title").expect("Spec missing 'title'").clone();
        let template: Value = spec
            .get("template")
            .expect("Spec missing 'template'")
            .clone();
        let context_val: Value = spec.get("context").expect("Spec missing 'context'").clone();

        let title: String = from_value(title)?;
        let template: Template = from_value(template)?;

        let engine = Engine::default();

        let mut context = Context::new();
        let Value::Object(context_obj) = context_val else {
            panic!("Spec 'context' must be object");
        };
        for (key, value) in context_obj.into_iter() {
            context.insert(key, value);
        }

        if let Some(output) = spec.get("output") {
            let actual = engine
                .render(&template, &context)
                .expect("Expected render output");

            assert_eq!(output.clone(), actual, "{} : output", title);
        } else if let Some(error) = spec.get("error") {
            let Value::Object(error) = error else {
                panic!("Spec 'error' should be object");
            };
            let Value::String(message) = error.get("message").expect("Spec missing 'error.message'") else {
                panic!("Spec 'error.message' should be string");
            };

            let actual = engine
                .render(&template, &context)
                .expect_err("Expected render error");

            assert_eq!(message, &actual.to_string(), "{} : error name", title);
        } else {
            panic!("Spec missing 'output' or 'error'")
        }
    }

    Ok(())
}

datatest_stable::harness!(test_spec, "./tests/spec", r"^.*/*");
