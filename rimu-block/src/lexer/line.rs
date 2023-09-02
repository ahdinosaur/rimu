use chumsky::Parser;
use rimu_expr::{lexer_parser as expr_lexer_parser, LexerError as ExprLexerError};
use rimu_report::{SourceId, Span, Spanned};
use rimu_token::SpannedToken;

pub type LineLexerError = ExprLexerError;

pub(crate) fn tokenize_line(
    spanned_line: Spanned<&str>,
    source: SourceId,
) -> (Option<Vec<SpannedToken>>, Vec<LineLexerError>) {
    let (line, span) = spanned_line.take();
    let eoi = Span::new(source.clone(), span.end(), span.end());
    expr_lexer_parser().parse_recovery(chumsky::Stream::from_iter(
        eoi,
        line.chars().enumerate().map(|(i, c)| {
            (
                c,
                Span::new(source.clone(), span.start() + i, span.start() + i + 1),
            )
        }),
    ))
}
