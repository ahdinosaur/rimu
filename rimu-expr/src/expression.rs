use std::ops::Range;

use rust_decimal::Decimal;

type Span = Range<usize>;
type Spanned<T> = (T, Span);

use crate::{BinaryOperator, UnaryOperator};

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
        key: Box<SpannedExpression>,
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
