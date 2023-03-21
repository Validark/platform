use dpp::prelude::Identifier;
use wasm_bindgen::prelude::*;

use crate::buffer::Buffer;
use crate::identifier::IdentifierWrapper;

#[wasm_bindgen(js_name=InvalidDocumentTypeError)]
pub struct InvalidDocumentTypeErrorWasm {
    document_type: String,
    data_contract_id: Identifier,
    code: u32,
}

impl InvalidDocumentTypeErrorWasm {
    pub fn new(document_type: String, data_contract_id: Identifier, code: u32) -> Self {
        InvalidDocumentTypeErrorWasm {
            document_type,
            data_contract_id,
            code,
        }
    }
}

#[wasm_bindgen(js_class=InvalidDocumentTypeError)]
impl InvalidDocumentTypeErrorWasm {
    #[wasm_bindgen(constructor)]
    pub fn constructor(
        document_type: String,
        data_contract_id: IdentifierWrapper,
        code: u32,
    ) -> Self {
        Self {
            document_type,
            data_contract_id: data_contract_id.into(),
            code,
        }
    }

    #[wasm_bindgen(js_name=getType)]
    pub fn get_document_type(&self) -> String {
        self.document_type.clone()
    }

    #[wasm_bindgen(js_name=getDataContractId)]
    pub fn get_data_contract_id(&self) -> Buffer {
        Buffer::from_bytes(self.data_contract_id.as_bytes())
    }

    #[wasm_bindgen(js_name=getCode)]
    pub fn get_code(&self) -> u32 {
        self.code
    }
}
