use rimu_meta::Spanned;

use crate::{expression::Expression, SpannedExpression};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Block {
    Expression(Expression),
    List(Vec<SpannedBlock>),
    Object(Vec<(Spanned<String>, SpannedBlock)>),
    Function {
        args: Vec<Spanned<String>>,
        body: Box<SpannedBlock>,
    },
    Call {
        function: Box<SpannedExpression>,
        args: Box<SpannedBlock>,
    },
    Let {
        variables: Vec<(Spanned<String>, SpannedBlock)>,
        body: Box<SpannedBlock>,
    },
    If {
        condition: Box<SpannedBlock>,
        // then
        consequent: Option<Box<SpannedBlock>>,
        // else
        alternative: Option<Box<SpannedBlock>>,
    },
}

pub type SpannedBlock = Spanned<Block>;
