use serde::Deserialize;

use super::Operation;
use crate::{Engine, Environment, RenderError, Template, Value};

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LetOperation {
    #[serde(rename = "$let")]
    pub variables: Box<Template>,
    #[serde(rename = "in")]
    pub body: Box<Template>,
}

impl Operation for LetOperation {
    fn render(&self, engine: &Engine, env: &Environment) -> Result<Value, RenderError> {
        let variables = engine.render(&self.variables, env)?;

        let context = Environment::from_value(&variables, Some(env))?;

        engine.render(&self.body, &context)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;
    use crate::Value;

    use map_macro::btree_map;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    #[test]
    fn let_() -> Result<(), Box<dyn Error>> {
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
        let mut context = Environment::new();
        context.insert("ten", Value::Number(dec!(10).into()));

        let actual: Value = engine.render(&template, &context)?;

        let expected: Value = Value::Object(btree_map! {
            "zero".into() => Value::Object(btree_map! {
                "three".into() => Value::Number(dec!(12).into())
            })
        });

        assert_eq!(expected, actual);

        Ok(())
    }
}
