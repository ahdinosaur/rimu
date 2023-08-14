// Inspired by:
//
// - https://docs.rs/miette/latest/miette/struct.Span.html
// - https://docs.rs/codespan/latest/codespan/struct.Span.html
// - https://github.com/noir-lang/noir/blob/master/crates/noirc_errors/src/position.rs

use std::{error::Error, fmt::Display, ops::Range};

/// A span of source code.
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
pub struct Span {
    /// The start byte offset of the span.
    start: usize,

    /// The end byte offset of the span
    end: usize,
}

impl Span {
    /// Create a new [`Span`].
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// The absolute offset, in bytes, to the beginning of a [`Span`].
    pub const fn start(&self) -> usize {
        self.start
    }

    /// The absolute offset, in bytes, to the end of a [`Span`].
    pub const fn end(&self) -> usize {
        self.end
    }

    /// Total length of the [`Span`], in bytes.
    pub const fn len(&self) -> usize {
        self.end - self.start
    }

    /// Whether this [`Span`] has a length of zero. It may still be useful
    /// to point to a specific point.
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl From<(usize, usize)> for Span {
    fn from((start, len): (usize, usize)) -> Self {
        Self::new(start, len)
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Self {
            start: range.start.into(),
            end: range.end.into(),
        }
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.start, self.end)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Spanned<T> {
    span: Span,
    contents: T,
}

impl<T: Clone> Spanned<T> {
    pub fn from<S: Into<Span>>(span: S, contents: T) -> Spanned<T> {
        Spanned {
            span: span.into(),
            contents,
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn into_contents(&self) -> T {
        self.contents.clone()
    }
}

#[derive(thiserror::Error, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[error("error at span {span}: {error}")]
pub struct SpannedError<E: Error> {
    pub span: Span,
    pub error: E,
}

impl<E: Clone> SpannedError<E>
where
    E: Error,
{
    pub const fn from(span: Span, error: E) -> SpannedError<E> {
        SpannedError { span, error }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn into_error(&self) -> E {
        self.error.clone()
    }
}
