use crate::Value;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub struct NativeFunction {
    function: fn(&[Value]) -> Result<Value, NativeFunctionError>,
}

impl NativeFunction {
    pub fn new(function: fn(&[Value]) -> Result<Value, NativeFunctionError>) -> Self {
        Self { function }
    }

    pub fn call(&self, args: &[Value]) -> Result<Value, NativeFunctionError> {
        (self.function)(args)
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum NativeFunctionError {
    ArgTypeError {
        index: usize,
        expected: String,
        got: Box<Value>,
    },
}
