use dpp::data_contract::document_type::IndexLevel;

use grovedb::batch::key_info::KeyInfo;
use grovedb::batch::key_info::KeyInfo::KnownKey;
use grovedb::batch::KeyInfoPath;
use grovedb::reference_path::ReferencePathType::SiblingReference;
use grovedb::EstimatedLayerCount::{ApproximateElements, PotentiallyAtMaxElements};
use grovedb::EstimatedLayerSizes::AllSubtrees;
use grovedb::EstimatedSumTrees::NoSumTrees;
use grovedb::{Element, EstimatedLayerInformation, TransactionArg};
use std::borrow::Cow;
use std::collections::HashMap;
use std::option::Option::None;

use crate::drive::defaults::{DEFAULT_HASH_SIZE_U8, STORAGE_FLAGS_SIZE};
use crate::drive::document::{
    contract_document_type_path_vec,
    contract_documents_keeping_history_primary_key_path_for_document_id,
    contract_documents_keeping_history_primary_key_path_for_unknown_document_id,
    contract_documents_keeping_history_storage_time_reference_path_size,
    contract_documents_primary_key_path, document_reference_size, make_document_reference,
    unique_event_id,
};
use crate::drive::flags::StorageFlags;
use crate::drive::object_size_info::DocumentInfo::{
    DocumentAndSerialization, DocumentEstimatedAverageSize, DocumentOwnedInfo,
    DocumentRefAndSerialization, DocumentRefInfo,
};
use crate::drive::object_size_info::DriveKeyInfo::{Key, KeyRef};
use crate::drive::object_size_info::KeyElementInfo::{KeyElement, KeyUnknownElementSize};
use crate::drive::object_size_info::PathKeyElementInfo::{
    PathFixedSizeKeyRefElement, PathKeyUnknownElementSize,
};
use crate::drive::object_size_info::PathKeyInfo::{PathFixedSizeKeyRef, PathKeySize};
use crate::drive::object_size_info::{
    DocumentAndContractInfo, OwnedDocumentInfo, PathInfo, PathKeyElementInfo,
};
use crate::drive::Drive;
use crate::error::drive::DriveError;
use crate::error::Error;
use crate::fee::calculate_fee;
use crate::fee::op::LowLevelDriveOperation;
use dpp::data_contract::DataContract;

use crate::drive::grove_operations::DirectQueryType::{StatefulDirectQuery, StatelessDirectQuery};
use crate::drive::grove_operations::QueryTarget::QueryTargetValue;
use crate::drive::grove_operations::{BatchInsertApplyType, BatchInsertTreeApplyType};
use crate::error::document::DocumentError;
use crate::error::fee::FeeError;
use crate::fee::result::FeeResult;
use dpp::block::extended_block_info::BlockInfo;
use dpp::data_contract::document_type::document_field::v0::{
    encode_unsigned_integer, DocumentFieldType,
};
use dpp::document::Document;
use dpp::prelude::Identifier;
use dpp::version::drive_versions::DriveVersion;

impl Drive {
    /// Adds a document to primary storage.
    /// If a document isn't sent to this function then we are just calling to know the query and
    /// insert operations
    pub(super) fn add_document_to_primary_storage_0(
        &self,
        document_and_contract_info: &DocumentAndContractInfo,
        block_info: &BlockInfo,
        insert_without_check: bool,
        estimated_costs_only_with_layer_info: &mut Option<
            HashMap<KeyInfoPath, EstimatedLayerInformation>,
        >,
        transaction: TransactionArg,
        drive_operations: &mut Vec<LowLevelDriveOperation>,
        drive_version: &DriveVersion,
    ) -> Result<(), Error> {
        let contract = document_and_contract_info.contract;
        let document_type = document_and_contract_info.document_type;
        let primary_key_path = contract_documents_primary_key_path(
            contract.id().as_bytes(),
            document_type.name.as_str(),
        );
        // if we are trying to get estimated costs we should add this level
        if let Some(estimated_costs_only_with_layer_info) = estimated_costs_only_with_layer_info {
            Self::add_estimation_costs_for_add_document_to_primary_storage(
                document_and_contract_info,
                primary_key_path,
                estimated_costs_only_with_layer_info,
            );
        }

        if document_type.documents_keep_history {
            let (path_key_info, storage_flags) = if document_and_contract_info
                .owned_document_info
                .document_info
                .is_document_size()
            {
                (
                    PathKeySize(
                        KeyInfoPath::from_known_path(primary_key_path),
                        KeyInfo::MaxKeySize {
                            unique_id: document_type.unique_id_for_storage().to_vec(),
                            max_size: DEFAULT_HASH_SIZE_U8,
                        },
                    ),
                    StorageFlags::optional_default_as_ref(),
                )
            } else {
                let inserted_storage_flags = if contract.config.can_be_deleted {
                    document_and_contract_info
                        .owned_document_info
                        .document_info
                        .get_storage_flags_ref()
                } else {
                    // there are no need for storage flags if the contract can not be deleted
                    // as this tree can never be deleted
                    None
                };
                (
                    PathFixedSizeKeyRef((
                        primary_key_path,
                        document_and_contract_info
                            .owned_document_info
                            .document_info
                            .get_document_id_as_slice()
                            .ok_or(Error::Drive(DriveError::CorruptedCodeExecution(
                                "can not get document id from estimated document",
                            )))?,
                    )),
                    inserted_storage_flags,
                )
            };
            let apply_type = if estimated_costs_only_with_layer_info.is_none() {
                BatchInsertTreeApplyType::StatefulBatchInsertTree
            } else {
                BatchInsertTreeApplyType::StatelessBatchInsertTree {
                    in_tree_using_sums: false,
                    is_sum_tree: false,
                    flags_len: storage_flags
                        .map(|s| s.serialized_size())
                        .unwrap_or_default(),
                }
            };
            // we first insert an empty tree if the document is new
            self.batch_insert_empty_tree_if_not_exists(
                path_key_info,
                storage_flags,
                apply_type,
                transaction,
                &mut None, //not going to have multiple same documents in same batch
                drive_operations,
                drive_version,
            )?;
            let encoded_time = DocumentFieldType::encode_unsigned_integer(block_info.time_ms)?;
            let path_key_element_info = match &document_and_contract_info
                .owned_document_info
                .document_info
            {
                DocumentRefAndSerialization((document, serialized_document, storage_flags)) => {
                    let element = Element::Item(
                        serialized_document.to_vec(),
                        StorageFlags::map_borrowed_cow_to_some_element_flags(storage_flags),
                    );
                    let document_id_in_primary_path =
                        contract_documents_keeping_history_primary_key_path_for_document_id(
                            contract.id().as_bytes(),
                            document_type.name.as_str(),
                            document.id.as_slice(),
                        );
                    PathFixedSizeKeyRefElement((
                        document_id_in_primary_path,
                        encoded_time.as_slice(),
                        element,
                    ))
                }
                DocumentAndSerialization((document, serialized_document, storage_flags)) => {
                    let element = Element::Item(
                        serialized_document.to_vec(),
                        StorageFlags::map_borrowed_cow_to_some_element_flags(storage_flags),
                    );
                    let document_id_in_primary_path =
                        contract_documents_keeping_history_primary_key_path_for_document_id(
                            contract.id().as_bytes(),
                            document_type.name.as_str(),
                            document.id.as_slice(),
                        );
                    PathFixedSizeKeyRefElement((
                        document_id_in_primary_path,
                        encoded_time.as_slice(),
                        element,
                    ))
                }
                DocumentOwnedInfo((document, storage_flags)) => {
                    let serialized_document =
                        document.serialize(document_and_contract_info.document_type)?;
                    let element = Element::Item(
                        serialized_document,
                        StorageFlags::map_borrowed_cow_to_some_element_flags(storage_flags),
                    );
                    let document_id_in_primary_path =
                        contract_documents_keeping_history_primary_key_path_for_document_id(
                            contract.id().as_bytes(),
                            document_type.name.as_str(),
                            document.id.as_slice(),
                        );
                    PathFixedSizeKeyRefElement((
                        document_id_in_primary_path,
                        encoded_time.as_slice(),
                        element,
                    ))
                }
                DocumentRefInfo((document, storage_flags)) => {
                    let serialized_document =
                        document.serialize(document_and_contract_info.document_type)?;
                    let element = Element::Item(
                        serialized_document,
                        StorageFlags::map_borrowed_cow_to_some_element_flags(storage_flags),
                    );
                    let document_id_in_primary_path =
                        contract_documents_keeping_history_primary_key_path_for_document_id(
                            contract.id().as_bytes(),
                            document_type.name.as_str(),
                            document.id.as_slice(),
                        );
                    PathFixedSizeKeyRefElement((
                        document_id_in_primary_path,
                        encoded_time.as_slice(),
                        element,
                    ))
                }
                DocumentEstimatedAverageSize(max_size) => {
                    let document_id_in_primary_path =
                        contract_documents_keeping_history_primary_key_path_for_unknown_document_id(
                            contract.id().as_bytes(),
                            document_type,
                        );
                    PathKeyUnknownElementSize((
                        document_id_in_primary_path,
                        KnownKey(encoded_time.clone()),
                        Element::required_item_space(*max_size, STORAGE_FLAGS_SIZE),
                    ))
                }
            };
            self.batch_insert(path_key_element_info, drive_operations, drive_version)?;
            let path_key_element_info = if document_and_contract_info
                .owned_document_info
                .document_info
                .is_document_size()
            {
                let document_id_in_primary_path =
                    contract_documents_keeping_history_primary_key_path_for_unknown_document_id(
                        contract.id().as_bytes(),
                        document_type,
                    );
                let reference_max_size =
                    contract_documents_keeping_history_storage_time_reference_path_size(
                        document_type.name.len() as u32,
                    );
                PathKeyUnknownElementSize((
                    document_id_in_primary_path,
                    KnownKey(vec![0]),
                    Element::required_item_space(reference_max_size, STORAGE_FLAGS_SIZE),
                ))
            } else {
                // we should also insert a reference at 0 to the current value
                // todo: we could construct this only once
                let document_id_in_primary_path =
                    contract_documents_keeping_history_primary_key_path_for_document_id(
                        contract.id().as_bytes(),
                        document_type.name.as_str(),
                        document_and_contract_info
                            .owned_document_info
                            .document_info
                            .get_document_id_as_slice()
                            .ok_or(Error::Drive(DriveError::CorruptedCodeExecution(
                                "can not get document id from estimated document",
                            )))?,
                    );
                PathFixedSizeKeyRefElement((
                    document_id_in_primary_path,
                    &[0],
                    Element::Reference(
                        SiblingReference(encoded_time),
                        Some(1),
                        StorageFlags::map_to_some_element_flags(storage_flags),
                    ),
                ))
            };

            self.batch_insert(path_key_element_info, drive_operations)?;
        } else if insert_without_check {
            let path_key_element_info = match &document_and_contract_info
                .owned_document_info
                .document_info
            {
                DocumentRefAndSerialization((document, serialized_document, storage_flags)) => {
                    let element = Element::Item(
                        serialized_document.to_vec(),
                        StorageFlags::map_borrowed_cow_to_some_element_flags(storage_flags),
                    );
                    PathFixedSizeKeyRefElement((primary_key_path, document.id.as_slice(), element))
                }
                DocumentAndSerialization((document, serialized_document, storage_flags)) => {
                    let element = Element::Item(
                        serialized_document.to_vec(),
                        StorageFlags::map_borrowed_cow_to_some_element_flags(storage_flags),
                    );
                    PathFixedSizeKeyRefElement((primary_key_path, document.id.as_slice(), element))
                }
                DocumentRefInfo((document, storage_flags)) => {
                    let serialized_document =
                        document.serialize(document_and_contract_info.document_type)?;
                    let element = Element::Item(
                        serialized_document,
                        StorageFlags::map_borrowed_cow_to_some_element_flags(storage_flags),
                    );
                    PathFixedSizeKeyRefElement((primary_key_path, document.id.as_slice(), element))
                }
                DocumentEstimatedAverageSize(average_size) => PathKeyUnknownElementSize((
                    KeyInfoPath::from_known_path(primary_key_path),
                    KeyInfo::MaxKeySize {
                        unique_id: document_type.unique_id_for_storage().to_vec(),
                        max_size: DEFAULT_HASH_SIZE_U8,
                    },
                    Element::required_item_space(*average_size, STORAGE_FLAGS_SIZE),
                )),
                DocumentOwnedInfo((document, storage_flags)) => {
                    let serialized_document =
                        document.serialize(document_and_contract_info.document_type)?;
                    let element = Element::Item(
                        serialized_document,
                        StorageFlags::map_borrowed_cow_to_some_element_flags(storage_flags),
                    );
                    PathFixedSizeKeyRefElement((primary_key_path, document.id.as_slice(), element))
                }
            };
            self.batch_insert(path_key_element_info, drive_operations)?;
        } else {
            let path_key_element_info = match &document_and_contract_info
                .owned_document_info
                .document_info
            {
                DocumentRefAndSerialization((document, serialized_document, storage_flags)) => {
                    let element = Element::Item(
                        serialized_document.to_vec(),
                        StorageFlags::map_borrowed_cow_to_some_element_flags(storage_flags),
                    );
                    PathFixedSizeKeyRefElement((primary_key_path, document.id.as_slice(), element))
                }
                DocumentAndSerialization((document, serialized_document, storage_flags)) => {
                    let element = Element::Item(
                        serialized_document.to_vec(),
                        StorageFlags::map_borrowed_cow_to_some_element_flags(storage_flags),
                    );
                    PathFixedSizeKeyRefElement((primary_key_path, document.id.as_slice(), element))
                }
                DocumentOwnedInfo((document, storage_flags)) => {
                    let serialized_document =
                        document.serialize(document_and_contract_info.document_type)?;
                    let element = Element::Item(
                        serialized_document,
                        StorageFlags::map_borrowed_cow_to_some_element_flags(storage_flags),
                    );
                    PathFixedSizeKeyRefElement((primary_key_path, document.id.as_slice(), element))
                }
                DocumentRefInfo((document, storage_flags)) => {
                    let serialized_document =
                        document.serialize(document_and_contract_info.document_type)?;
                    let element = Element::Item(
                        serialized_document,
                        StorageFlags::map_borrowed_cow_to_some_element_flags(storage_flags),
                    );
                    PathFixedSizeKeyRefElement((primary_key_path, document.id.as_slice(), element))
                }
                DocumentEstimatedAverageSize(max_size) => PathKeyUnknownElementSize((
                    KeyInfoPath::from_known_path(primary_key_path),
                    KeyInfo::MaxKeySize {
                        unique_id: document_type.unique_id_for_storage().to_vec(),
                        max_size: DEFAULT_HASH_SIZE_U8,
                    },
                    Element::required_item_space(*max_size, STORAGE_FLAGS_SIZE),
                )),
            };
            let apply_type = if estimated_costs_only_with_layer_info.is_none() {
                BatchInsertApplyType::StatefulBatchInsert
            } else {
                BatchInsertApplyType::StatelessBatchInsert {
                    in_tree_using_sums: false,
                    target: QueryTargetValue(document_type.estimated_size() as u32),
                }
            };
            let inserted = self.batch_insert_if_not_exists(
                path_key_element_info,
                apply_type,
                transaction,
                drive_operations,
            )?;
            if !inserted {
                return Err(Error::Drive(DriveError::CorruptedDocumentAlreadyExists(
                    "item already exists",
                )));
            }
        }
        Ok(())
    }
}
