// with help from
// - https://github.com/DennisPrediger/SLAC/blob/main/src/interpreter.rs

use std::{cell::RefCell, ops::Deref, rc::Rc};

use rimu_ast::{Block, Expression, SpannedBlock, SpannedExpression};
use rimu_meta::{Span, Spanned};
use rimu_value::{
    convert_value_object_to_serde_value_object, Environment, Function, FunctionBody, SpannedValue,
    Value, ValueList, ValueObject,
};

use crate::{common, expression::evaluate as evaluate_expression, EvalError, Result};

pub fn evaluate(expression: &SpannedBlock, env: Rc<RefCell<Environment>>) -> Result<SpannedValue> {
    Evaluator::new(env).block(expression)
}

/// A tree walking interpreter which given an [`Environment`] and an [`Block`]
/// recursivly walks the tree and computes a single [`Value`].
struct Evaluator {
    env: Rc<RefCell<Environment>>,
}

impl Evaluator {
    fn new(env: Rc<RefCell<Environment>>) -> Self {
        Self { env }
    }

    fn block(&self, block: &SpannedBlock) -> Result<SpannedValue> {
        let span = block.span();
        match block.inner() {
            Block::Expression(expr) => self.expression(span, expr),
            Block::Object(object) => self.object(span, object),
            Block::List(list) => self.list(span, list),
            Block::Function { args, body } => self.function(span, args, body),
            Block::Call { function, args } => self.call(span, function, args),
            Block::If {
                condition,
                consequent,
                alternative,
            } => self.if_(
                span,
                condition,
                consequent.as_ref().map(|c| c.deref()),
                alternative.as_ref().map(|a| a.deref()),
            ),
            Block::Let { variables, body } => self.let_(span, variables, body.deref()),
        }
    }

    fn expression(&self, span: Span, expr: &Expression) -> Result<SpannedValue> {
        evaluate_expression(&Spanned::new(expr.clone(), span), self.env.clone())
    }

    fn object(
        &self,
        span: Span,
        entries: &[(Spanned<String>, SpannedBlock)],
    ) -> Result<SpannedValue> {
        let mut object = ValueObject::new();
        for (key, value) in entries.iter() {
            let key = key.clone().into_inner();
            let value = self.block(value)?;
            if value.inner() == &Value::Null {
                continue;
            };
            object.insert(key, value);
        }
        let value = Value::Object(object);
        Ok(Spanned::new(value, span))
    }

    fn list(&self, span: Span, items: &[SpannedBlock]) -> Result<SpannedValue> {
        let mut list = ValueList::with_capacity(items.len());
        for item in items.iter() {
            let item = self.block(item)?;
            if item.inner() == &Value::Null {
                continue;
            };
            list.push(item);
        }
        let value = Value::List(list);
        Ok(Spanned::new(value, span))
    }

    fn function(
        &self,
        span: Span,
        args: &[Spanned<String>],
        body: &SpannedBlock,
    ) -> Result<SpannedValue> {
        let args: Vec<String> = args.iter().map(|a| a.inner()).cloned().collect();
        let body = FunctionBody::Block(body.clone());
        let env = self.env.clone();
        let value = Value::Function(Function { args, body, env });
        Ok(Spanned::new(value, span))
    }

    fn call(
        &self,
        span: Span,
        function: &SpannedExpression,
        args: &SpannedBlock,
    ) -> Result<SpannedValue> {
        let Value::Function(function) =
            evaluate_expression(function, self.env.clone())?.into_inner()
        else {
            return Err(EvalError::CallNonFunction {
                span: function.span(),
                expr: function.clone().into_inner(),
            });
        };

        let arg = self.block(args)?;

        let args = match arg.inner() {
            Value::List(list) => list.clone(),
            _ => vec![arg],
        };

        common::call(span, function, &args)
    }

    fn if_(
        &self,
        span: Span,
        condition: &SpannedBlock,
        consequent: Option<&SpannedBlock>,
        alternative: Option<&SpannedBlock>,
    ) -> Result<SpannedValue> {
        let condition = self.block(condition)?.into_inner();

        let value = if Into::<bool>::into(condition) {
            if let Some(consequent) = &consequent {
                self.block(consequent)?.into_inner()
            } else {
                Value::Null
            }
        } else {
            #[allow(clippy::collapsible_else_if)]
            if let Some(alternative) = &alternative {
                self.block(alternative)?.into_inner()
            } else {
                Value::Null
            }
        };

        Ok(Spanned::new(value, span))
    }

    fn let_(
        &self,
        span: Span,
        entries: &[(Spanned<String>, SpannedBlock)],
        body: &SpannedBlock,
    ) -> Result<SpannedValue> {
        let mut variables = ValueObject::new();
        for (key, value) in entries.iter() {
            let key = key.clone().into_inner();
            let value = self.block(value)?;
            if value.inner() == &Value::Null {
                continue;
            };
            variables.insert(key, value);
        }

        let parent_env = self.env.clone();
        let variables = convert_value_object_to_serde_value_object(variables);
        let let_env = Environment::from_object(&variables, Some(parent_env)).map_err(|error| {
            EvalError::Environment {
                span: span.clone(),
                source: Box::new(error),
            }
        })?;
        let let_env = Rc::new(RefCell::new(let_env));

        let value = evaluate(body, let_env)?.into_inner();

        Ok(Spanned::new(value, span))
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use indexmap::IndexMap;

    use indexmap::indexmap;
    use pretty_assertions::assert_eq;
    use rimu_ast::SpannedBlock;
    use rimu_meta::SourceId;
    use rimu_parse::parse_block;
    use rimu_value::SerdeValue;
    use rimu_value::{Environment, Value};
    use rust_decimal_macros::dec;

    use super::{evaluate, EvalError};

    fn test_block(
        expr: SpannedBlock,
        env_object: Option<IndexMap<String, SerdeValue>>,
    ) -> Result<Value, EvalError> {
        let mut env = Environment::new();
        if let Some(env_object) = env_object {
            for (key, value) in env_object.into_iter() {
                env.insert(key, value);
            }
        }
        let env = Rc::new(RefCell::new(env));

        let value = evaluate(&expr, env)?.into_inner();
        Ok(value)
    }

    fn test_code(
        code: &str,
        env_object: Option<IndexMap<String, SerdeValue>>,
    ) -> Result<SerdeValue, EvalError> {
        let (Some(expr), errors) = parse_block(code, SourceId::empty()) else {
            panic!()
        };
        assert_eq!(errors.len(), 0);
        Ok(test_block(expr, env_object)?.into())
    }

    #[test]
    fn op_if() {
        let code = "
zero:
  if ten > five
  then five
  else ten
";

        let env = indexmap! {
            "five".into() => SerdeValue::Number(dec!(5).into()),
            "ten".into() => SerdeValue::Number(dec!(10).into()),
        };
        let actual = test_code(code, Some(env));

        let expected = Ok(SerdeValue::Object(indexmap! {
            "zero".into() => SerdeValue::Number(dec!(5).into())
        }));

        assert_eq!(expected, actual);
    }

    #[test]
    fn op_let() {
        let code = "
zero:
  let
    one: ten
    two: 2
  in
    three: one + two
";

        let env = indexmap! {
            "ten".into() => SerdeValue::Number(dec!(10).into()),
        };
        let actual = test_code(code, Some(env));

        let expected = Ok(SerdeValue::Object(indexmap! {
            "zero".into() => SerdeValue::Object(indexmap! {
                "three".into() => SerdeValue::Number(dec!(12).into())
            })
        }));

        assert_eq!(expected, actual);
    }
}
