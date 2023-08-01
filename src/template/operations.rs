use rhai::{Dynamic, Engine};
use serde::Deserialize;

use crate::{context::Context, value::Value};

use super::{evaluate::evaluate, Template, TemplateError};

pub(crate) trait Operation {
    fn evaluate(&self, context: &Context) -> Result<Template, TemplateError>;
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(tag = "type")]
pub enum Operations {
    #[serde(alias = "op.eval")]
    Eval(EvalOperation),
}

impl Operations {
    pub(crate) fn evaluate(&self, context: &Context) -> Result<Template, TemplateError> {
        match self {
            Operations::Eval(eval) => eval.evaluate(context),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct EvalOperation {
    expr: Box<Template>,
}

impl Operation for EvalOperation {
    fn evaluate(&self, context: &Context) -> Result<Template, TemplateError> {
        let expr = self.expr.as_ref();
        let value = evaluate(expr, context)?;
        let Value::String(expr) = value else {
            return Err(TemplateError::InvalidOperation { template: expr.clone() })
        };
        let mut rhai_scope = context.to_rhai_scope();
        let engine = Engine::new();

        let result: Dynamic = engine.eval_expression_with_scope(&mut rhai_scope, &expr)?;

        let value: Template = match rhai::serde::from_dynamic(&result) {
            Ok(value) => value,
            Err(error) => {
                panic!("Failed to convert dynamic to value: {}", error);
            }
        };

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{
        context::Context,
        template::{evaluate::evaluate, Template},
        value::{Number, Value},
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

        let value: Value = evaluate(&template, &context)?;

        println!("{:?}", value);

        Ok(())
    }
}
