use serde::Deserialize;

use super::Block;
use crate::{Environment, Engine, RenderError, Template, Value};

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LetBlock {
    #[serde(rename = "$let")]
    pub variables: Box<Template>,
    #[serde(rename = "in")]
    pub body: Box<Template>,
}

impl Block for LetBlock {
    fn render(&self, engine: &Engine, context: &Environment) -> Result<Value, RenderError> {
        let variables = engine.render(&self.variables, context)?;

        let context = Environment::from_value(&variables, Some(context))?;

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
