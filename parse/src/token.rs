use rust_decimal::Decimal;
use std::fmt;

use rimu_meta::Spanned;

/// A [`Token`] is the smallest logical unit evaluated by the compiler.
/// It containes either an operator or a literal value.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    Invalid(char),

    Indent,
    Dedent,
    EndOfLine,

    Null,

    Boolean(bool),

    String(String),

    Number(Decimal),

    Identifier(String),

    /// (
    LeftParen,
    /// )
    RightParen,
    /// [
    LeftBrack,
    /// ]
    RightBrack,
    /// {
    LeftBrace,
    /// }
    RightBrace,

    /// if
    If,
    /// then
    Then,
    /// else
    Else,
    /// let
    Let,
    /// in
    In,

    /// ,
    Comma,
    /// :
    Colon,

    /// .
    Dot,

    /// =>
    FatArrow,

    /// +
    Plus,
    /// -
    Dash,
    /// *
    Star,
    /// /
    Slash,
    /// >
    Greater,
    /// >=
    GreaterEqual,
    /// <
    Less,
    /// <=
    LessEqual,
    /// ==
    Equal,
    /// !=
    NotEqual,
    /// &&
    And,
    /// ||
    Or,
    /// ^
    Xor,
    /// !
    Not,
    /// %
    Rem,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Invalid(c) => write!(f, "{:?}", c),
            Token::Indent => write!(f, "  "),
            Token::Dedent => write!(f, ""),
            Token::EndOfLine => writeln!(f),
            Token::Null => write!(f, "null"),
            Token::Boolean(b) => match b {
                true => write!(f, "true"),
                false => write!(f, "false"),
            },
            Token::String(s) => write!(f, "{}", s),
            Token::Number(n) => write!(f, "{}", n),
            Token::Identifier(i) => write!(f, "{}", i),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::LeftBrack => write!(f, "["),
            Token::RightBrack => write!(f, "]"),
            Token::LeftBrace => write!(f, "{{"),
            Token::RightBrace => write!(f, "}}"),
            Token::If => write!(f, "if"),
            Token::Then => write!(f, "then"),
            Token::Else => write!(f, "else"),
            Token::Let => write!(f, "let"),
            Token::In => write!(f, "in"),
            Token::Comma => write!(f, ","),
            Token::Colon => write!(f, ":"),
            Token::Dot => write!(f, "."),
            Token::FatArrow => write!(f, "=>"),
            Token::Plus => write!(f, "+"),
            Token::Dash => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Greater => write!(f, ">"),
            Token::GreaterEqual => write!(f, ">="),
            Token::Less => write!(f, "<"),
            Token::LessEqual => write!(f, "<="),
            Token::Equal => write!(f, "=="),
            Token::NotEqual => write!(f, "!="),
            Token::And => write!(f, "&&"),
            Token::Or => write!(f, "||"),
            Token::Xor => write!(f, "^"),
            Token::Not => write!(f, "!"),
            Token::Rem => write!(f, "%"),
        }
    }
}

pub type SpannedToken = Spanned<Token>;
