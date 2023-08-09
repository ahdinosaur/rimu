use serde::Deserialize;

use super::Operation;
use crate::{Context, Engine, RenderError, Template, Value};

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
    fn render(&self, engine: &Engine, context: &Context) -> Result<Value, RenderError> {
        let condition = engine.render(&self.condition, context)?;

        let value: Value = if let Value::String(condition) = condition {
            engine.evaluate(&condition, context)?
        } else {
            condition
        };

        if value.is_truthy() {
            if let Some(consequent) = &self.consequent {
                engine.render(consequent, context)
            } else {
                Ok(Value::Null)
            }
        } else {
            if let Some(alternative) = &self.alternative {
                engine.render(alternative, context)
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
    use crate::{Number, Value};

    use map_macro::btree_map;
    use pretty_assertions::assert_eq;

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
        let mut context = Context::new();
        context.insert("five", Value::Number(Number::Signed(5)));
        context.insert("ten", Value::Number(Number::Signed(10)));

        let actual: Value = engine.render(&template, &context)?;

        let expected: Value = Value::Object(btree_map! {
            "zero".into() => Value::Number(Number::Signed(5))
        });

        assert_eq!(expected, actual);

        Ok(())
    }
}
