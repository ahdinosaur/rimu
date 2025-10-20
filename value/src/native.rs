use rimu_meta::{Span, Spanned};

use crate::{EvalError, Value};

type Args = [Spanned<Value>];

#[derive(Debug, Clone)]
pub struct NativeFunction {
    name: &'static str,
    function: fn(Span, &Args) -> Result<Spanned<Value>, EvalError>,
}

impl NativeFunction {
    pub fn new(
        name: &'static str,
        function: fn(Span, &Args) -> Result<Spanned<Value>, EvalError>,
    ) -> Self {
        Self { name, function }
    }

    pub fn call(&self, span: Span, args: &Args) -> Result<Spanned<Value>, EvalError> {
        (self.function)(span, args)
    }
}

impl PartialEq for NativeFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for NativeFunction {}

impl PartialOrd for NativeFunction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(other.name)
    }
}
