use rimu::{evaluate, Environment, ErrorReport, ErrorReports};
use serde::Serialize;
use serde_wasm_bindgen::{Error as SerdeWasmError, Serializer as WasmSerializer};
use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

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
pub fn render(code: &str, source_id: &str, format: Format) -> Result<JsValue, JsValue> {
    let source_id = source_id.parse().unwrap();

    let (block, errors) = rimu::parse(code, source_id);

    let Some(block) = block else {
        let reports: Vec<ErrorReport> = errors.into_iter().map(Into::into).collect::<Vec<_>>();
        let reports: ErrorReports = reports.into();
        return Err(to_js_value(&reports)?);
    };

    let env = Environment::new();

    let value = match evaluate(&block, &env) {
        Ok(value) => value,
        Err(error) => {
            let reports: Vec<ErrorReport> = vec![error.into()];
            let reports: ErrorReports = reports.into();
            return Err(to_js_value(&reports)?);
        }
    };

    let output: JsValue = match format {
        Format::Json => to_js_value(&value)?,
        Format::Yaml => to_js_value(&value)?,
        Format::Toml => match toml::to_string(&value) {
            Ok(string) => string.into(),
            Err(error) => {
                let error = OutputFormatError::new(error.to_string());
                let error: JsValue = error.try_into()?;
                return Err(error);
            }
        },
        _ => panic!("Unexpected format!"),
    };

    Ok(output)
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

impl TryFrom<OutputFormatError> for JsValue {
    type Error = SerdeWasmError;

    fn try_from(value: OutputFormatError) -> Result<Self, Self::Error> {
        to_js_value(&value)
    }
}
