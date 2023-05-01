use dpp::errors::consensus::basic::json_schema_error::JsonSchemaError;
use serde::Serialize;

use dpp::errors::consensus::codes::ErrorWithCode;

use dpp::errors::consensus::ConsensusError;

use crate::buffer::Buffer;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name=JsonSchemaError, inspectable)]
#[derive(Debug)]
pub struct JsonSchemaErrorWasm {
    inner: JsonSchemaError,
}

impl From<&JsonSchemaError> for JsonSchemaErrorWasm {
    fn from(e: &JsonSchemaError) -> Self {
        Self { inner: e.clone() }
    }
}

#[wasm_bindgen(js_class=JsonSchemaError)]
impl JsonSchemaErrorWasm {
    #[wasm_bindgen(js_name=getKeyword)]
    pub fn keyword(&self) -> String {
        self.inner.keyword().to_string()
    }

    #[wasm_bindgen(js_name=getInstancePath)]
    pub fn instance_path(&self) -> String {
        self.inner.instance_path().to_string()
    }

    #[wasm_bindgen(js_name=getSchemaPath)]
    pub fn schema_path(&self) -> String {
        self.inner.schema_path().to_string()
    }

    #[wasm_bindgen(js_name=getPropertyName)]
    pub fn property_name(&self) -> String {
        self.inner.property_name().to_string()
    }

    #[wasm_bindgen(js_name=getParams)]
    pub fn params(&self) -> Result<JsValue, JsError> {
        let ser = serde_wasm_bindgen::Serializer::json_compatible();

        self.inner.params().serialize(&ser).map_err(|e| e.into())
    }

    #[wasm_bindgen(js_name=getCode)]
    pub fn get_code(&self) -> u32 {
        ConsensusError::from(self.inner.clone()).code()
    }

    #[wasm_bindgen(js_name=toString)]
    pub fn to_string_format(&self) -> String {
        format!("{:#?}", self)
    }

    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.inner.to_string()
    }

    #[wasm_bindgen(js_name=serialize)]
    pub fn serialize(&self) -> Result<Buffer, JsError> {
        let bytes = ConsensusError::from(self.inner.clone())
            .serialize()
            .map_err(JsError::from)?;

        Ok(Buffer::from_bytes(bytes.as_slice()))
    }
}
