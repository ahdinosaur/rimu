use chumsky::prelude::*;
use rimu_report::{SourceId, Span, Spanned};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LineToken {
    Key(String),
    ListItem,
    Value(String),
}

impl fmt::Display for LineToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LineToken::Key(_) => todo!(),
            LineToken::ListItem => todo!(),
            LineToken::Value(_) => todo!(),
        }
    }
}

pub type SpannedLineToken = Spanned<LineToken>;

pub type LineLexerError = Simple<char, Span>;

pub trait Lexer<T>: Parser<char, T, Error = LineLexerError> + Sized + Clone {}
impl<P, T> Lexer<T> for P where P: Parser<char, T, Error = LineLexerError> + Clone {}

pub fn tokenize(
    line: &str,
    source: SourceId,
) -> Result<Vec<SpannedLineToken>, Vec<LineLexerError>> {
    let len = line.chars().count();
    let eoi = Span::new(source.clone(), len, len);
    lexer_parser().parse(chumsky::Stream::from_iter(
        eoi,
        line.chars()
            .enumerate()
            .map(|(i, c)| (c, Span::new(source.clone(), i, i + 1))),
    ))
}

pub fn lexer_parser() -> impl Lexer<Vec<SpannedLineToken>> {
    let space = text::whitespace();
    let colon = just(":");
    let li = just("-")
        .to(LineToken::ListItem)
        .map_with_span(Spanned::new)
        .labelled("list item");

    let value = any()
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(LineToken::Value)
        .map_with_span(Spanned::new)
        .labelled("value");

    let escape = just('\\')
        .ignore_then(
            just('\\')
                .or(just('/'))
                .or(just('"'))
                .or(just('b').to('\x08'))
                .or(just('f').to('\x0C'))
                .or(just('n').to('\n'))
                .or(just('r').to('\r'))
                .or(just('t').to('\t')),
        )
        .labelled("escape");

    let string = just('"')
        .ignore_then(filter(|c| *c != '\\' && *c != '"').or(escape).repeated())
        .then_ignore(just('"'))
        .collect::<String>()
        .labelled("string");

    let identifier = text::ident().labelled("identifier");

    let key = string
        .or(identifier)
        .map(LineToken::Key)
        .map_with_span(Spanned::new)
        .labelled("key")
        .boxed();

    let object_key = key
        .clone()
        .then_ignore(space)
        .then_ignore(colon)
        .then_ignore(space)
        .map(|key| vec![key])
        .boxed();
    let object_key_value = key
        .clone()
        .then_ignore(space)
        .then_ignore(colon)
        .then_ignore(space)
        .then(value)
        .then_ignore(space)
        .map(|(key, value)| vec![key, value])
        .boxed();
    let list_item = li
        .clone()
        .then_ignore(space)
        .then(value)
        .then_ignore(space)
        .map(|(li, value)| vec![li, value])
        .boxed();
    let list_item_key = li
        .clone()
        .then_ignore(space)
        .then(key.clone())
        .then_ignore(colon)
        .then_ignore(space)
        .map(|(li, key)| vec![li, key])
        .boxed();
    let list_item_key_value = li
        .clone()
        .then_ignore(space)
        .then(key.clone())
        .then_ignore(colon)
        .then_ignore(space)
        .then(value)
        .then_ignore(space)
        .map(|((li, key), value)| vec![li, key, value])
        .boxed();

    let token = object_key_value
        .or(object_key)
        .or(list_item_key_value)
        .or(list_item_key)
        .or(list_item)
        .boxed()
        .recover_with(skip_then_retry_until([]));

    token.then_ignore(end())
}

#[cfg(test)]
mod tests {

    use std::ops::Range;

    use pretty_assertions::assert_eq;
    use rimu_report::SourceId;

    use super::{tokenize, LineLexerError, LineToken, Span, Spanned, SpannedLineToken};

    fn span(range: Range<usize>) -> Span {
        Span::new(SourceId::empty(), range.start, range.end)
    }

    fn test(code: &str) -> Result<Vec<SpannedLineToken>, Vec<LineLexerError>> {
        tokenize(code, SourceId::empty())
    }

    /*
    #[test]
    fn empty_input() {
        let actual = test("");

        let expected = Ok(vec![]);

        assert_eq!(actual, expected);
    }
    */

    #[test]
    fn key_value() {
        let actual = test("key: value");

        let expected = Ok(vec![
            Spanned::new(LineToken::Key("key".into()), span(0..3)),
            Spanned::new(LineToken::Value("value".into()), span(5..10)),
        ]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn key_identifier() {
        let actual = test("key:");

        let expected = Ok(vec![Spanned::new(LineToken::Key("key".into()), span(0..3))]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn key_quoted() {
        let actual = test("\"key\":");

        let expected = Ok(vec![Spanned::new(LineToken::Key("key".into()), span(0..5))]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn list_item() {
        let actual = test("- list item");

        let expected = Ok(vec![
            Spanned::new(LineToken::ListItem, span(0..1)),
            Spanned::new(LineToken::Value("list item".into()), span(2..11)),
        ]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn list_item_key_value() {
        let actual = test("- key: value");

        let expected = Ok(vec![
            Spanned::new(LineToken::ListItem, span(0..1)),
            Spanned::new(LineToken::Key("key".into()), span(2..5)),
            Spanned::new(LineToken::Value("value".into()), span(7..12)),
        ]);

        assert_eq!(actual, expected);
    }

    #[test]
    fn list_item_key() {
        let actual = test("- key:");

        let expected = Ok(vec![
            Spanned::new(LineToken::ListItem, span(0..1)),
            Spanned::new(LineToken::Key("key".into()), span(2..5)),
        ]);

        assert_eq!(actual, expected);
    }
}
