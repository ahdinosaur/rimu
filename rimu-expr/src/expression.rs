use rust_decimal::Decimal;

use rimu_span::Span;

use crate::Operator;

/// An expression represents an entity which can be evaluated to a value.
pub enum ExpressionKind {
    /// Literal null.
    Null,

    /// Literal boolean.
    Boolean(bool),

    /// Literal string.
    String(String),

    /// Literal number.
    Number(Decimal),

    /// Literal list.
    List(Vec<Expression>),

    /// Literal key-value object.
    Object(Vec<(String, Expression)>),

    /// A named identifier.
    Identifier { name: String },

    /// An operation on a single [`Expression`] operand with an [`Operator`]
    Unary {
        right: Box<Expression>,
        operator: Operator,
    },

    /// An operation on two [`Expression`] operands with a an [`Operator`].
    Binary {
        left: Box<Expression>,
        right: Box<Expression>,
        operator: Operator,
    },

    /// A function invocation with a list of [`Expression`] parameters.
    Call {
        function: Box<Expression>,
        params: Vec<Expression>,
    },

    /// Index operation (`a[x]`).
    Index {
        container: Box<Expression>,
        index: Box<Expression>,
    },

    /// Slice operation (`b[x:y]`).
    Slice {
        container: Box<Expression>,
        start: Option<Box<Expression>>,
        end: Option<Box<Expression>>,
    },

    /// Dot operation (`c.z`).
    Dot {
        container: Box<Expression>,
        key: String,
    },
}

/// An expression node.
pub struct Expression {
    kind: ExpressionKind,
    span: Span,
}
