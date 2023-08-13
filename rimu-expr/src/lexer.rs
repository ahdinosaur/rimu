use rimu_span::{Span, Spanned};
use rust_decimal::{Decimal, Error as DecimalError};
use std::str::FromStr;

use crate::Token;

/*
type Span = std::ops::Range<usize>;
type Spanned<T> = (T, Span);

pub fn lexer() -> impl Parser<char, Vec<(Token, Span)>, Error = Simple<char>> {
    let null = just("null").to(Token::Null);

    let boolean = choice((
        just("true").to(Token::Boolean(true)),
        just("false").to(Token::Boolean(false)),
    ));

    let number = text::int(10)
        .chain::<char, _, _>(just('.').chain(text::digits(10)).or_not().flatten())
        .collect::<String>()
        .map(Token::Number);

    let escape = just('\\').ignore_then(
        just('\\')
            .or(just('/'))
            .or(just('"'))
            .or(just('b').to('\x08'))
            .or(just('f').to('\x0C'))
            .or(just('n').to('\n'))
            .or(just('r').to('\r'))
            .or(just('t').to('\t')),
    );

    let string = just('"')
        .ignore_then(filter(|c| *c != '\\' && *c != '"').or(escape).repeated())
        .then_ignore(just('"'))
        .collect::<String>()
        .map(Token::String)
        .labelled("string");

    let delimiter = choice((
        just('(').to(Token::LeftParen),
        just(')').to(Token::RightParen),
        just('[').to(Token::LeftBrack),
        just(']').to(Token::RightBrack),
        just('{').to(Token::LeftBrace),
        just('}').to(Token::RightBrace),
    ));

    let control = choice((
        just(',').to(Token::Comma),
        just(':').to(Token::Colon),
        just('.').to(Token::Dot),
    ));

    let operator = choice((
        just('+').to(Token::Plus),
        just('-').to(Token::Minus),
        just('*').to(Token::Star),
        just('/').to(Token::Slash),
        just('>').to(Token::Greater),
        just(">=").to(Token::GreaterEqual),
        just('<').to(Token::Less),
        just("<=").to(Token::LessEqual),
        just("and").to(Token::And),
        just("or").to(Token::Or),
        just("xor").to(Token::Xor),
        just("not").to(Token::Not),
        just("div").to(Token::Div),
        just("mod").to(Token::Mod),
    ));

    let token = choice((null, boolean, number, string, delimiter, control, operator))
        .recover_with(skip_then_retry_until([]));

    token
        .map_with_span(|tok, span| (tok, span))
        .padded()
        .repeated()
}
*/

pub struct Scanner<'a> {
    source: &'a str,
    start: usize,
    current: usize,
    end: usize,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ScannerErrorKind {
    #[error("empty string")]
    EmptyString,
    #[error("parsing decimal: {0}")]
    ParseDecimal(#[from] DecimalError),
    #[error("invalid token: {0}")]
    InvalidToken(String),
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("at span {span}: {kind}")]
pub struct ScannerError {
    pub span: Span,
    pub kind: ScannerErrorKind,
}

type Result<T> = std::result::Result<T, ScannerError>;

impl<'a> Scanner<'a> {
    pub fn tokenize(source: &'a str) -> Result<Vec<Spanned<Token>>> {
        let mut scanner = Self {
            source,
            start: 0,
            current: 0,
            end: source.chars().count(),
        };

        let mut tokens: Vec<Spanned<Token>> = vec![];

        scanner.skip_whitespace();

        while !scanner.is_at_end() {
            tokens.push(scanner.next_token()?);
            scanner.skip_whitespace();
        }

        if tokens.is_empty() {
            scanner.err(ScannerErrorKind::EmptyString)
        } else {
            Ok(tokens)
        }
    }

    fn next_token(&mut self) -> Result<Spanned<Token>> {
        self.start = self.current;

        let next = self.next_char().unwrap();

        if Scanner::is_identifier_start(next) {
            return self.ok(self.identifier());
        }

        if char::is_numeric(next) {
            return self.ok(self.number()?);
        }

        match next {
            '\'' => self.ok(self.string()),
            '.' => self.ok(Token::Dot),
            '(' => self.ok(Token::LeftParen),
            ')' => self.ok(Token::RightParen),
            '[' => self.ok(Token::LeftBrack),
            ']' => self.ok(Token::RightBrack),
            ',' => self.ok(Token::Comma),
            '+' => self.ok(Token::Plus),
            '-' => self.ok(Token::Minus),
            '*' => self.ok(Token::Star),
            '/' => self.ok(Token::Slash),
            '=' => self.ok(Token::Equal),
            '>' => self.ok(self.greater()),
            '<' => self.ok(self.lesser()),
            _ => self.err(SyntaxError(format!("invalid token: {next}"))),
        }
    }

    fn is_at_end(&self) -> bool {
        self.current == self.end
    }

    fn advance(&mut self) {
        self.current += 1;
    }

    fn advance_numeric(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_numeric() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn next_char(&mut self) -> Option<char> {
        self.advance();
        self.source.chars().nth(self.current - 1)
    }

    fn peek(&self) -> Option<char> {
        self.peek_ahead(0)
    }

    fn peek_ahead(&self, offset: usize) -> Option<char> {
        self.source.chars().nth(self.current + offset)
    }

    fn skip_whitespace(&mut self) {
        while let Some(' ' | '\r' | '\t' | '\n') = self.peek() {
            self.advance();
        }
    }

    fn get_content(&self, trim_by: usize) -> String {
        let from = self.start + trim_by;
        let to = self.current - trim_by;

        self.source.chars().take(to).skip(from).collect()
    }

    fn is_identifier_start(character: char) -> bool {
        character.is_alphabetic() || character == '_'
    }

    fn is_identifier(character: char) -> bool {
        character.is_alphanumeric() || character == '_' || character == '-'
    }

    fn identifier(&mut self) -> Token {
        while self.peek().is_some_and(Scanner::is_identifier) {
            self.advance();
        }

        let ident = self.get_content(0);

        match ident.to_lowercase().as_str() {
            "null" => Token::Null,
            "true" => Token::Boolean(true),
            "false" => Token::Boolean(false),
            "and" => Token::And,
            "or" => Token::Or,
            "xor" => Token::Xor,
            "not" => Token::Not,
            "div" => Token::Div,
            "mod" => Token::Mod,
            _ => Token::Identifier(ident),
        }
    }

    fn extract_number(content: &str) -> Result<Decimal> {
        Ok(Decimal::from_str(content)?)
    }

    fn number(&mut self) -> Result<Token> {
        self.advance_numeric(); // advance integral

        if self.peek() == Some('.') {
            self.advance(); // advance dot

            if let Some(fractional) = self.peek() {
                if fractional.is_numeric() {
                    self.advance_numeric(); // advance fraction
                }
            }
        }

        let content = self.get_content(0);
        let number = Scanner::extract_number(content.as_str())?;

        Ok(Token::Number(number))
    }

    fn string(&mut self) -> Result<Token> {
        while self.peek().is_some_and(|c| c != '\'') {
            self.advance();
        }

        if self.is_at_end() {
            let message = format!("Unterminated String at character {}", self.start);
            return Err(SyntaxError(message));
        };

        self.advance();
        let content = self.get_content(1);

        Ok(Token::Literal(Value::String(content)))
    }

    fn encounter_double(&mut self, token: Token) -> Token {
        self.advance();
        token
    }

    fn greater(&mut self) -> Token {
        match self.peek() {
            Some('=') => self.encounter_double(Token::GreaterEqual),
            _ => Token::Greater,
        }
    }

    fn lesser(&mut self) -> Token {
        match self.peek() {
            Some('=') => self.encounter_double(Token::LessEqual),
            Some('>') => self.encounter_double(Token::NotEqual),
            _ => Token::Less,
        }
    }

    fn current_span(&self) -> Span {
        (self.start, self.current).into()
    }

    fn err<T>(&self, err_kind: ScannerErrorKind) -> Result<T> {
        Err(ScannerError {
            span: self.current_span(),
            kind: err_kind,
        })
    }

    fn ok<T>(&self, contents: T) -> Result<Spanned<T>> {
        Ok(Spanned::from(self.current_span(), contents))
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use super::{Scanner, Token};
    use crate::{
        error::{Result, SyntaxError},
        value::Value,
    };

    #[test]
    fn simple_bool() -> Result<()> {
        let tokens = Scanner::tokenize("True")?;
        let expected = Token::Literal(Value::Boolean(true));

        assert_eq!(tokens[0], expected);
        Ok(())
    }

    #[test]
    fn simple_integer() -> Result<()> {
        let tokens = Scanner::tokenize("9001")?;
        let expected = Token::Literal(Value::Number(9001.0));

        assert_eq!(tokens[0], expected);
        Ok(())
    }

    #[test]
    fn simple_float() -> Result<()> {
        let tokens = Scanner::tokenize("3.141592653589793")?;
        let expected = Token::Literal(Value::Number(PI));

        assert_eq!(tokens[0], expected);
        Ok(())
    }

    #[test]
    fn simple_string() -> Result<()> {
        let tokens = Scanner::tokenize("'Hello World'")?;
        let expected = Token::Literal(Value::String(String::from("Hello World")));

        assert!(tokens.first().is_some());
        assert_eq!(tokens[0], expected);
        Ok(())
    }

    #[test]
    fn multiple_tokens() -> Result<()> {
        let tokens = Scanner::tokenize("1 + 1")?;
        let expected: Vec<Token> = vec![
            Token::Literal(Value::Number(1.0)),
            Token::Plus,
            Token::Literal(Value::Number(1.0)),
        ];

        assert_eq!(tokens, expected);
        Ok(())
    }

    #[test]
    fn var_name_underscore() -> Result<()> {
        let tokens = Scanner::tokenize("(_SOME_VAR1 * ANOTHER-ONE)")?;
        let expected = vec![
            Token::LeftParen,
            Token::Identifier(String::from("_SOME_VAR1")),
            Token::Star,
            Token::Identifier(String::from("ANOTHER-ONE")),
            Token::RightParen,
        ];

        assert_eq!(expected, tokens);
        Ok(())
    }

    #[test]
    fn unterminated_less() -> Result<()> {
        let tokens = Scanner::tokenize("<")?;
        let expected = vec![Token::Less];

        assert_eq!(expected, tokens);
        Ok(())
    }

    fn test_number(input: &str, expected: f64) -> Result<()> {
        let tokens = Scanner::tokenize(input)?;
        let expected = vec![Token::Literal(Value::Number(expected))];

        assert_eq!(expected, tokens);
        Ok(())
    }

    #[test]
    fn number_parts() -> Result<()> {
        test_number("10", 10.0)?;
        test_number("10.0", 10.0)?;
        test_number("20.4", 20.4)?;
        test_number("30.", 30.0)?;
        test_number(".4", 0.4)?;

        Ok(())
    }

    #[test]
    fn err_empty_input() {
        let tokens = Scanner::tokenize("");
        let expected = Err(SyntaxError::from("empty String"));

        assert_eq!(expected, tokens);
    }

    #[test]
    fn err_unknown_token_1() {
        let tokens = Scanner::tokenize("$");
        let expected = Err(SyntaxError::from("invalid token: $"));

        assert_eq!(expected, tokens);
    }

    #[test]
    fn err_unknown_token_2() {
        let tokens = Scanner::tokenize("$hello");
        let expected = Err(SyntaxError::from("invalid token: $"));

        assert_eq!(expected, tokens);
    }

    #[test]
    fn err_unterminated_string() {
        let tokens = Scanner::tokenize("'hello' + 'world");
        let expected = Err(SyntaxError::from("Unterminated String at character 10"));

        assert_eq!(expected, tokens);
    }
}
