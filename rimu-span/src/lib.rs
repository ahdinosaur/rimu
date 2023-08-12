// Inspired by:
//
// - https://docs.rs/miette/latest/miette/struct.Span.html
// - https://docs.rs/codespan/latest/codespan/struct.Span.html
// - https://github.com/noir-lang/noir/blob/master/crates/noirc_errors/src/position.rs

/// A span of source code.
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
pub struct Span {
    /// The start of the span.
    start: usize,
    /// The total length of the span
    length: usize,
}

impl Span {
    /// Create a new [`Span`].
    pub const fn new(start: usize, length: usize) -> Self {
        Self { start, length }
    }

    /// The absolute start, in bytes, from the beginning of a [`SourceCode`].
    pub const fn start(&self) -> usize {
        self.start
    }

    /// Total length of the [`Span`], in bytes.
    pub const fn len(&self) -> usize {
        self.length
    }

    /// Whether this [`Span`] has a length of zero. It may still be useful
    /// to point to a specific point.
    pub const fn is_empty(&self) -> bool {
        self.length == 0
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
            length: range.len(),
        }
    }
}

#[derive(Debug, Clone, PartialOrd, Ord, Eq)]
pub struct Spanned<T> {
    pub contents: T,
    span: Span,
}

/// This is important for tests. Two Spanned objects are equal if their content is equal
/// They may not have the same span. Use into_span to test for Span being equal specifically
impl<T: std::cmp::PartialEq> PartialEq<Spanned<T>> for Spanned<T> {
    fn eq(&self, other: &Spanned<T>) -> bool {
        self.contents == other.contents
    }
}

impl chumsky::Span for Span {
    type Context = ();

    type Offset = u32;

    fn new(_context: Self::Context, range: Range<Self::Offset>) -> Self {
        Span::new(range)
    }

    fn context(&self) -> Self::Context {}

    fn start(&self) -> Self::Offset {
        self.start()
    }

    fn end(&self) -> Self::Offset {
        self.end()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Location {
    pub span: Span,
    pub file: FileId,
}

impl Location {
    pub fn new(span: Span, file: FileId) -> Self {
        Self { span, file }
    }
}
