use rimu_meta::Spanned;
use rust_decimal::Decimal;
use std::fmt;

use crate::{BinaryOperator, UnaryOperator};

/// An expression represents an entity which can be evaluated to a value.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Expression {
    /// Literal null.
    Null,

    /// Literal boolean.
    Boolean(bool),

    /// Literal string.
    String(String),

    /// Literal number.
    Number(Decimal),

    /// Literal list.
    List(Vec<SpannedExpression>),

    /// Literal key-value object.
    Object(Vec<(Spanned<String>, SpannedExpression)>),

    /// A named local variable.
    Identifier(String),

    /// An operation on a single [`Expression`] operand with an [`Operator`]
    Unary {
        right: Box<SpannedExpression>,
        operator: UnaryOperator,
    },

    /// An operation on two [`Expression`] operands with a an [`Operator`].
    Binary {
        left: Box<SpannedExpression>,
        right: Box<SpannedExpression>,
        operator: BinaryOperator,
    },

    /// A function invocation with a list of [`Expression`] parameters.
    Call {
        function: Box<SpannedExpression>,
        args: Vec<SpannedExpression>,
    },

    /// Get index operation (`a[x]`).
    GetIndex {
        container: Box<SpannedExpression>,
        index: Box<SpannedExpression>,
    },

    /// Get key operation (`c.z`).
    GetKey {
        container: Box<SpannedExpression>,
        key: Spanned<String>,
    },

    /// Slice operation (`b[x:y]`).
    GetSlice {
        container: Box<SpannedExpression>,
        start: Option<Box<SpannedExpression>>,
        end: Option<Box<SpannedExpression>>,
    },

    Error,
}

pub type SpannedExpression = Spanned<Expression>;

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Null => write!(f, "null"),
            Expression::Boolean(boolean) => write!(f, "{}", boolean),
            Expression::String(string) => write!(f, "\"{}\"", string),
            Expression::Number(number) => write!(f, "{}", number),
            Expression::List(list) => {
                let keys = list
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "[{}]", keys)
            }
            Expression::Object(object) => {
                let entries = object
                    .iter()
                    .map(|(key, value)| format!("\"{}\": {}", key, value.to_string()))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "{{{}}}", entries)
            }
            Expression::Identifier(identifier) => write!(f, "{}", identifier),
            Expression::Unary { right, operator } => write!(f, "{}{}", operator, right),
            Expression::Binary {
                left,
                operator,
                right,
            } => write!(f, "{} {} {}", left, operator, right),
            Expression::Call { function, args } => {
                let args = args
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "{}({})", function, args)
            }
            Expression::GetIndex { container, index } => write!(f, "{}[{}]", container, index),
            Expression::GetKey { container, key } => write!(f, "{}.{}", container, key),
            Expression::GetSlice {
                container,
                start,
                end,
            } => write!(
                f,
                "{}[{}:{}]",
                container,
                start.as_ref().map(|s| s.to_string()).unwrap_or("".into()),
                end.as_ref().map(|e| e.to_string()).unwrap_or("".into()),
            ),
            Expression::Error => write!(f, "error"),
        }
    }
}
