// with help from
// - https://github.com/DennisPrediger/SLAC/blob/main/src/interpreter.rs

use rimu_env::{Environment, EnvironmentError};
use rimu_expr::{BinaryOperator, Expression, SpannedExpression, UnaryOperator};
use rimu_value::{Number, Object, Value};

#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    #[error("environment error: {0}")]
    Environment(#[source] EnvironmentError),
    #[error("missing context: {var}")]
    MissingEnvironment { var: String },
}

/// A tree walking interpreter which given an [`Environment`] and an [`Expression`]
/// recursivly walks the tree and computes a single [`Value`].
pub struct Evaluator<'a> {
    env: &'a Environment<'a>,
}

impl<'a> Evaluator<'a> {
    pub fn new(env: &'a Environment) -> Self {
        Self { env }
    }

    pub fn evaluate(expression: &SpannedExpression, env: &Environment) -> Result<Value, EvalError> {
        Self::new(env).expression(expression)
    }

    pub fn expression(&self, expr: &SpannedExpression) -> Result<Value, EvalError> {
        let value = match expr.into_inner() {
            Expression::Null => Value::Null,

            Expression::Boolean(boolean) => Value::Boolean(boolean),

            Expression::String(string) => Value::String(string),

            Expression::Number(decimal) => Value::Number(decimal.into()),

            Expression::List(items) => Value::List(
                items
                    .iter()
                    .map(|item| self.expression(item))
                    .collect::<Result<Vec<Value>, EvalError>>()?,
            ),

            Expression::Object(entries) => {
                let mut object = Object::new();
                for (key, value) in entries.into_iter() {
                    let Expression::Identifier(key) = key.into_inner() else {
                    panic!();
                };
                    let value = self.expression(&value)?;
                    object.insert(key, value);
                }
                Value::Object(object)
            }

            Expression::Identifier(var) => self
                .env
                .get(&var)
                .ok_or_else(|| EvalError::MissingEnvironment { var })?
                .clone(),

            Expression::Unary { right, operator } => {
                let right = self.expression(&*right)?;
                match operator {
                    UnaryOperator::Negative => -right,
                    UnaryOperator::Not => !right,
                }
            }

            Expression::Binary {
                left,
                right,
                operator,
            } => {
                let left = self.expression(&*left)?;
                let right = self.expression(&*right)?;
                match operator {
                    BinaryOperator::And => todo!(),
                    BinaryOperator::Or => todo!(),
                    BinaryOperator::Add => left + right,
                    BinaryOperator::Subtract => left - right,
                    BinaryOperator::Multiply => left * right,
                    BinaryOperator::Divide => left / right,
                    BinaryOperator::Div => left.div_int(right),
                    BinaryOperator::Mod => todo!(),
                    BinaryOperator::Xor => todo!(),
                    BinaryOperator::Greater => Value::Boolean(left > right),
                    BinaryOperator::GreaterEqual => Value::Boolean(left >= right),
                    BinaryOperator::Less => Value::Boolean(left < right),
                    BinaryOperator::LessEqual => Value::Boolean(left <= right),
                    BinaryOperator::Equal => Value::Boolean(left == right),
                    BinaryOperator::NotEqual => Value::Boolean(left != right),
                }
            }

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

    pub fn unary(right: SpannedExpression, operator: &Operator) -> Result<Value, EvalError> {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert!(true);
    }
}
