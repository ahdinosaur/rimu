use std::{
    fmt::{self, Display},
    ops::Range,
};

use crate::SourceId;

/// A span of source code.
#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
pub struct Span {
    source: SourceId,

    /// The start byte offset of the span.
    start: usize,

    /// The end byte offset of the span
    end: usize,
}

impl Span {
    /// Create a new [`Span`].
    pub const fn new(source: SourceId, start: usize, end: usize) -> Self {
        Self { source, start, end }
    }

    pub fn source(&self) -> SourceId {
        self.source.clone()
    }

    /// The absolute offset, in bytes, to the beginning of a [`Span`].
    pub const fn start(&self) -> usize {
        self.start
    }

    /// The absolute offset, in bytes, to the end of a [`Span`].
    pub const fn end(&self) -> usize {
        self.end
    }

    pub const fn range(&self) -> Range<usize> {
        self.start..self.end
    }

    pub fn union(self, other: Self) -> Self {
        Self {
            start: self.start().min(other.start()),
            end: self.end().max(other.end()),
            ..self
        }
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

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}:{:?}", self.source, self.range())
    }
}

impl chumsky::span::Span for Span {
    type Context = SourceId;
    type Offset = usize;

    fn new(context: SourceId, range: Range<usize>) -> Self {
        assert!(range.start <= range.end);
        Self {
            source: context,
            start: range.start,
            end: range.end,
        }
    }

    fn context(&self) -> SourceId {
        self.source.clone()
    }

    fn start(&self) -> Self::Offset {
        self.start
    }

    fn end(&self) -> Self::Offset {
        self.end
    }
}

impl ariadne::Span for Span {
    type SourceId = SourceId;

    fn source(&self) -> &SourceId {
        &self.source
    }

    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
pub struct Spanned<T> {
    inner: T,
    span: Span,
}

impl<T> Spanned<T>
where
    T: Clone,
{
    /// Create a new [`Spanned`] with the given inner value and span.
    pub fn new(inner: T, span: Span) -> Self {
        Spanned { inner, span }
    }

    /// Get a reference to the inner value.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Get a mutable to the inner value.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Take the node's inner value.
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Take the node's inner value and span
    pub fn take(self) -> (T, Span) {
        (self.inner, self.span)
    }

    /// Get the node's span.
    pub fn span(&self) -> Span {
        self.span.clone()
    }
}

impl<T: Display> Display for Spanned<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}
