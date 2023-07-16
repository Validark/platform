use crate::version::{FeatureVersion, FeatureVersionBounds};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub struct DPPVersion {
    pub state_transition_serialization_versions: StateTransitionSerializationVersions,
    pub contract_versions: ContractVersions,
    pub document_versions: DocumentVersions,
    pub identity_versions: IdentityVersions,
}

#[derive(Clone, Debug, Default)]
pub struct StateTransitionSerializationVersions {
    pub identity_create_state_transition: FeatureVersionBounds,
    pub identity_update_state_transition: FeatureVersionBounds,
    pub identity_top_up_state_transition: FeatureVersionBounds,
    pub identity_credit_withdrawal_state_transition: FeatureVersionBounds,
    pub identity_credit_transfer_state_transition: FeatureVersionBounds,
    pub contract_create_state_transition: FeatureVersionBounds,
    pub contract_update_state_transition: FeatureVersionBounds,
    pub documents_batch_state_transition: FeatureVersionBounds,
    pub document_base_state_transition: FeatureVersionBounds,
    pub document_create_state_transition: DocumentFeatureVersionBounds,
    pub document_replace_state_transition: DocumentFeatureVersionBounds,
    pub document_delete_state_transition: DocumentFeatureVersionBounds,
}

#[derive(Clone, Debug, Default)]
pub struct DocumentFeatureVersionBounds {
    pub bounds: FeatureVersionBounds,
}

#[derive(Clone, Debug, Default)]
pub struct ContractVersions {
    /// This is how we serialize and deserialize a contract
    pub contract_serialization_version: FeatureVersionBounds,
    /// This is the structure of the Contract as it is defined for code paths
    pub contract_structure_version: FeatureVersion,
    pub created_data_contract_structure_version: FeatureVersion,
    pub document_type_versions: DocumentTypeVersions,
    pub contract_class_method_versions: ContractClassMethodVersions,
}

#[derive(Clone, Debug, Default)]
pub struct ContractClassMethodVersions {
    pub get_property_definition_by_path: FeatureVersion,
    pub get_binary_properties_from_schema: FeatureVersion,
    pub get_definitions: FeatureVersion,
    pub get_document_types_from_contract: FeatureVersion,
    pub get_document_types_from_value: FeatureVersion,
    pub get_document_types_from_value_array: FeatureVersion,
}

#[derive(Clone, Debug, Default)]
pub struct DocumentTypeVersions {
    /// This is for the overall structure of the document type, like DocumentTypeV0
    pub document_type_structure_version: FeatureVersion,
    /// Class based method
    pub find_identifier_and_binary_paths: FeatureVersion,
    /// Class based method
    pub insert_values: FeatureVersion,
    /// Class based method
    pub insert_values_nested: FeatureVersion,
    pub index_for_types: FeatureVersion,
    pub unique_id_for_storage: FeatureVersion,
    pub unique_id_for_document_field: FeatureVersion,
    pub serialize_value_for_key: FeatureVersion,
    pub convert_value_to_document: FeatureVersion,
    pub max_size: FeatureVersion,
    pub estimated_size: FeatureVersion,
    pub top_level_indices: FeatureVersion,
    pub document_field_for_property: FeatureVersion,
    pub document_field_type_for_property: FeatureVersion,
    pub field_can_be_null: FeatureVersion,
    pub initial_revision: FeatureVersion,
    pub requires_revision: FeatureVersion,
}

#[derive(Clone, Debug, Default)]
pub struct IdentityVersions {
    /// This is the structure of the Identity as it is defined for code paths
    pub identity_structure_version: FeatureVersion,
    pub identity_key_structure_version: FeatureVersion,
}

#[derive(Clone, Debug, Default)]
pub struct DocumentVersions {
    // This is for the overall structure of the document, like DocumentV0
    pub document_structure_version: FeatureVersion,
    pub document_serialization_version: FeatureVersionBounds,
    pub extended_document_structure_version: FeatureVersionBounds,
    pub document_method_versions: DocumentMethodVersions,
    pub document_class_method_versions: DocumentClassMethodVersions,
}

#[derive(Clone, Debug, Default)]
pub struct DocumentClassMethodVersions {
    pub get_identifiers_and_binary_paths: FeatureVersion,
}

#[derive(Clone, Debug, Default)]
pub struct DocumentMethodVersions {
    pub hash: FeatureVersion,
    pub get_raw_for_contract: FeatureVersion,
    pub get_raw_for_document_type: FeatureVersion,
}
