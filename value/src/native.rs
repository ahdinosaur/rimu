use rimu_meta::Spanned;

use crate::{EvalError, Value};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct NativeFunction {
    function: fn(&[Spanned<Value>]) -> Result<Value, EvalError>,
}

impl NativeFunction {
    pub fn new(function: fn(&[Spanned<Value>]) -> Result<Value, EvalError>) -> Self {
        Self { function }
    }

    pub fn call(&self, args: &[Spanned<Value>]) -> Result<Value, EvalError> {
        (self.function)(args)
    }
}
