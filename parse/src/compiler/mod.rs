use chumsky::{prelude::Simple, Parser};
use rimu_meta::Span;

use crate::token::Token;

mod block;
mod expression;

pub(crate) use block::compile_block;
pub(crate) use expression::compile_expression;

pub type CompilerError = Simple<Token, Span>;

pub(crate) trait Compiler<T>:
    Parser<Token, T, Error = CompilerError> + Sized + Clone
{
}
impl<P, T> Compiler<T> for P where P: Parser<Token, T, Error = CompilerError> + Clone {}
