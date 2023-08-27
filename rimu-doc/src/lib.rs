// maybe this shouldn't use chumsky.
//
// state
// - current indentation
//
// parse each line
//   - get indentation (relative to current)
//   - get type
//     - just use regexes
//     - if has ":" (not inside string), then object entry
//     - if starts with "-", then list item
//
// tokens:
// - indent
// - dedent
// - key
// - value
// - list item
//
// doc:
// - object
// - list
// - expression: string

mod lines;

use std::collections::BTreeMap;

use rimu_report::Spanned;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Token {
    Indent,
    Dedent,
    Key(String),
    Value(String),
    ListItem(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Doc {
    Object(BTreeMap<String, Box<SpannedDoc>>),
    List(Box<SpannedDoc>),
    Expression(String),
}

pub type SpannedDoc = Spanned<Doc>;
