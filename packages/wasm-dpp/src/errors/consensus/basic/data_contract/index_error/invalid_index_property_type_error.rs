use crate::buffer::Buffer;
use dpp::consensus::basic::data_contract::InvalidIndexPropertyTypeError;
use dpp::consensus::codes::ErrorWithCode;
use dpp::consensus::ConsensusError;
use dpp::serialization::serialization_traits::PlatformSerializable;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name=InvalidIndexPropertyTypeError)]
pub struct InvalidIndexPropertyTypeErrorWasm {
    inner: InvalidIndexPropertyTypeError,
}

impl From<&InvalidIndexPropertyTypeError> for InvalidIndexPropertyTypeErrorWasm {
    fn from(e: &InvalidIndexPropertyTypeError) -> Self {
        Self { inner: e.clone() }
    }
}

#[wasm_bindgen(js_class=InvalidIndexPropertyTypeError)]
impl InvalidIndexPropertyTypeErrorWasm {
    #[wasm_bindgen(js_name=getDocumentType)]
    pub fn get_document_type(&self) -> String {
        self.inner.document_type().to_string()
    }

    #[wasm_bindgen(js_name=getIndexName)]
    pub fn get_index_name(&self) -> String {
        self.inner.index_name().to_string()
    }

    #[wasm_bindgen(js_name=getPropertyName)]
    pub fn get_property_name(&self) -> String {
        self.inner.property_name().to_string()
    }

    #[wasm_bindgen(js_name=getPropertyType)]
    pub fn get_property_type(&self) -> String {
        self.inner.property_type().to_string()
    }

    #[wasm_bindgen(js_name=getCode)]
    pub fn get_code(&self) -> u32 {
        ConsensusError::from(self.inner.clone()).code()
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
