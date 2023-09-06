use rimu_ast::Expression;
use rimu_meta::{ErrorReport, Span};
use rimu_value::{Object, Value};

use crate::EnvironmentError;

#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum EvalError {
    #[error("{source}")]
    Environment {
        span: Span,
        #[source]
        source: EnvironmentError,
    },
    #[error("missing variable: {var}")]
    MissingVariable { span: Span, var: String },
    #[error("tried to call non-function: {expr}")]
    CallNonFunction { span: Span, expr: Expression },
    #[error("type error, expected: {expected}, got: {got}")]
    TypeError {
        span: Span,
        expected: String,
        got: Value,
    },
    #[error("index out of bounds, index: {index}, length: {length}")]
    IndexOutOfBounds {
        container_span: Span,
        index_span: Span,
        index: isize,
        length: usize,
    },
    #[error("key error, key: {key}, object: {object:?}")]
    KeyNotFound {
        key_span: Span,
        key: String,
        object_span: Span,
        object: Object,
    },
    #[error("range start >= end, start: {start}, end: {end}")]
    RangeStartGreaterThanOrEqualToEnd {
        span: Span,
        start: usize,
        end: usize,
    },
    #[error("unterminated interpolation: {src}")]
    UnterminatedInterpolation { span: Span, src: String },
    #[error("cannot be interpolated into a string: {value}")]
    InvalidInterpolationValue { span: Span, value: Value },
    #[error("error expression")]
    ErrorExpression { span: Span },
}

impl From<EvalError> for ErrorReport {
    fn from(value: EvalError) -> Self {
        let (msg, spans, notes): (&str, Vec<(Span, String)>, Vec<String>) = match value {
            EvalError::Environment { span, source } => (
                "Eval: Environment error",
                vec![(span.clone(), format!("{}", source))],
                vec![],
            ),
            EvalError::MissingVariable { span, var } => (
                "Eval: Missing variable",
                vec![(span.clone(), format!("Not in environment: {}", var))],
                vec![],
            ),
            EvalError::CallNonFunction { span, expr } => (
                "Eval: Tried to call non-function",
                vec![(span.clone(), format!("Not a function: {}", expr))],
                vec![],
            ),
            EvalError::TypeError {
                span,
                expected,
                got,
            } => (
                "Eval: Unexpected type",
                vec![(
                    span.clone(),
                    format!("Expected: {}, got: {}", expected, got),
                )],
                vec![],
            ),
            EvalError::IndexOutOfBounds {
                container_span,
                index,
                index_span,
                length,
            } => (
                "Eval: Index out of bounds",
                vec![
                    (container_span.clone(), format!("Length: {}", length)),
                    (index_span.clone(), format!("Index: {}", index)),
                ],
                vec![],
            ),
            EvalError::KeyNotFound {
                key_span,
                key,
                object_span,
                object,
            } => (
                "Eval: Key not found",
                vec![
                    (
                        object_span.clone(),
                        format!("Object: {}", Value::Object(object.clone())),
                    ),
                    (key_span.clone(), format!("Key: {}", key)),
                ],
                vec![],
            ),
            EvalError::RangeStartGreaterThanOrEqualToEnd { span, start, end } => (
                "Eval: Range start >= end",
                vec![(span.clone(), format!("{} >= {}", start, end))],
                vec![],
            ),
            EvalError::UnterminatedInterpolation { span, src } => (
                "Eval: Unterminated interpolation",
                vec![(span.clone(), format!("Source: {}", src))],
                vec![],
            ),
            EvalError::InvalidInterpolationValue { span, value } => (
                "Eval: Cannot be interpolated into a string",
                vec![(
                    span.clone(),
                    format!("Value cannot be interpolated into a string: {}", value),
                )],
                vec![],
            ),
            EvalError::ErrorExpression { span } => (
                "Eval: Expression error",
                vec![(span.clone(), "Error".to_string())],
                vec![],
            ),
        };

        ErrorReport {
            message: msg.into(),
            spans,
            notes,
        }
    }
}
