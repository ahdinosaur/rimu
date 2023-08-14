// with help from https://github.com/DennisPrediger/SLAC/blob/main/src/scanner.rs

use rimu_span::{Span, Spanned, SpannedError};
use rust_decimal::{Decimal, Error as DecimalError};
use std::str::FromStr;

use crate::Token;

pub struct Scanner<'a> {
    source: &'a str,
    start: usize,
    current: usize,
    end: usize,
}

#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum ScannerError {
    #[error("empty string")]
    EmptyString,
    #[error("parsing decimal: {0}")]
    ParseDecimal(#[source] DecimalError),
    #[error("invalid token: {token}")]
    InvalidToken { token: String },
    #[error("undeterminated String at character {start}")]
    UnterminatedString { start: usize },
}

pub fn tokenize(source: &str) -> Result<Vec<Spanned<Token>>, SpannedError<ScannerError>> {
    Scanner::new(source).tokenize()
}

impl<'a> Scanner<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            start: 0,
            current: 0,
            end: source.chars().count(),
        }
    }

    fn tokenize(&mut self) -> Result<Vec<Spanned<Token>>, SpannedError<ScannerError>> {
        let mut tokens: Vec<Spanned<Token>> = vec![];

        self.skip_whitespace();

        while !self.is_at_end() {
            match self.next_token() {
                Ok(token) => {
                    tokens.push(self.spanned(token));
                    self.skip_whitespace();
                }
                Err(error) => {
                    return Err(self.spanned_err(error));
                }
            }
        }

        if tokens.is_empty() {
            Err(self.spanned_err(ScannerError::EmptyString))
        } else {
            Ok(tokens)
        }
    }

    fn next_token(&mut self) -> Result<Token, ScannerError> {
        self.start = self.current;

        let next = self.next_char().unwrap();

        if Scanner::is_identifier_start(next) {
            return Ok(self.identifier());
        }

        if char::is_numeric(next) {
            return self.number();
        }

        match next {
            ',' => Ok(Token::Comma),
            ':' => Ok(Token::Colon),
            '.' => Ok(Token::Dot),
            '(' => Ok(Token::LeftParen),
            ')' => Ok(Token::RightParen),
            '[' => Ok(Token::LeftBrack),
            ']' => Ok(Token::RightBrack),
            '{' => Ok(Token::LeftBrace),
            '}' => Ok(Token::RightBrace),
            '\'' => self.string(),
            '+' => Ok(Token::Plus),
            '-' => Ok(Token::Minus),
            '*' => Ok(Token::Star),
            '/' => Ok(Token::Slash),
            '=' => Ok(Token::Equal),
            '>' => Ok(self.greater()),
            '<' => Ok(self.lesser()),
            _ => Err(ScannerError::InvalidToken { token: next.into() }),
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
        character.is_alphanumeric() || character == '_'
    }

    fn identifier(&mut self) -> Token {
        while self.peek().is_some_and(Scanner::is_identifier) {
            self.advance();
        }

        let ident = self.get_content(0);

        match ident.as_str() {
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

    fn extract_number(&self, content: &str) -> Result<Decimal, ScannerError> {
        Decimal::from_str(content).map_err(|error| ScannerError::ParseDecimal(error))
    }

    fn number(&mut self) -> Result<Token, ScannerError> {
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
        let number = self.extract_number(content.as_str())?;

        Ok(Token::Number(number))
    }

    fn string(&mut self) -> Result<Token, ScannerError> {
        while self.peek().is_some_and(|c| c != '\'') {
            self.advance();
        }

        if self.is_at_end() {
            return Err(ScannerError::UnterminatedString { start: self.start });
        };

        self.advance();
        let content = self.get_content(1);

        Ok(Token::String(content))
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

    fn spanned_err(&self, error: ScannerError) -> SpannedError<ScannerError> {
        SpannedError {
            span: self.current_span(),
            error,
        }
    }

    fn spanned<T: Clone>(&self, contents: T) -> Spanned<T> {
        Spanned::from(self.current_span(), contents)
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error, f64::consts::PI};

    use pretty_assertions::assert_eq;
    use rimu_span::{Spanned, SpannedError};
    use rust_decimal::{prelude::FromPrimitive, Decimal};

    use super::{tokenize, ScannerError, Token};

    fn assert_eq_tokens(tokens: Vec<Spanned<Token>>, expected: Vec<Token>) {
        let actual: Vec<Token> = tokens.into_iter().map(|i| i.contents().clone()).collect();
        assert_eq!(actual, expected);
    }

    fn assert_eq_result(
        tokens: Result<Vec<Spanned<Token>>, SpannedError<ScannerError>>,
        expected: Result<Vec<Token>, ScannerError>,
    ) {
        let actual = tokens
            .map(|t| {
                t.into_iter()
                    .map(|i| i.contents().clone())
                    .collect::<Vec<Token>>()
            })
            .map_err(|error| error.error().clone());
        assert_eq!(actual, expected);
    }

    #[test]
    fn simple_null() -> Result<(), Box<dyn Error>> {
        let actual = tokenize("null")?;

        let expected = vec![Spanned::from((0, 4), Token::Null)];

        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn simple_bool() -> Result<(), Box<dyn Error>> {
        let actual = tokenize("true")?;

        let expected = vec![Spanned::from((0, 4), Token::Boolean(true))];

        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn simple_integer() -> Result<(), Box<dyn Error>> {
        let actual = tokenize("9001")?;

        let expected = vec![Spanned::from(
            (0, 4),
            Token::Number(Decimal::from_u64(9001).unwrap()),
        )];

        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn simple_float() -> Result<(), Box<dyn Error>> {
        let actual = tokenize("3.141592653589793")?;

        let expected = vec![Spanned::from(
            (0, 17),
            Token::Number(Decimal::from_f64(PI).unwrap()),
        )];

        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn simple_string() -> Result<(), Box<dyn Error>> {
        let actual = tokenize("'Hello World'")?;

        let expected = vec![Spanned::from(
            (0, 13),
            Token::String(String::from("Hello World")),
        )];

        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn multiple_tokens() -> Result<(), Box<dyn Error>> {
        let actual = tokenize("1 + 1")?;

        let expected: Vec<Spanned<Token>> = vec![
            Spanned::from((0, 1), Token::Number(Decimal::from_u8(1).unwrap())),
            Spanned::from((2, 3), Token::Plus),
            Spanned::from((4, 5), Token::Number(Decimal::from_u8(1).unwrap())),
        ];

        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn var_name_underscore() -> Result<(), Box<dyn Error>> {
        let actual = tokenize("(_SOME_VAR1 * ANOTHER_ONE)")?;

        let expected = vec![
            Spanned::from((0, 1), Token::LeftParen),
            Spanned::from((1, 11), Token::Identifier(String::from("_SOME_VAR1"))),
            Spanned::from((12, 13), Token::Star),
            Spanned::from((14, 25), Token::Identifier(String::from("ANOTHER_ONE"))),
            Spanned::from((25, 26), Token::RightParen),
        ];

        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn unterminated_less() -> Result<(), Box<dyn Error>> {
        let tokens = tokenize("<")?;
        let expected = vec![Token::Less];

        assert_eq_tokens(tokens, expected);
        Ok(())
    }

    fn test_number(input: &str, expected: f64) -> Result<(), Box<dyn Error>> {
        let tokens = tokenize(input)?;
        let expected = vec![Token::Number(Decimal::from_f64(expected).unwrap())];

        assert_eq_tokens(tokens, expected);
        Ok(())
    }

    #[test]
    fn number_parts() -> Result<(), Box<dyn Error>> {
        test_number("10", 10.0)?;
        test_number("10.0", 10.0)?;
        test_number("20.4", 20.4)?;
        test_number("30.", 30.0)?;

        Ok(())
    }

    #[test]
    fn err_empty_input() {
        let tokens = tokenize("");
        let expected = Err(ScannerError::EmptyString);

        assert_eq_result(tokens, expected);
    }

    #[test]
    fn err_unknown_token_1() {
        let tokens = tokenize("$");
        let expected = Err(ScannerError::InvalidToken { token: "$".into() });

        assert_eq_result(tokens, expected);
    }

    #[test]
    fn err_unknown_token_2() {
        let tokens = tokenize("$hello");
        let expected = Err(ScannerError::InvalidToken { token: "$".into() });

        assert_eq_result(tokens, expected);
    }

    #[test]
    fn err_unterminated_string() {
        let tokens = tokenize("'hello' + 'world");
        let expected = Err(ScannerError::UnterminatedString { start: 10 });

        assert_eq_result(tokens, expected);
    }
}
