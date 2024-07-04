use bincode::{Decode, Encode};

pub mod v1;

#[derive(Clone, Debug, Encode, Decode, Default)]
pub struct FeeDataContractValidationVersion {
    pub document_type_base_fee: u64,
    pub document_type_size_fee: u64,
    pub document_type_per_property_fee: u64,
    pub document_type_base_non_unique_index_fee: u64,
    pub document_type_non_unique_index_per_property_fee: u64,
    pub document_type_base_unique_index_fee: u64,
    pub document_type_unique_index_per_property_fee: u64,
}

impl PartialEq for FeeDataContractValidationVersion {
    fn eq(&self, other: &Self) -> bool {
        self.document_type_base_fee == other.document_type_base_fee
            && self.document_type_size_fee == other.document_type_size_fee
            && self.document_type_per_property_fee == other.document_type_per_property_fee
            && self.document_type_base_non_unique_index_fee
                == other.document_type_base_non_unique_index_fee
            && self.document_type_non_unique_index_per_property_fee
                == other.document_type_non_unique_index_per_property_fee
            && self.document_type_base_unique_index_fee == other.document_type_base_unique_index_fee
            && self.document_type_unique_index_per_property_fee
                == other.document_type_unique_index_per_property_fee
    }
}
