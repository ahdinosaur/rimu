mod utils;

use std::string::ParseError;

use rimu::{ErrorReport, SourceId, SpannedBlock, Value};
use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn exec(code: &str) -> (Option<Value>, Vec<ErrorReport>) {
    let source_id = SourceId::empty();
    let (block, errors) = rimu::parse(code, source_id);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, rimu-wasm!");
}
