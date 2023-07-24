use crate::consensus::basic::document::InvalidDocumentTypeError;
use crate::data_contract::base::DataContractBaseMethodsV0;
use crate::data_contract::document_type::accessors::DocumentTypeV0Getters;
use crate::data_contract::document_type::v0::v0_methods::DocumentTypeV0Methods;
use crate::data_contract::errors::DataContractError;
use crate::data_contract::DataContract;
use crate::document::{Document, INITIAL_REVISION};
use crate::identity::TimestampMillis;
use crate::util::entropy_generator::{DefaultEntropyGenerator, EntropyGenerator};
use crate::version::{PlatformVersion, LATEST_PLATFORM_VERSION};
use crate::ProtocolError;
use chrono::Utc;
use platform_value::{Identifier, Value};

#[cfg(feature = "extended-document")]
use crate::document::ExtendedDocument;

const PROPERTY_FEATURE_VERSION: &str = "$version";
const PROPERTY_ENTROPY: &str = "$entropy";
const PROPERTY_ACTION: &str = "$action";
const PROPERTY_OWNER_ID: &str = "ownerId";
const PROPERTY_DOCUMENT_OWNER_ID: &str = "$ownerId";
const PROPERTY_TYPE: &str = "$type";
const PROPERTY_ID: &str = "$id";
const PROPERTY_TRANSITIONS: &str = "transitions";
const PROPERTY_DATA_CONTRACT_ID: &str = "$dataContractId";
const PROPERTY_REVISION: &str = "$revision";
const PROPERTY_CREATED_AT: &str = "$createdAt";
const PROPERTY_UPDATED_AT: &str = "$updatedAt";
const PROPERTY_DOCUMENT_TYPE: &str = "$type";

const DOCUMENT_CREATE_KEYS_TO_STAY: [&str; 5] = [
    PROPERTY_ID,
    PROPERTY_TYPE,
    PROPERTY_DATA_CONTRACT_ID,
    PROPERTY_CREATED_AT,
    PROPERTY_UPDATED_AT,
];

const DOCUMENT_REPLACE_KEYS_TO_STAY: [&str; 5] = [
    PROPERTY_ID,
    PROPERTY_TYPE,
    PROPERTY_DATA_CONTRACT_ID,
    PROPERTY_REVISION,
    PROPERTY_UPDATED_AT,
];

/// Factory for creating documents
pub struct DocumentFactoryV0 {
    protocol_version: u32,
    pub(super) data_contract: DataContract,
    entropy_generator: Box<dyn EntropyGenerator>,
}

impl DocumentFactoryV0 {
    pub fn new(protocol_version: u32, data_contract: DataContract) -> Self {
        DocumentFactoryV0 {
            protocol_version,
            data_contract,
            entropy_generator: Box::new(DefaultEntropyGenerator),
        }
    }

    pub fn new_with_entropy_generator(
        protocol_version: u32,
        data_contract: DataContract,
        entropy_generator: Box<dyn EntropyGenerator>,
    ) -> Self {
        DocumentFactoryV0 {
            protocol_version,
            data_contract,
            entropy_generator,
        }
    }

    pub fn create_document(
        &self,
        owner_id: Identifier,
        document_type_name: String,
        data: Value,
    ) -> Result<Document, ProtocolError> {
        let platform_version = PlatformVersion::get(self.protocol_version)?;
        if !self.data_contract.is_document_defined(&document_type_name) {
            return Err(DataContractError::InvalidDocumentTypeError(
                InvalidDocumentTypeError::new(document_type_name, self.data_contract.id()),
            )
            .into());
        }

        let document_entropy = self.entropy_generator.generate()?;

        let document_type = self
            .data_contract
            .document_type_for_name(document_type_name.as_str())?;

        document_type.create_document_from_data(data, owner_id, document_entropy, platform_version)
    }

    #[cfg(feature = "extended-document")]
    pub fn create_extended_document(
        &self,
        owner_id: Identifier,
        document_type_name: String,
        data: Value,
    ) -> Result<ExtendedDocument, ProtocolError> {
        let platform_version = PlatformVersion::get(self.protocol_version)?;
        if !self.data_contract.is_document_defined(&document_type_name) {
            return Err(DataContractError::InvalidDocumentTypeError(
                InvalidDocumentTypeError::new(document_type_name, self.data_contract.id()),
            )
            .into());
        }

        let document_entropy = self.entropy_generator.generate()?;

        let document_type = self
            .data_contract
            .document_type_for_name(document_type_name.as_str())?;

        let document = document_type.create_document_from_data(data, owner_id, document_entropy)?;

        let extended_document = ExtendedDocument {
            feature_version: LATEST_PLATFORM_VERSION
                .extended_document
                .default_current_version,
            document_type_name,
            data_contract_id: data_contract.id,
            document,
            data_contract,
            metadata: None,
            entropy: Bytes32::new(document_entropy),
        };

        Ok(extended_document)
    }
    //
    // pub fn create_state_transition(
    //     &self,
    //     documents_iter: impl IntoIterator<Item = (Action, Vec<ExtendedDocument>)>,
    // ) -> Result<DocumentsBatchTransition, ProtocolError> {
    //     let mut raw_documents_transitions: Vec<Value> = vec![];
    //     let mut data_contracts: Vec<DataContract> = vec![];
    //     let documents: Vec<(Action, Vec<ExtendedDocument>)> = documents_iter.into_iter().collect();
    //     let flattened_documents_iter = documents.iter().flat_map(|(_, v)| v);
    //
    //     if Self::is_empty(flattened_documents_iter.clone()) {
    //         return Err(DocumentError::NoDocumentsSuppliedError.into());
    //     }
    //
    //     let is_the_same = Self::is_ownership_the_same(
    //         flattened_documents_iter
    //             .clone()
    //             .map(|extended_document| &extended_document.document.owner_id),
    //     );
    //     if !is_the_same {
    //         return Err(DocumentError::MismatchOwnerIdsError {
    //             documents: documents.into_iter().flat_map(|(_, v)| v).collect(),
    //         }
    //             .into());
    //     }
    //
    //     let owner_id = flattened_documents_iter
    //         .clone()
    //         .next()
    //         .unwrap()
    //         .owner_id()
    //         .to_owned();
    //     for (action, documents) in documents {
    //         data_contracts.extend(documents.iter().map(|d| d.data_contract().clone()));
    //
    //         let raw_transitions = match action {
    //             Action::Create => Self::raw_document_create_transitions(documents)?,
    //             Action::Delete => Self::raw_document_delete_transitions(documents)?,
    //             Action::Replace => Self::raw_document_replace_transitions(documents)?,
    //         };
    //
    //         raw_documents_transitions.extend(raw_transitions);
    //     }
    //
    //     if raw_documents_transitions.is_empty() {
    //         return Err(DocumentError::NoDocumentsSuppliedError.into());
    //     }
    //
    //     let raw_batch_transition = BTreeMap::from([
    //         (
    //             PROPERTY_FEATURE_VERSION.to_string(),
    //             Value::U16(LATEST_PLATFORM_VERSION.document.default_current_version),
    //         ),
    //         (
    //             PROPERTY_OWNER_ID.to_string(),
    //             Value::Identifier(owner_id.to_buffer()),
    //         ),
    //         (
    //             PROPERTY_TRANSITIONS.to_string(),
    //             Value::Array(raw_documents_transitions),
    //         ),
    //     ]);
    //
    //     DocumentsBatchTransition::from_value_map(raw_batch_transition, data_contracts)
    // }
    //
    // pub fn create_extended_from_document_buffer(
    //     &self,
    //     buffer: &[u8],
    //     document_type: &str,
    //     data_contract: &DataContract,
    //     platform_version: &PlatformVersion,
    // ) -> Result<ExtendedDocument, ProtocolError> {
    //     let document_type = data_contract.document_types.get(document_type).ok_or(
    //         ProtocolError::DataContractError(DataContractError::DocumentTypeNotFound(
    //             "document type was not found in the data contract",
    //         )),
    //     )?;
    //
    //     let document = Document::from_bytes(buffer, document_type, platform_version)?;
    //
    //     Ok(ExtendedDocument {
    //         protocol_version: data_contract.protocol_version,
    //         document_type_name: document_type.name.clone(),
    //         data_contract_id: data_contract.id,
    //         document,
    //         data_contract: data_contract.clone(),
    //         metadata: None,
    //         entropy: Bytes32::default(),
    //     })
    // }
    //
    // pub fn create_from_buffer(
    //     &self,
    //     buffer: impl AsRef<[u8]>,
    // ) -> Result<ExtendedDocument, ProtocolError> {
    //     let document = <ExtendedDocument as PlatformDeserializable>::deserialize(buffer.as_ref())
    //         .map_err(|e| {
    //             ConsensusError::BasicError(BasicError::SerializedObjectParsingError(
    //                 SerializedObjectParsingError::new(format!("Decode protocol entity: {:#?}", e)),
    //             ))
    //         })?;
    //     self.create_from_object(document.to_value()?).await
    // }
    //
    // pub fn create_from_object(
    //     &self,
    //     raw_document: Value,
    // ) -> Result<ExtendedDocument, ProtocolError> {
    //     ExtendedDocument::from_untrusted_platform_value(raw_document, data_contract)
    // }
    // //
    // // async fn validate_data_contract_for_extended_document(
    // //     &self,
    // //     raw_document: &Value,
    // //     options: FactoryOptions,
    // // ) -> Result<DataContract, ProtocolError> {
    // //     let result = self
    // //         .data_contract_fetcher_and_validator
    // //         .validate_extended(raw_document)
    // //         .await?;
    // //
    // //     if !result.is_valid() {
    // //         return Err(ProtocolError::Document(Box::new(
    // //             DocumentError::InvalidDocumentError {
    // //                 errors: result.errors,
    // //                 raw_document: raw_document.clone(),
    // //             },
    // //         )));
    // //     }
    // //     let data_contract = result
    // //         .into_data()
    // //         .context("Validator didn't return Data Contract. This shouldn't happen")?;
    // //
    // //     if !options.skip_validation {
    // //         let result = self
    // //             .document_validator
    // //             .validate_extended(raw_document, &data_contract)?;
    // //         if !result.is_valid() {
    // //             return Err(ProtocolError::Document(Box::new(
    // //                 DocumentError::InvalidDocumentError {
    // //                     errors: result.errors,
    // //                     raw_document: raw_document.clone(),
    // //                 },
    // //             )));
    // //         }
    // //     }
    // //
    // //     Ok(data_contract)
    // // }
    //
    // fn raw_document_create_transitions(
    //     documents: Vec<ExtendedDocument>,
    // ) -> Result<Vec<Value>, ProtocolError> {
    //     let mut raw_transitions = vec![];
    //     for document in documents {
    //         if document.needs_revision()? {
    //             let Some(revision) = document.revision() else {
    //                 return Err(DocumentError::RevisionAbsentError {
    //                     document: Box::new(document),
    //                 }.into());
    //             };
    //             if revision != &INITIAL_REVISION {
    //                 return Err(DocumentError::InvalidInitialRevisionError {
    //                     document: Box::new(document),
    //                 }
    //                     .into());
    //             }
    //         }
    //         let mut map = document.to_map_value()?;
    //
    //         map.retain(|key, _| {
    //             !key.starts_with('$') || DOCUMENT_CREATE_KEYS_TO_STAY.contains(&key.as_str())
    //         });
    //         map.insert(PROPERTY_ACTION.to_string(), Value::U8(Action::Create as u8));
    //         map.insert(
    //             PROPERTY_ENTROPY.to_string(),
    //             Value::Bytes(document.entropy.to_vec()),
    //         );
    //         raw_transitions.push(map.into());
    //     }
    //
    //     Ok(raw_transitions)
    // }
    //
    // fn raw_document_replace_transitions(
    //     documents: Vec<ExtendedDocument>,
    // ) -> Result<Vec<Value>, ProtocolError> {
    //     let mut raw_transitions = vec![];
    //     for document in documents {
    //         if !document.can_be_modified()? {
    //             return Err(DocumentError::TryingToReplaceImmutableDocument {
    //                 document: Box::new(document),
    //             }
    //                 .into());
    //         }
    //         let Some(document_revision) = document.revision() else {
    //             return Err(DocumentError::RevisionAbsentError {
    //                 document: Box::new(document),
    //             }.into());
    //         };
    //         let mut map = document.to_map_value()?;
    //
    //         map.retain(|key, _| {
    //             !key.starts_with('$') || DOCUMENT_REPLACE_KEYS_TO_STAY.contains(&key.as_str())
    //         });
    //         map.insert(
    //             PROPERTY_ACTION.to_string(),
    //             Value::U8(Action::Replace as u8),
    //         );
    //         let new_revision = document_revision + 1;
    //         map.insert(PROPERTY_REVISION.to_string(), Value::U64(new_revision));
    //
    //         // If document have an originally set `updatedAt`
    //         // we should update it then
    //         let contains_updated_at = document
    //             .document_type()?
    //             .required_fields
    //             .contains(PROPERTY_UPDATED_AT);
    //
    //         if contains_updated_at {
    //             let now = Utc::now().timestamp_millis() as TimestampMillis;
    //             map.insert(PROPERTY_UPDATED_AT.to_string(), Value::U64(now));
    //         }
    //
    //         raw_transitions.push(map.into());
    //     }
    //     Ok(raw_transitions)
    // }
    //
    // fn raw_document_delete_transitions(
    //     documents: Vec<ExtendedDocument>,
    // ) -> Result<Vec<Value>, ProtocolError> {
    //     Ok(documents
    //         .into_iter()
    //         .map(|document| {
    //             let mut map: BTreeMap<String, Value> = BTreeMap::new();
    //             map.insert(PROPERTY_ACTION.to_string(), Value::U8(Action::Delete as u8));
    //             map.insert(PROPERTY_ID.to_string(), document.document().id.into());
    //             map.insert(
    //                 PROPERTY_TYPE.to_string(),
    //                 Value::Text(document.document_type_name().clone()),
    //             );
    //             map.insert(
    //                 PROPERTY_DATA_CONTRACT_ID.to_string(),
    //                 document.data_contract_id().into(),
    //             );
    //             map.into()
    //         })
    //         .collect())
    // }
    //
    // fn is_empty<T>(data: impl IntoIterator<Item = T>) -> bool {
    //     data.into_iter().next().is_none()
    // }
    //
    // fn is_ownership_the_same<'a>(ids: impl IntoIterator<Item = &'a Identifier>) -> bool {
    //     ids.into_iter().all_equal()
    // }
}