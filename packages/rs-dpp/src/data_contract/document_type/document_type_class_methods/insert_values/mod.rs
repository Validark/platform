use crate::data_contract::document_type::document_field::DocumentField;
use crate::data_contract::document_type::DocumentType;
use crate::version::dpp_versions::DocumentTypeVersions;
use crate::ProtocolError;
use platform_value::Value;
use std::collections::{BTreeMap, BTreeSet};

mod v0;

impl DocumentType {
    pub fn insert_values(
        document_properties: &mut BTreeMap<String, DocumentField>,
        known_required: &BTreeSet<String>,
        prefix: Option<String>,
        property_key: String,
        property_value: &Value,
        definition_references: &BTreeMap<String, &Value>,
        document_type_version: &DocumentTypeVersions,
    ) -> Result<(), ProtocolError> {
        match document_type_version.insert_values {
            0 => Self::insert_values_v0(
                document_properties,
                known_required,
                prefix,
                property_key,
                property_value,
                definition_references,
            ),
            version => Err(ProtocolError::UnknownVersionMismatch {
                method: "insert_values".to_string(),
                known_versions: vec![0],
                received: version,
            }),
        }
    }
}
