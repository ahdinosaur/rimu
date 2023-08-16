use std::ops::Range;

use rust_decimal::Decimal;

type Span = Range<usize>;
type Spanned<T> = (T, Span);

use crate::Operator;

/// An expression represents an entity which can be evaluated to a value.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
    Object(Vec<(SpannedExpression, SpannedExpression)>),

    /// A named local variable.
    Identifier(String),

    /// An operation on a single [`Expression`] operand with an [`Operator`]
    Unary {
        right: Box<SpannedExpression>,
        operator: Operator,
    },

    /// An operation on two [`Expression`] operands with a an [`Operator`].
    Binary {
        left: Box<SpannedExpression>,
        right: Box<SpannedExpression>,
        operator: Operator,
    },

    /// A function invocation with a list of [`Expression`] parameters.
    Call {
        function: Box<SpannedExpression>,
        args: Vec<SpannedExpression>,
    },

    /// Index operation (`a[x]`).
    Index {
        container: Box<SpannedExpression>,
        index: Box<SpannedExpression>,
    },

    /// Slice operation (`b[x:y]`).
    Slice {
        container: Box<SpannedExpression>,
        start: Option<Box<SpannedExpression>>,
        end: Option<Box<SpannedExpression>>,
    },

    /// Dot operation (`c.z`).
    Dot {
        container: Box<SpannedExpression>,
        key: String,
    },

    Error,
}

pub type SpannedExpression = Spanned<Expression>;
