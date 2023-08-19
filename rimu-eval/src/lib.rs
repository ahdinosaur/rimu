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
    #[error("tried to call non-function")]
    CallNonFunction, // { expr: SpannedExpression },
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

    pub fn evaluate(
        expression: &SpannedExpression,
        env: &'a Environment<'a>,
    ) -> Result<Value, EvalError> {
        Self::new(env).expression(expression)
    }

    pub fn expression(&self, expr: &SpannedExpression) -> Result<Value, EvalError> {
        let _span = expr.span();
        let value = match expr.inner() {
            Expression::Null => Value::Null,

            Expression::Boolean(boolean) => Value::Boolean(*boolean),

            Expression::String(string) => Value::String(string.clone()),

            Expression::Number(decimal) => Value::Number(Into::<Number>::into(*decimal)),

            Expression::List(items) => self.list(&items)?,

            Expression::Object(entries) => self.object(&entries)?,

            Expression::Identifier(var) => self.variable(var)?,

            Expression::Unary { right, operator } => self.unary(right.as_ref(), &operator)?,

            Expression::Binary {
                left,
                right,
                operator,
            } => self.binary(&left, &operator, &right)?,

            Expression::Call { function, args } => self.call(function.as_ref(), &args)?,

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

    pub fn unary(
        &self,
        right: &SpannedExpression,
        operator: &UnaryOperator,
    ) -> Result<Value, EvalError> {
        let right = self.expression(right)?;
        let value = match operator {
            UnaryOperator::Negative => -right,
            UnaryOperator::Not => !right,
        };
        Ok(value)
    }

    pub fn binary(
        &self,
        left: &SpannedExpression,
        operator: &BinaryOperator,
        right: &SpannedExpression,
    ) -> Result<Value, EvalError> {
        let left = self.expression(&*left)?;
        let value = match operator {
            BinaryOperator::And => self.boolean(left, right, true)?,
            BinaryOperator::Or => self.boolean(left, right, false)?,
            _ => {
                let right = self.expression(&*right)?;
                match operator {
                    BinaryOperator::Or => unreachable!(),
                    BinaryOperator::And => unreachable!(),
                    BinaryOperator::Add => left + right,
                    BinaryOperator::Subtract => left - right,
                    BinaryOperator::Multiply => left * right,
                    BinaryOperator::Divide => left / right,
                    BinaryOperator::Mod => left % right,
                    BinaryOperator::Xor => left ^ right,
                    BinaryOperator::Greater => Value::Boolean(left > right),
                    BinaryOperator::GreaterEqual => Value::Boolean(left >= right),
                    BinaryOperator::Less => Value::Boolean(left < right),
                    BinaryOperator::LessEqual => Value::Boolean(left <= right),
                    BinaryOperator::Equal => Value::Boolean(left == right),
                    BinaryOperator::NotEqual => Value::Boolean(left != right),
                }
            }
        };
        Ok(value)
    }

    fn boolean(
        &self,
        left: Value,
        right: &SpannedExpression,
        full_evaluate_on: bool,
    ) -> Result<Value, EvalError> {
        let value = match left {
            Value::Boolean(left) => {
                if left == full_evaluate_on {
                    // if `left` is not the result we need, evaluate `right`
                    match self.expression(right)? {
                        Value::Boolean(right) => Value::Boolean(right),
                        _ => Value::Null,
                    }
                } else {
                    Value::Boolean(left) // short circuit
                }
            }
            _ => Value::Null,
        };
        Ok(value)
    }

    fn list(&self, items: &Vec<SpannedExpression>) -> Result<Value, EvalError> {
        Ok(Value::List(
            items
                .iter()
                .map(|item| self.expression(item))
                .collect::<Result<Vec<Value>, EvalError>>()?,
        ))
    }

    fn object(
        &self,
        entries: &Vec<(SpannedExpression, SpannedExpression)>,
    ) -> Result<Value, EvalError> {
        let mut object = Object::new();
        for (key, value) in entries.into_iter() {
            let Expression::Identifier(key) = key.inner() else {
                    panic!();
                };
            let key = key.clone();
            let value = self.expression(&value)?;
            object.insert(key, value);
        }
        Ok(Value::Object(object))
    }

    fn variable(&self, var: &str) -> Result<Value, EvalError> {
        self.env
            .get(&var)
            .map(Clone::clone)
            // .map_or(Value::Null, |v| (*v).clone())
            .ok_or_else(|| EvalError::MissingEnvironment {
                var: var.to_string(),
            })
    }

    fn call(
        &self,
        function: &SpannedExpression,
        args: &[SpannedExpression],
    ) -> Result<Value, EvalError> {
        let Expression::Identifier(var) = function.inner() else {
            return Err(EvalError::CallNonFunction)
        };
        let Value::Function(function) = self.variable(var)? else {
            return Err(EvalError::CallNonFunction);
        };

        let args: Vec<Value> = args
            .iter()
            .map(|expression| self.expression(expression))
            .collect::<Result<Vec<Value>, EvalError>>()?;

        let mut function_env = self.env.child();

        for index in 0..function.args.len() {
            let arg_name = function.args[index].clone();
            let arg_value = args
                .get(index)
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| Value::Null);
            function_env.insert(arg_name, arg_value);
        }

        Evaluator::evaluate(&function.body, &function_env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert!(true);
    }
}
