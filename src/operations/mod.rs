use serde::Deserialize;

use crate::{Context, Engine, Template, TemplateError, Value};

pub(crate) trait Operation {
    fn render(&self, engine: &Engine, context: &Context) -> Result<Template, TemplateError>;
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(tag = "type")]
pub enum Operations {
    #[serde(alias = "op.eval")]
    Eval(EvalOperation),
}

impl Operations {
    pub(crate) fn render(
        &self,
        engine: &Engine,
        context: &Context,
    ) -> Result<Template, TemplateError> {
        match self {
            Operations::Eval(eval) => eval.render(engine, context),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct EvalOperation {
    expr: Box<Template>,
}

impl Operation for EvalOperation {
    fn render(&self, engine: &Engine, context: &Context) -> Result<Template, TemplateError> {
        let expr = self.expr.as_ref();
        let value = engine.render(expr, context)?;
        let Value::String(expr) = value else {
            return Err(TemplateError::InvalidOperation { template: expr.clone() })
        };

        let value: Template = engine.evaluate(&expr, context)?;

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{
        context::Context,
        template::Template,
        value::{Number, Value},
        Engine,
    };

    #[test]
    fn test_eval() -> Result<(), Box<dyn Error>> {
        let content = r#"
zero:
  type: op.eval
  expr: one + 2
three:
  four: five
"#;

        let template: Template = serde_yaml::from_str(content).unwrap();

        let mut context = Context::new();
        context.insert("one", Value::Number(Number::Signed(98)));

        let engine = Engine::default();

        let value: Value = engine.render(&template, &context)?;

        println!("{:?}", value);

        Ok(())
    }
}
