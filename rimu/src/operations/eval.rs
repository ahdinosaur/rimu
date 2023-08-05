use serde::Deserialize;

use super::Operation;
use crate::{Context, Engine, RenderError, Template};

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct EvalOperation {
    #[serde(alias = "$eval")]
    pub expr: String,
}

impl Operation for EvalOperation {
    fn render(&self, engine: &Engine, context: &Context) -> Result<Template, RenderError> {
        let value: Template = engine.evaluate(&self.expr, context)?;

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;
    use crate::{Number, Value};

    use map_macro::btree_map;
    use pretty_assertions::assert_eq;

    #[test]
    fn eval() -> Result<(), Box<dyn Error>> {
        let content = r#"
zero:
  $eval: one + 2
three:
  four: five
"#;
        let template: Template = serde_yaml::from_str(content)?;

        let engine = Engine::default();
        let mut context = Context::new();
        context.insert("one", Value::Number(Number::Signed(98)));

        let actual: Value = engine.render(&template, &context)?;

        let expected: Value = Value::Object(btree_map! {
            "zero".into() => Value::Number(100.into()),
            "three".into() => Value::Object(btree_map! {
                "four".into() => Value::String("five".into())
            })
        });

        assert_eq!(expected, actual);

        Ok(())
    }
}
