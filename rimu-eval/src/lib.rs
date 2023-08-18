mod error;

pub use rimu_expr::Expression;
use rimu_value::Value;

pub use self::error::EvalError;

pub fn evaluate(expr: Expression) -> Result<Value, EvalError> {
    Ok(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert!(true);
    }
}
