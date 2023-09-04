use pretty_assertions::assert_eq;
use rimu::{evaluate, from_value, parse, Environment, SourceId, Value /*ValueError*/};
use std::error::Error;

#[track_caller]
fn test_spec(spec: Value) -> Result<(), Box<dyn Error>> {
    let Value::Object(spec) = spec else {
            panic!("Spec should be object");
        };

    let title: Value = spec.get("title").expect("Spec missing 'title'").clone();
    let template: Value = spec
        .get("template")
        .expect("Spec missing 'template'")
        .clone();
    println!("Template:");
    println!("{}", template);
    let Value::String(template) = template else {
        panic!("Spec 'template' must be (folded) string");
    };
    let context_val: Value = spec.get("context").expect("Spec missing 'context'").clone();

    let title: String = from_value(title)?;

    let mut context = Environment::new();
    let Value::Object(context_obj) = context_val else {
            panic!("Spec 'context' must be object");
        };
    for (key, value) in context_obj.into_iter() {
        context.insert(key, value);
    }

    if let Some(output) = spec.get("output") {
        let (template, errors) = parse(&template, SourceId::empty());

        if errors.len() > 0 {
            panic!("ParseError: {:?}", errors[0]);
        }
        let Some(template) = template else {
            panic!("Failed to parse template");
        };

        let actual = evaluate(&template, &context)?;

        assert_eq!(output.clone(), actual, "{} : output", title);
    } else if let Some(error) = spec.get("error") {
        let Value::Object(error) = error else {
                panic!("Spec 'error' should be object");
            };
        let Value::String(_message) = error.get("message").expect("Spec missing 'error.message'") else {
                panic!("Spec 'error.message' should be string");
            };
        let default_error_type = Value::String("RenderError".into());
        let Value::String(type_) = error.get("type").unwrap_or(&default_error_type) else {
                panic!("Spec 'error.type' should be string");
            };

        match type_.as_str() {
            "ParseError" => {
                unimplemented!();
                /*
                let (template, errors) = parse(&template, SourceId::empty());

                if errors.len() == 0 {
                    panic!("Expected parse error");
                }

                let actual = errors[0];

                assert_eq!(message, &actual.to_string(), "{} : error name", title);
                */
            }
            "EvalError" => {
                unimplemented!();
                /*
                let template: Template = from_value(template)?;
                let actual = engine
                    .render(&template, &context)
                    .expect_err("Expected render error");

                assert_eq!(message, &actual.to_string(), "{} : error name", title);
                */
            }
            _ => {
                panic!("Unexpected error type: {}", type_)
            }
        }
    } else {
        panic!("Spec missing 'output' or 'error'")
    }

    Ok(())
}

fn test_specs(content: &str) -> Result<(), Box<dyn Error>> {
    let specs: Value = serde_yaml::from_str(content)?;

    let Value::List(specs) = specs else {
        panic!("Specs should be list");
    };

    for spec in specs {
        test_spec(spec)?;
    }

    Ok(())
}

#[test]
fn identity() -> Result<(), Box<dyn Error>> {
    test_specs(include_str!("./spec/identity.yml"))
}

/*
#[test]
fn interpolation() -> Result<(), Box<dyn Error>> {
    test_specs(include_str!("./spec/interpolation.yml"))
}
*/

#[test]
fn let_() -> Result<(), Box<dyn Error>> {
    test_specs(include_str!("./spec/let.yml"))
}

#[test]
fn if_() -> Result<(), Box<dyn Error>> {
    test_specs(include_str!("./spec/if.yml"))
}
