// with help from
// - https://github.com/DennisPrediger/SLAC/blob/main/src/interpreter.rs

use crate::Environment;
use rimu_ast::{Block, BlockOperation, Expression, SpannedBlock};
use rimu_meta::{Span, Spanned};
use rimu_value::{List, Object, Value};

use crate::{expression::evaluate as evaluate_expression, EvalError};

type Result<Value> = std::result::Result<Value, EvalError>;

pub fn evaluate<'a>(expression: &SpannedBlock, env: &'a Environment<'a>) -> Result<Value> {
    let (value, _span) = Evaluator::new(env).block(expression)?;
    Ok(value)
}

/// A tree walking interpreter which given an [`Environment`] and an [`Block`]
/// recursivly walks the tree and computes a single [`Value`].
struct Evaluator<'a> {
    env: &'a Environment<'a>,
}

impl<'a> Evaluator<'a> {
    fn new(env: &'a Environment) -> Self {
        Self { env }
    }

    fn block(&self, block: &SpannedBlock) -> Result<(Value, Span)> {
        let span = block.span();
        let span_ret = span.clone();

        let value = match block.inner() {
            Block::Object(object) => self.object(span, object)?,
            Block::List(list) => self.list(span, list)?,
            Block::Expression(expr) => self.expression(span, expr)?,
            Block::Operation(op) => self.operation(span, op)?,
        };

        Ok((value, span_ret))
    }

    fn object(&self, _span: Span, entries: &[(Spanned<String>, SpannedBlock)]) -> Result<Value> {
        let mut object = Object::new();
        for (key, value) in entries.iter() {
            let key = key.clone().into_inner();
            let (value, _value_span) = self.block(value)?;
            if value == Value::Null {
                continue;
            };
            object.insert(key, value);
        }
        Ok(Value::Object(object))
    }

    fn list(&self, _span: Span, items: &[SpannedBlock]) -> Result<Value> {
        let mut list = List::with_capacity(items.len());
        for item in items.iter() {
            let (item, _item_span) = self.block(item)?;
            if item == Value::Null {
                continue;
            };
            list.push(item);
        }
        Ok(Value::List(list))
    }

    fn expression(&self, span: Span, expr: &Expression) -> Result<Value> {
        evaluate_expression(&Spanned::new(expr.clone(), span), self.env)
    }

    fn operation(&self, span: Span, op: &BlockOperation) -> Result<Value> {
        let value = match op {
            BlockOperation::If {
                condition,
                consequent,
                alternative,
            } => {
                let (value, _value_span) = self.block(condition)?;

                if Into::<bool>::into(value) {
                    if let Some(consequent) = &consequent {
                        self.block(consequent)?.0
                    } else {
                        Value::Null
                    }
                } else {
                    #[allow(clippy::collapsible_else_if)]
                    if let Some(alternative) = &alternative {
                        self.block(alternative)?.0
                    } else {
                        Value::Null
                    }
                }
            }
            BlockOperation::Let { variables, body } => {
                let (variables, _variables_span) = self.block(variables)?;

                let env = Environment::from_value(&variables, Some(self.env)).map_err(|error| {
                    EvalError::Environment {
                        span,
                        source: error,
                    }
                })?;

                evaluate(body, &env)?
            }
        };
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use crate::Environment;
    use indexmap::indexmap;
    use pretty_assertions::assert_eq;
    use rimu_ast::SpannedBlock;
    use rimu_meta::SourceId;
    use rimu_parse::parse_block;
    use rimu_value::Value;
    use rust_decimal_macros::dec;

    use super::{evaluate, EvalError};

    fn test_block(
        expr: SpannedBlock,
        env_object: Option<IndexMap<String, Value>>,
    ) -> Result<Value, EvalError> {
        let mut env = Environment::new();
        if let Some(env_object) = env_object {
            for (key, value) in env_object.into_iter() {
                env.insert(key, value);
            }
        }

        evaluate(&expr, &env)
    }

    fn test_code(
        code: &str,
        env_object: Option<IndexMap<String, Value>>,
    ) -> Result<Value, EvalError> {
        let (Some(expr), errors) = parse_block(code, SourceId::empty()) else {
            panic!()
        };
        assert_eq!(errors.len(), 0);
        test_block(expr, env_object)
    }

    #[test]
    fn op_if() {
        let code = "
zero:
  $if: ten > five
  then: five
  else: ten
";

        let env = indexmap! {
            "five".into() => Value::Number(dec!(5).into()),
            "ten".into() => Value::Number(dec!(10).into()),
        };
        let actual = test_code(code, Some(env));

        let expected = Ok(Value::Object(indexmap! {
            "zero".into() => Value::Number(dec!(5).into())
        }));

        assert_eq!(expected, actual);
    }

    #[test]
    fn op_let() {
        let code = "
zero:
  $let:
    one: ten
    two: 2
  in:
    three: one + two
";

        let env = indexmap! {
            "ten".into() => Value::Number(dec!(10).into()),
        };
        let actual = test_code(code, Some(env));

        let expected = Ok(Value::Object(indexmap! {
            "zero".into() => Value::Object(indexmap! {
                "three".into() => Value::Number(dec!(12).into())
            })
        }));

        assert_eq!(expected, actual);
    }
}
