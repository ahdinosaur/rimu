/// A binary or arithemtic operator.
#[derive(Debug, PartialEq, PartialOrd, Eq, Clone, Copy)]
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
