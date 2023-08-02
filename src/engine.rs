use rhai::Dynamic;

use crate::{interpolate::interpolate, Context, Object, Template, TemplateError, Value};

pub struct Engine {
    rhai: rhai::Engine,
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            rhai: rhai::Engine::new(),
        }
    }
}

impl Engine {
    pub fn new(rhai: rhai::Engine) -> Self {
        Self { rhai }
    }

    pub fn render(&self, template: &Template, context: &Context) -> Result<Value, TemplateError> {
        match template {
            Template::Null => Ok(Value::Null),
            Template::Boolean(boolean) => Ok(Value::Boolean(boolean.clone())),
            Template::Number(number) => Ok(Value::Number(number.clone())),
            Template::String(string) => {
                let next_string = interpolate(string, context)?;
                Ok(Value::String(next_string))
            }
            Template::List(list) => {
                let next_list = list
                    .into_iter()
                    .filter_map(|item| match self.render(item, context) {
                        Ok(Value::Null) => None,
                        Ok(template) => Some(Ok(template)),
                        Err(error) => Some(Err(error)),
                    })
                    .collect::<Result<Vec<Value>, TemplateError>>()?;
                Ok(Value::List(next_list))
            }
            Template::Object(object) => {
                let mut next_object = Object::new();
                for (key, template) in object.iter() {
                    match self.render(template, context)? {
                        Value::Null => {}
                        template => {
                            next_object.insert(interpolate(key, context)?, template);
                        }
                    };
                }
                Ok(Value::Object(next_object))
            }
            Template::Operation(operation) => {
                let next_template = operation.render(&self, context)?;
                self.render(&next_template, context)
            }
        }
    }

    pub fn evaluate(&self, expr: &str, context: &Context) -> Result<Template, TemplateError> {
        let mut rhai_scope = context.to_rhai_scope();

        let result: Dynamic = self
            .rhai
            .eval_expression_with_scope(&mut rhai_scope, &expr)?;

        let value: Template = match rhai::serde::from_dynamic(&result) {
            Ok(value) => value,
            Err(error) => {
                panic!("Failed to convert dynamic to value: {}", error);
            }
        };

        Ok(value)
    }
}
