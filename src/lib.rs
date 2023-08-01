mod context;
mod engine;
mod error;
mod operations;
mod template;
mod value;

pub use context::Context;
pub use engine::Engine;
pub use error::{ParseError, RenderError};
pub use template::Template;
pub use value::{convert, from_value, List, Number, Object, Value, ValueError};

#[cfg(test)]
mod tests {
    use std::error::Error;

    use map_macro::btree_map;
    use pretty_assertions::assert_eq;
    use toml;

    use crate::{Context, Engine, Template, Value};

    #[test]
    fn something() -> Result<(), Box<dyn Error>> {
        let content = r#"
[one]
"$eval" = "a"

[two]
three = "${ b }"
"#;
        let tmpl: Template = toml::from_str(content)?;

        let engine = Engine::default();
        let mut ctx = Context::new();
        ctx.insert("a", "apples");
        ctx.insert("b", "blueberries");

        let actual: Value = engine.render(&tmpl, &ctx)?;

        let expected: Value = Value::Object(btree_map! {
            "one".into() => Value::String("apples".into()),
            "two".into() => Value::Object(btree_map! {
                "three".into() => Value::String("blueberries".into())
            })
        });

        assert_eq!(expected, actual);

        Ok(())
    }
}
