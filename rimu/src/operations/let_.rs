use serde::Deserialize;

use super::Operation;
use crate::{Context, Engine, RenderError, Template, Value};

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct LetOperation {
    #[serde(rename = "$let")]
    pub variables: Box<Template>,
    #[serde(rename = "in")]
    pub body: Box<Template>,
}

impl Operation for LetOperation {
    fn render(&self, engine: &Engine, context: &Context) -> Result<Value, RenderError> {
        let variables = engine.render(&self.variables, context)?;

        let context = Context::from_value(&variables, Some(context))?;

        engine.render(&self.body, &context)
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
  $let:
    one:
      $eval: ten
    two: 2
  in:
    three:
      $eval: one + two
"#;
        let template: Template = serde_yaml::from_str(content)?;

        let engine = Engine::default();
        let mut context = Context::new();
        context.insert("ten", Value::Number(Number::Signed(10)));

        let actual: Value = engine.render(&template, &context)?;

        let expected: Value = Value::Object(btree_map! {
            "zero".into() => Value::Object(btree_map! {
                "three".into() => Value::Number(Number::Signed(12))
            })
        });

        assert_eq!(expected, actual);

        Ok(())
    }
}
