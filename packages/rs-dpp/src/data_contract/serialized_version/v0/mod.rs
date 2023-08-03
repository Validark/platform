use crate::data_contract::accessors::v0::DataContractV0Getters;
use crate::data_contract::data_contract_config::DataContractConfig;
use crate::data_contract::document_type::accessors::DocumentTypeV0Getters;
use crate::data_contract::v0::DataContractV0;
use crate::data_contract::{DataContract, DefinitionName, DocumentName};
use crate::identity::state_transition::asset_lock_proof::{Decode, Encode};
use platform_value::{Identifier, Value};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Encode, Decode)]
#[serde(rename_all = "camelCase")]
pub struct DataContractInSerializationFormatV0 {
    /// A unique identifier for the data contract.
    pub id: Identifier,

    /// Internal configuration for the contract.
    pub config: DataContractConfig,

    /// The version of this data contract.
    pub version: u32,

    /// The identifier of the contract owner.
    pub owner_id: Identifier,

    /// Shared subschemas to reuse across documents as $defs object
    pub schema_defs: Option<BTreeMap<DefinitionName, Value>>,

    /// Document JSON Schemas per type
    pub document_schemas: BTreeMap<DocumentName, Value>,
}

impl From<DataContract> for DataContractInSerializationFormatV0 {
    fn from(value: DataContract) -> Self {
        match value {
            DataContract::V0(v0) => {
                let DataContractV0 {
                    id,
                    config,
                    version,
                    owner_id,
                    schema_defs,
                    document_types,
                    ..
                } = v0;

                DataContractInSerializationFormatV0 {
                    id,
                    config,
                    version,
                    owner_id,
                    document_schemas: document_types
                        .into_iter()
                        .map(|(key, r#type)| (key, r#type.schema_owned()))
                        .collect(),
                    schema_defs,
                }
            }
        }
    }
}
