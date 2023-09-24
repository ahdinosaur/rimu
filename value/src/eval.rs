use rimu_ast::Expression;
use rimu_meta::{ErrorReport, Span};

use crate::{EnvironmentError, Object, Value};

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
    #[error("missing argument: {index}")]
    MissingArgument { span: Span, index: usize },
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
        let (span, msg, labels, notes): (Span, &str, Vec<(Span, String)>, Vec<String>) = match value
        {
            EvalError::Environment { span, source } => (
                span.clone(),
                "Eval: Environment error",
                vec![(span.clone(), format!("{}", source))],
                vec![],
            ),
            EvalError::MissingVariable { span, var } => (
                span.clone(),
                "Eval: Missing variable",
                vec![(span.clone(), format!("Not in environment: {}", var))],
                vec![],
            ),
            EvalError::CallNonFunction { span, expr } => (
                span.clone(),
                "Eval: Tried to call non-function",
                vec![(span.clone(), format!("Not a function: {}", expr))],
                vec![],
            ),
            EvalError::MissingArgument { span, index } => (
                span.clone(),
                "Eval: Tried to call function without required argument",
                vec![(span.clone(), format!("Argument index: {}", index))],
                vec![],
            ),
            EvalError::TypeError {
                span,
                expected,
                got,
            } => (
                span.clone(),
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
                container_span.clone().union(index_span.clone()),
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
                key_span.clone().union(object_span.clone()),
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
                span.clone(),
                "Eval: Range start >= end",
                vec![(span.clone(), format!("{} >= {}", start, end))],
                vec![],
            ),
            EvalError::UnterminatedInterpolation { span, src } => (
                span.clone(),
                "Eval: Unterminated interpolation",
                vec![(span.clone(), format!("Source: {}", src))],
                vec![],
            ),
            EvalError::InvalidInterpolationValue { span, value } => (
                span.clone(),
                "Eval: Cannot be interpolated into a string",
                vec![(
                    span.clone(),
                    format!("Value cannot be interpolated into a string: {}", value),
                )],
                vec![],
            ),
            EvalError::ErrorExpression { span } => (
                span.clone(),
                "Eval: Expression error",
                vec![(span.clone(), "Error".to_string())],
                vec![],
            ),
        };

        ErrorReport {
            span,
            message: msg.into(),
            labels,
            notes,
        }
    }
}
