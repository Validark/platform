use std::sync::Arc;
use serde::{Deserialize, Serialize};
use dpp::platform_value::Identifier;
use dpp::data_contract::DataContract;
use dpp::ProtocolError;
use dpp::state_transition::documents_batch_transition::document_base_transition::v0::DocumentBaseTransitionV0;
use crate::drive::contract::DataContractFetchInfo;
use crate::state_transition_action::document::documents_batch::document_transition::document_base_transition_action::DocumentBaseTransitionActionV0;

impl DocumentBaseTransitionActionV0 {
    pub fn try_from_base_transition_with_contract_lookup(
        value: DocumentBaseTransitionV0,
        get_data_contract: impl Fn(Identifier) -> Result<Arc<DataContractFetchInfo>, ProtocolError>,
    ) -> Result<Self, ProtocolError> {
        let DocumentBaseTransitionV0 {
            id,
            document_type_name,
            data_contract_id,
            ..
        } = value;
        Ok(DocumentBaseTransitionActionV0 {
            id,
            document_type_name,
            data_contract_id,
            data_contract: get_data_contract(data_contract_id)?,
        })
    }

    pub fn try_from_borrowed_base_transition_with_contract_lookup(
        value: &DocumentBaseTransitionV0,
        get_data_contract: impl Fn(Identifier) -> Result<Arc<DataContractFetchInfo>, ProtocolError>,
    ) -> Result<Self, ProtocolError> {
        let DocumentBaseTransitionV0 {
            id,
            document_type_name,
            data_contract_id,
            ..
        } = value;
        Ok(DocumentBaseTransitionActionV0 {
            id: *id,
            document_type_name: document_type_name.clone(),
            data_contract_id: *data_contract_id,
            data_contract: get_data_contract(*data_contract_id)?,
        })
    }
}