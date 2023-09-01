mod eval;
mod if_;
mod let_;

use serde::{de::value::MapDeserializer, Deserialize};

pub use self::eval::EvalOperation;
pub use self::if_::IfOperation;
pub use self::let_::LetOperation;
use crate::{Engine, Environment, Object, ParseError, RenderError, Value};

