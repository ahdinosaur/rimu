use crate::value::Value;

/// A [`Token`] is the smallest logical unit evaluated by the compiler.
/// It containes either an operator or a literal value.
#[rustfmt::skip]
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Token {
  // Single-character tokens
  LeftParen, RightParen, 
  LeftBracket, RightBracket, 
  Plus, Minus, Star, Slash, 
  Comma,
  // One or two character tokens
  Greater, GreaterEqual,
  Less, LessEqual,
  // Equality
  Equal, NotEqual,
  // Keywords
  And, Or, Xor, Not, Div, Mod,
  // Literal Values
  Literal(Value),
  Identifier(String)
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
