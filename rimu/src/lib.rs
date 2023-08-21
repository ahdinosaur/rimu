mod blocks;
mod engine;
mod error;
mod template;

pub use engine::Engine;
pub use error::{ParseError, RenderError};
pub use rimu_env::{Environment, EnvironmentError};
pub use rimu_eval::evaluate;
pub use rimu_expr::parse;
pub use rimu_value::{convert, from_value, List, Number, Object, Value, ValueError};
pub use template::Template;

#[cfg(test)]
mod tests {
    use std::error::Error;

    use map_macro::btree_map;
    use pretty_assertions::assert_eq;
    use toml;

    use crate::{Engine, Environment, Template, Value};

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
        let mut ctx = Environment::new();
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
