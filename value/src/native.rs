use rimu_meta::{Span, Spanned};

use crate::{EvalError, Value};

type Args = [Spanned<Value>];

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct NativeFunction {
    function: fn(Span, &Args) -> Result<Spanned<Value>, EvalError>,
}

impl NativeFunction {
    pub fn new(function: fn(Span, &Args) -> Result<Spanned<Value>, EvalError>) -> Self {
        Self { function }
    }

    pub fn call(&self, span: Span, args: &Args) -> Result<Spanned<Value>, EvalError> {
        (self.function)(span, args)
    }
}
