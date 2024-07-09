use crate::version::fee::data_contract::v1::FEE_DATA_CONTRACT_VALIDATION_VERSION1;
use crate::version::fee::hashing::v2::FEE_HASHING_VERSION2;
use crate::version::fee::processing::v1::FEE_PROCESSING_VERSION1;
use crate::version::fee::signature::v2::FEE_SIGNATURE_VERSION2;
use crate::version::fee::state_transition_min_fees::v1::STATE_TRANSITION_MIN_FEES_VERSION1;
use crate::version::fee::storage::v2::FEE_STORAGE_VERSION2;
use crate::version::fee::FeeVersion;

pub const FEE_VERSION2: FeeVersion = FeeVersion {
    storage: FEE_STORAGE_VERSION2,
    signature: FEE_SIGNATURE_VERSION2,
    hashing: FEE_HASHING_VERSION2,
    processing: FEE_PROCESSING_VERSION1,
    data_contract: FEE_DATA_CONTRACT_VALIDATION_VERSION1,
    state_transition_min_fees: STATE_TRANSITION_MIN_FEES_VERSION1,
};
