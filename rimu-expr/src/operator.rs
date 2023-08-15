/// A binary or arithemtic operator.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Equal,
    NotEqual,
    And,
    Or,
    Xor,
    Not,
    Div,
    Mod,
}
