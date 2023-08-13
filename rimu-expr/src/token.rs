use std::fmt;

/// A [`Token`] is the smallest logical unit evaluated by the compiler.
/// It containes either an operator or a literal value.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    Invalid(char),

    Null,

    Boolean(bool),

    String(String),

    Number(String),

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
    /// =
    Equal,
    /// !=
    NotEqual,
    /// and
    And,
    /// or
    Or,
    /// xor
    Xor,
    /// not
    Not,
    /// div
    Div,
    /// mod
    Mod,
}

impl fmt::Display for Token {
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
            Token::Equal => write!(f, "="),
            Token::NotEqual => write!(f, "!="),
            Token::And => write!(f, "and"),
            Token::Or => write!(f, "or"),
            Token::Xor => write!(f, "xor"),
            Token::Not => write!(f, "not"),
            Token::Div => write!(f, "div"),
            Token::Mod => write!(f, "mod"),
        }
    }
}

/// The precedences used to order the operators evaluated in the
/// [Pratt-Parser](https://en.wikipedia.org/wiki/Operator-precedence_parser#Pratt_parsing)
/// when building the [`Expression`](crate::ast::Expression) tree.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum Precedence {
    None,
    Or,         // or
    And,        // and
    Xor,        // xor
    Equality,   // = <>
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * / div mod
    Unary,      // not -
    Call,       // ()
    Primary,    // Literals
}

#[rustfmt::skip]
impl From<&Token> for Precedence {
    fn from(token: &Token) -> Self {
        match token {
            Token::Minus | Token::Plus => Precedence::Term,
            Token::Star | Token::Slash | Token::Div | Token::Mod => Precedence::Factor,
            Token::Equal | Token::NotEqual => Precedence::Equality,
            Token::Greater | Token::GreaterEqual | Token::Less | Token::LessEqual => Precedence::Comparison,
            Token::And => Precedence::And,
            Token::Or => Precedence::Or,
            Token::Xor => Precedence::Xor, 
            Token::LeftParen => Precedence::Call,
            _ => Precedence::None,
        }
    }
}

impl Precedence {
    pub fn next(self) -> Precedence {
        match self {
            Precedence::None => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Xor,
            Precedence::Xor => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => Precedence::None,
        }
    }
}
