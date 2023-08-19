use rimu_eval::EvalError;

use crate::{EnvironmentError, Value, ValueError};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("unknown block key: {block_key}")]
    UnknownBlockKey { block_key: String },
    #[error("too many block keys")]
    TooManyBlockKeys,
    #[error("value error: {0}")]
    Value(#[from] ValueError),
}

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("rhai eval error: {0}")]
    EvalError(#[from] EvalError),
    #[error("missing context: {var}")]
    MissingEnvironment { var: String },
    #[error("unterminated interpolation: {src}")]
    UnterminatedInterpolation { src: String },
    #[error("'{var}' cannot be interpolated into a string: {value}")]
    InvalidValueInterpolation { var: String, value: Value },
    #[error("context error: {0}")]
    EnvironmentError(#[from] EnvironmentError),
}
