use crate::buffer::Buffer;

use dpp::consensus::codes::ErrorWithCode;
use dpp::consensus::state::data_trigger::data_trigger_invalid_result_error::DataTriggerInvalidResultError;
use dpp::consensus::ConsensusError;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name=DataTriggerInvalidResultError)]
pub struct DataTriggerInvalidResultErrorWasm {
    inner: DataTriggerInvalidResultError,
}

impl From<&DataTriggerInvalidResultError> for DataTriggerInvalidResultErrorWasm {
    fn from(e: &DataTriggerInvalidResultError) -> Self {
        Self { inner: e.clone() }
    }
}

#[wasm_bindgen(js_class=DataTriggerInvalidResultError)]
impl DataTriggerInvalidResultErrorWasm {
    #[wasm_bindgen(js_name=getDataContractId)]
    pub fn data_contract_id(&self) -> Buffer {
        Buffer::from_bytes(self.inner.data_contract_id().as_bytes())
    }

    #[wasm_bindgen(js_name=getDocumentId)]
    pub fn document_id(&self) -> Buffer {
        Buffer::from_bytes(self.inner.document_id().as_bytes())
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
            .map_err(|e| JsError::from(e))?;

        Ok(Buffer::from_bytes(bytes.as_slice()))
    }
}
