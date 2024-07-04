use bincode::{Decode, Encode};

pub mod v1;

#[derive(Clone, Debug, Encode, Decode, Default)]
pub struct FeeProcessingVersion {
    pub fetch_identity_balance_processing_cost: u64,
    pub fetch_identity_revision_processing_cost: u64,
    pub fetch_identity_balance_and_revision_processing_cost: u64,
    pub fetch_identity_cost_per_look_up_key_by_id: u64,
    pub fetch_single_identity_key_processing_cost: u64,
    pub validate_key_structure: u64,
}

impl PartialEq for FeeProcessingVersion {
    fn eq(&self, other: &Self) -> bool {
        self.fetch_identity_balance_processing_cost == other.fetch_identity_balance_processing_cost
            && self.fetch_identity_revision_processing_cost
                == other.fetch_identity_revision_processing_cost
            && self.fetch_identity_balance_and_revision_processing_cost
                == other.fetch_identity_balance_and_revision_processing_cost
            && self.fetch_identity_cost_per_look_up_key_by_id
                == other.fetch_identity_cost_per_look_up_key_by_id
            && self.fetch_single_identity_key_processing_cost
                == other.fetch_single_identity_key_processing_cost
            && self.validate_key_structure == other.validate_key_structure
    }
}
