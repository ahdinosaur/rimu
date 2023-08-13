// Inspired by:
//
// - https://docs.rs/miette/latest/miette/struct.Span.html
// - https://docs.rs/codespan/latest/codespan/struct.Span.html
// - https://github.com/noir-lang/noir/blob/master/crates/noirc_errors/src/position.rs

use std::ops::Range;

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

impl From<std::ops::Range<usize>> for Span {
    fn from(range: std::ops::Range<usize>) -> Self {
        Self {
            start: range.start.into(),
            end: range.end.into(),
        }
    }
}

impl chumsky::Span for Span {
    type Context = ();

    type Offset = usize;

    fn new(_context: Self::Context, range: Range<Self::Offset>) -> Self {
        range.into()
    }

    fn context(&self) -> Self::Context {}

    fn start(&self) -> Self::Offset {
        self.start
    }

    fn end(&self) -> Self::Offset {
        self.end
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, Ord, Eq, Hash, Default)]
pub struct Spanned<T> {
    pub contents: T,
    span: Span,
}

impl<T> Spanned<T> {
    pub const fn from(span: Span, contents: T) -> Spanned<T> {
        Spanned { span, contents }
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

/// This is important for tests. Two Spanned objects are equal if their content is equal
/// They may not have the same span. Use `.span()` to test for Span being equal specifically
impl<T: std::cmp::PartialEq> PartialEq<Spanned<T>> for Spanned<T> {
    fn eq(&self, other: &Spanned<T>) -> bool {
        self.contents == other.contents
    }
}
