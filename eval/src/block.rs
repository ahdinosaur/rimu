// with help from
// - https://github.com/DennisPrediger/SLAC/blob/main/src/interpreter.rs

use std::{cell::RefCell, ops::Deref, rc::Rc};

use rimu_ast::{Block, Expression, SpannedBlock, SpannedExpression};
use rimu_meta::{Span, Spanned};
use rimu_value::{Environment, Function, FunctionBody, List, Object, SpannedValue, Value};

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

    fn object(&self, _span: Span, entries: &[(Spanned<String>, SpannedBlock)]) -> Result<Value> {
        let mut object = Object::new();
        for (key, value) in entries.iter() {
            let key = key.clone().into_inner();
            let (value, _value_span) = self.block(value)?.take();
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
            let (item, _item_span) = self.block(item)?.take();
            if item == Value::Null {
                continue;
            };
            list.push(item);
        }
        Ok(Value::List(list))
    }

    fn function(
        &self,
        _span: Span,
        args: &[Spanned<String>],
        body: &SpannedBlock,
    ) -> Result<Value> {
        let args: Vec<String> = args.iter().map(|a| a.inner()).cloned().collect();
        let body = FunctionBody::Block(body.clone());
        let env = self.env.clone();
        Ok(Value::Function(Function { args, body, env }))
    }

    fn call(&self, span: Span, function: &SpannedExpression, args: &SpannedBlock) -> Result<Value> {
        let Value::Function(function) =
            evaluate_expression(function, self.env.clone())?.into_inner()
        else {
            return Err(EvalError::CallNonFunction {
                span: function.span(),
                expr: function.clone().into_inner(),
            });
        };

        let (arg, arg_span) = self.block(args)?.take();

        // TODO if arg is a list, then becomes args
        // - HOWEVER, we need span info, so we need eval to return spans for this to work.

        let args = vec![Spanned::new(arg, arg_span)];
        common::call(span, function, &args)
    }

    fn if_(
        &self,
        _span: Span,
        condition: &SpannedBlock,
        consequent: Option<&SpannedBlock>,
        alternative: Option<&SpannedBlock>,
    ) -> Result<Value> {
        let (condition, _condition_span) = self.block(condition)?.take();

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
        Ok(value)
    }

    fn let_(
        &self,
        span: Span,
        entries: &[(Spanned<String>, SpannedBlock)],
        body: &SpannedBlock,
    ) -> Result<Value> {
        let mut variables = Object::new();
        for (key, value) in entries.iter() {
            let key = key.clone().into_inner();
            let (value, _value_span) = self.block(value)?.take();
            if value == Value::Null {
                continue;
            };
            variables.insert(key, value);
        }

        let parent_env = self.env.clone();
        let let_env = Environment::from_object(&variables, Some(parent_env)).map_err(|error| {
            EvalError::Environment {
                span,
                source: error,
            }
        })?;
        let let_env = Rc::new(RefCell::new(let_env));

        let value = evaluate(body, let_env)?.into_inner();
        Ok(value)
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
    use rimu_value::{Environment, Value};
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
        let env = Rc::new(RefCell::new(env));

        let value = evaluate(&expr, env)?.into_inner();
        Ok(value)
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
  if ten > five
  then five
  else ten
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
  let
    one: ten
    two: 2
  in
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
