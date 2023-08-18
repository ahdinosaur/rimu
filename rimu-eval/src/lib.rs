use rimu_env::{Environment, EnvironmentError};
pub use rimu_expr::Expression;
use rimu_value::{Number, Value};

#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    #[error("environment error: {0}")]
    Environment(#[source] EnvironmentError),
}

pub fn evaluate(expr: Expression, _env: &Environment) -> Result<Value, EvalError> {
    let value = match expr {
        Expression::Null => Value::Null,

        Expression::Boolean(boolean) => Value::Boolean(boolean),

        Expression::String(string) => Value::String(string),

        Expression::Number(decimal) => Value::Number(decimal.into()),

        Expression::List(items) => todo!(),

        Expression::Object(entries) => todo!(),

        Expression::Identifier(string) => todo!(),

        Expression::Unary { right, operator } => todo!(),

        Expression::Binary {
            left,
            right,
            operator,
        } => todo!(),

        Expression::Call { function, args } => todo!(),

        Expression::GetIndex { container, index } => todo!(),

        Expression::GetKey { container, key } => todo!(),

        Expression::GetSlice {
            container,
            start,
            end,
        } => todo!(),

        Expression::Error => todo!(),
    };
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert!(true);
    }
}
