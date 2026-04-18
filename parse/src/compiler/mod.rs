use chumsky::{extra, input::ValueInput, prelude::*};
use rimu_meta::Span;

use crate::token::Token;

mod block;
mod expression;

pub(crate) use block::compile_block;
pub(crate) use expression::compile_expression;

pub type CompilerError = Rich<'static, Token, Span>;

pub(crate) trait Compiler<'src, I, T>:
    Parser<'src, I, T, extra::Err<Rich<'src, Token, Span>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = Span>,
{
}

impl<'src, I, P, T> Compiler<'src, I, T> for P
where
    I: ValueInput<'src, Token = Token, Span = Span>,
    P: Parser<'src, I, T, extra::Err<Rich<'src, Token, Span>>> + Clone,
{
}
