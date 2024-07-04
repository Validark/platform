use crate::version::fee::data_contract::FeeDataContractValidationVersion;
use crate::version::fee::hashing::FeeHashingVersion;
use crate::version::fee::processing::FeeProcessingVersion;
use crate::version::fee::signature::FeeSignatureVersion;
use crate::version::fee::state_transition_min_fees::StateTransitionMinFees;
use crate::version::fee::storage::FeeStorageVersion;
use bincode::{Decode, Encode};

mod data_contract;
mod hashing;
mod processing;
pub mod signature;
pub mod state_transition_min_fees;
pub mod storage;
pub mod v1;

#[derive(Clone, Debug, Encode, Decode, Default)]
pub struct FeeVersion {
    pub storage: FeeStorageVersion,
    pub signature: FeeSignatureVersion,
    pub hashing: FeeHashingVersion,
    pub processing: FeeProcessingVersion,
    pub data_contract: FeeDataContractValidationVersion,
    pub state_transition_min_fees: StateTransitionMinFees,
}

impl PartialEq for FeeVersion {
    fn eq(&self, other: &Self) -> bool {
        self.storage == other.storage
            && self.signature == other.signature
            && self.hashing == other.hashing
            && self.processing == other.processing
            && self.data_contract == other.data_contract
            && self.state_transition_min_fees == other.state_transition_min_fees
    }
}
