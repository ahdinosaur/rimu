use core::fmt;

use rust_decimal::Decimal;

use crate::{BinaryOperator, Spanned, UnaryOperator};

/// An expression represents an entity which can be evaluated to a value.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Expression<'src> {
    /// Literal null.
    Null,

    /// Literal boolean.
    Boolean(bool),

    /// Literal string.
    String(&'src str),

    /// Literal number.
    Number(Decimal),

    /// Literal list.
    List(Vec<SpannedExpression<'src>>),

    /// Literal key-value object.
    Object(Vec<(Spanned<&'src str>, SpannedExpression<'src>)>),

    /// A named local variable.
    Identifier(&'src str),

    /// An operation on a single [`Expression`] operand with an [`Operator`]
    Unary {
        right: Box<SpannedExpression<'src>>,
        operator: UnaryOperator,
    },

    /// An operation on two [`Expression`] operands with a an [`Operator`].
    Binary {
        left: Box<SpannedExpression<'src>>,
        right: Box<SpannedExpression<'src>>,
        operator: BinaryOperator,
    },

    /// A function invocation with a list of [`Expression`] parameters.
    Call {
        function: Box<SpannedExpression<'src>>,
        args: Vec<SpannedExpression<'src>>,
    },

    /// Get index operation (`a[x]`).
    GetIndex {
        container: Box<SpannedExpression<'src>>,
        index: Box<SpannedExpression<'src>>,
    },

    /// Get key operation (`c.z`).
    GetKey {
        container: Box<SpannedExpression<'src>>,
        key: Spanned<&'src str>,
    },

    /// Slice operation (`b[x:y]`).
    GetSlice {
        container: Box<SpannedExpression<'src>>,
        start: Option<Box<SpannedExpression<'src>>>,
        end: Option<Box<SpannedExpression<'src>>>,
    },

    Error,
}

pub type SpannedExpression<'src> = Spanned<Expression<'src>>;

impl<'src> fmt::Display for Expression<'src> {
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
