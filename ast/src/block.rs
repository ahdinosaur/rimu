use rimu_meta::Spanned;

use crate::{expression::Expression, operation::BlockOperation};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Block {
    Object(Vec<(Spanned<String>, SpannedBlock)>),
    List(Vec<SpannedBlock>),
    Expression(Expression),
    Operation(Box<BlockOperation>),
    Function {
        args: Vec<Spanned<String>>,
        body: Box<SpannedBlock>,
    },
}

pub type SpannedBlock = Spanned<Block>;
