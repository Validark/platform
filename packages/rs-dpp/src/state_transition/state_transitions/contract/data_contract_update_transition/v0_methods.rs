use platform_value::{Bytes32, Identifier};
use crate::data_contract::DataContract;
use crate::identity::{KeyID, PartialIdentity};
use crate::identity::signer::Signer;
use crate::ProtocolError;
use crate::state_transition::data_contract_update_transition::{DataContractUpdateTransition, DataContractUpdateTransitionV0};
use crate::state_transition::data_contract_update_transition::v0::v0_methods::DataContractUpdateTransitionV0Methods;
use crate::version::FeatureVersion;

impl DataContractUpdateTransitionV0Methods for DataContractUpdateTransition {
    fn new_from_data_contract<S: Signer>(data_contract: DataContract, identity: &PartialIdentity, key_id: KeyID, signer: &S, version: FeatureVersion) -> Result<DataContractUpdateTransition, ProtocolError> {
        match version { 0 => {
            DataContractUpdateTransitionV0::new_from_data_contract(data_contract, identity, key_id, signer, version)
        }
            v => Err(ProtocolError::UnknownVersionError(format!(
                "Unknown DataContractUpdateTransition version for new_from_data_contract {v}"
            ))),
        }
    }

    fn get_data_contract(&self) -> &DataContract {
        match self { DataContractUpdateTransition::V0(transition) =>transition.get_data_contract() }
    }

    fn set_data_contract(&mut self, data_contract: DataContract) {
        match self { DataContractUpdateTransition::V0(transition) =>transition.set_data_contract(data_contract) }
    }
}