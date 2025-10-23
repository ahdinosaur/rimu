use std::{cell::RefCell, path::PathBuf, rc::Rc, str::FromStr};

use rimu::{
    create_stdlib, evaluate, Environment, ErrorReport, ErrorReports, SerdeValue, SourceId, Value,
};
use serde::Serialize;
use serde_wasm_bindgen::{Error as SerdeWasmError, Serializer as WasmSerializer};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn init() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub enum Format {
    Json = "json",
    Yaml = "yaml",
    Toml = "toml",
}

#[wasm_bindgen]
pub fn render(code: &str, source_id: &str, format: Format) -> Result<String, JsValue> {
    let source_id = SourceId::Path(PathBuf::from_str(source_id).unwrap());

    let (block, errors) = rimu::parse(code, source_id);

    let Some(block) = block else {
        let reports: Vec<ErrorReport> = errors.into_iter().map(Into::into).collect::<Vec<_>>();
        let reports: ErrorReports = reports.into();
        return Err(to_js_value(&reports)?);
    };

    let stdlib = create_stdlib();
    let env = Environment::from_object(&stdlib, None).unwrap();
    let env = Rc::new(RefCell::new(env));
    let value = match evaluate(&block, env) {
        Ok(value) => value,
        Err(error) => {
            let reports: Vec<ErrorReport> = vec![error.into()];
            let reports: ErrorReports = reports.into();
            return Err(to_js_value(&reports)?);
        }
    };
    let value: Value = value.into_inner();
    let value: SerdeValue = value.into();

    let output: Result<String, OutputFormatError> = match format {
        Format::Json => serde_json::to_string_pretty(&value).map_err(OutputFormatError::new),
        Format::Yaml => serde_yaml::to_string(&value).map_err(OutputFormatError::new),
        Format::Toml => toml::to_string_pretty(&value).map_err(OutputFormatError::new),
        _ => panic!("Unexpected format!"),
    };

    match output {
        Ok(output) => Ok(output),
        Err(error) => Err(to_js_value(&error)?),
    }
}

pub fn to_js_value<T: serde::ser::Serialize + ?Sized>(
    value: &T,
) -> Result<JsValue, SerdeWasmError> {
    value.serialize(&WasmSerializer::json_compatible())
}

#[derive(Serialize)]
pub struct OutputFormatError {
    pub message: String,
}

impl OutputFormatError {
    fn new<E: ToString>(error: E) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}
