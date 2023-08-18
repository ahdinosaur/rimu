mod eval;
mod if_;
mod let_;

use serde::{de::value::MapDeserializer, Deserialize};

pub use self::eval::EvalBlock;
pub use self::if_::IfBlock;
pub use self::let_::LetBlock;
use crate::{Environment, Engine, Object, ParseError, RenderError, Value};

pub trait Block {
    fn render(&self, engine: &Engine, context: &Environment) -> Result<Value, RenderError>;
}

#[derive(Clone, Debug, PartialEq)]
pub enum Blocks {
    Eval(EvalBlock),
    Let(LetBlock),
    If(IfBlock),
}

impl Blocks {
    pub(crate) fn render(&self, engine: &Engine, context: &Environment) -> Result<Value, RenderError> {
        match self {
            Blocks::Eval(op) => op.render(engine, context),
            Blocks::Let(op) => op.render(engine, context),
            Blocks::If(op) => op.render(engine, context),
        }
    }
}

pub(crate) fn find_block_key(object: &Object) -> Result<Option<String>, ParseError> {
    let block_keys: Vec<&String> = object
        .keys()
        .filter(|key| {
            let mut chars = key.chars();
            chars.next() == Some('$') && chars.next() != Some('$')
        })
        .collect();
    if block_keys.len() > 1 {
        Err(ParseError::TooManyBlockKeys)
    } else if block_keys.len() == 1 {
        Ok(Some(block_keys[0].to_owned()))
    } else {
        Ok(None)
    }
}

pub(crate) fn parse_block(block_key: &str, object: &Object) -> Result<Blocks, ParseError> {
    let map_de = MapDeserializer::new(object.clone().into_iter());
    match block_key {
        "$eval" => Ok(Blocks::Eval(EvalBlock::deserialize(map_de)?)),
        "$let" => Ok(Blocks::Let(LetBlock::deserialize(map_de)?)),
        "$if" => Ok(Blocks::If(IfBlock::deserialize(map_de)?)),
        _ => Err(ParseError::UnknownBlockKey {
            block_key: block_key.to_owned(),
        }),
    }
}

pub(crate) fn unescape_non_block_key(key: &str) -> &str {
    if key.starts_with("$$") {
        &key[1..]
    } else {
        &key[..]
    }
}
