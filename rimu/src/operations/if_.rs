use serde::Deserialize;

use super::Operation;
use crate::{Engine, Environment, RenderError, Template, Value};

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IfOperation {
    #[serde(rename = "$if")]
    pub condition: Box<Template>,
    #[serde(rename = "then")]
    pub consequent: Option<Box<Template>>,
    #[serde(rename = "else")]
    pub alternative: Option<Box<Template>>,
}

impl Operation for IfOperation {
    fn render(&self, engine: &Engine, env: &Environment) -> Result<Value, RenderError> {
        let condition = engine.render(&self.condition, env)?;

        let value: Value = if let Value::String(condition) = condition {
            engine.evaluate(&condition, env)?
        } else {
            condition
        };

        if Into::<bool>::into(value) {
            if let Some(consequent) = &self.consequent {
                engine.render(consequent, env)
            } else {
                Ok(Value::Null)
            }
        } else {
            if let Some(alternative) = &self.alternative {
                engine.render(alternative, env)
            } else {
                Ok(Value::Null)
            }
        }
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
    fn if_() -> Result<(), Box<dyn Error>> {
        let content = r#"
zero:
  $if: ten > five
  then:
    $eval: five
  else:
    $eval: ten
"#;
        let template: Template = serde_yaml::from_str(content)?;

        let engine = Engine::default();
        let mut context = Environment::new();
        context.insert("five", Value::Number(dec!(5).into()));
        context.insert("ten", Value::Number(dec!(10).into()));

        let actual: Value = engine.render(&template, &context)?;

        let expected: Value = Value::Object(btree_map! {
            "zero".into() => Value::Number(dec!(5).into())
        });

        assert_eq!(expected, actual);

        Ok(())
    }
}
