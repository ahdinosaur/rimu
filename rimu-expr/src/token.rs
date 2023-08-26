use rust_decimal::Decimal;
use std::fmt;

use crate::Spanned;

/// A [`Token`] is the smallest logical unit evaluated by the compiler.
/// It containes either an operator or a literal value.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token<'src> {
    Invalid(char),

    Null,

    Boolean(bool),

    String(&'src str),

    Number(Decimal),

    Identifier(&'src str),

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

    /// ,
    Comma,
    /// :
    Colon,

    /// .
    Dot,

    /// +
    Plus,
    /// -
    Minus,
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

impl<'src> fmt::Display for Token<'src> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Invalid(c) => write!(f, "{:?}", c),
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
            Token::Comma => write!(f, ","),
            Token::Colon => write!(f, ":"),
            Token::Dot => write!(f, "."),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
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

pub type SpannedToken<'src> = Spanned<Token<'src>>;
