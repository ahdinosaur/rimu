use serde::Deserialize;

use super::Block;
use crate::{Engine, Environment, RenderError, Value};

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvalBlock {
    #[serde(alias = "$eval")]
    pub expr: String,
}

impl Block for EvalBlock {
    fn render(&self, engine: &Engine, context: &Environment) -> Result<Value, RenderError> {
        engine.evaluate(&self.expr, context)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;
    use crate::{Template, Value};

    use map_macro::btree_map;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

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
        let mut context = Environment::new();
        context.insert("one", Value::Number(dec!(98).into()));

        let actual: Value = engine.render(&template, &context)?;

        let expected: Value = Value::Object(btree_map! {
            "zero".into() => Value::Number(dec!(100).into()),
            "three".into() => Value::Object(btree_map! {
                "four".into() => Value::String("five".into())
            })
        });

        assert_eq!(expected, actual);

        Ok(())
    }
}
