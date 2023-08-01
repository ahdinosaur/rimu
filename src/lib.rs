mod context;
mod template;
mod value;

pub use context::Context;
pub use template::{Template, TemplateError};
pub use value::{convert, from_value, List, Number, Object, Value, ValueError};

#[cfg(test)]
mod tests {
    use std::error::Error;

    use toml;

    use crate::{Context, Template, Value};

    #[test]
    fn test_something() -> Result<(), Box<dyn Error>> {
        let content = r#"
[one]
type = "op.eval"
expr = "a"

[two]
three = "{{ b }}"
"#;

        let mut ctx = Context::new();
        ctx.insert("a", "apples");
        ctx.insert("b", "blueberries");

        let tmpl: Template = toml::from_str(content).unwrap();

        let value: Value = tmpl.evaluate(&ctx)?;
        println!("{:?}", value);

        Ok(())
    }
}
