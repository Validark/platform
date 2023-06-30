
use grovedb::batch::key_info::KeyInfo::KnownKey;
use grovedb::batch::KeyInfoPath;

use grovedb::EstimatedLayerCount::{ApproximateElements, PotentiallyAtMaxElements};
use grovedb::EstimatedLayerSizes::{AllItems, AllReference, AllSubtrees};
use grovedb::{Element, EstimatedLayerInformation, TransactionArg};

use dpp::data_contract::document_type::{DocumentType, IndexLevel};

use grovedb::EstimatedSumTrees::NoSumTrees;
use std::collections::HashMap;

use dpp::data_contract::DataContract;
use crate::drive::defaults::{
    AVERAGE_NUMBER_OF_UPDATES, AVERAGE_UPDATE_BYTE_COUNT_REQUIRED_SIZE,
    CONTRACT_DOCUMENTS_PATH_HEIGHT, DEFAULT_HASH_SIZE_U8,
};
use crate::drive::document::{
    contract_document_type_path_vec, contract_documents_primary_key_path, document_reference_size,
    unique_event_id,
};
use crate::drive::flags::StorageFlags;
use crate::drive::object_size_info::DocumentInfo::{
    DocumentEstimatedAverageSize, DocumentOwnedInfo,
};
use crate::drive::object_size_info::DriveKeyInfo::KeyRef;
use dpp::block::extended_block_info::BlockInfo;
use dpp::document::Document;

use crate::drive::grove_operations::BatchDeleteApplyType::{
    StatefulBatchDelete, StatelessBatchDelete,
};
use crate::drive::grove_operations::DirectQueryType;
use crate::drive::grove_operations::QueryTarget::QueryTargetValue;
use crate::drive::object_size_info::{DocumentAndContractInfo, OwnedDocumentInfo, PathInfo};
use crate::drive::Drive;
use crate::error::document::DocumentError;
use crate::error::drive::DriveError;
use crate::error::fee::FeeError;
use crate::error::Error;
use crate::fee::calculate_fee;
use crate::fee::op::LowLevelDriveOperation;

use crate::fee::result::FeeResult;
use dpp::block::epoch::Epoch;
use dpp::version::drive_versions::DriveVersion;


impl Drive {
    /// Removes indices for the top index level and calls for lower levels.
    pub(super) fn remove_indices_for_top_index_level_for_contract_operations_v0(
        &self,
        document_and_contract_info: &DocumentAndContractInfo,
        previous_batch_operations: &Option<&mut Vec<LowLevelDriveOperation>>,
        estimated_costs_only_with_layer_info: &mut Option<
            HashMap<KeyInfoPath, EstimatedLayerInformation>,
        >,
        transaction: TransactionArg,
        batch_operations: &mut Vec<LowLevelDriveOperation>,
        drive_version: &DriveVersion,
    ) -> Result<(), Error> {
        let document_type = document_and_contract_info.document_type;
        let index_level = &document_type.index_structure;
        let contract = document_and_contract_info.contract;
        let event_id = unique_event_id();
        let storage_flags = if document_type.documents_mutable || contract.config.can_be_deleted {
            document_and_contract_info
                .owned_document_info
                .document_info
                .get_storage_flags_ref()
        } else {
            None //there are no need for storage flags if documents are not mutable and contract can not be deleted
        };

        // we need to construct the path for documents on the contract
        // the path is
        //  * Document and Contract root tree
        //  * Contract ID recovered from document
        //  * 0 to signify Documents and not Contract
        let contract_document_type_path = contract_document_type_path_vec(
            document_and_contract_info.contract.id().as_bytes(),
            document_and_contract_info.document_type.name.as_str(),
        );

        let sub_level_index_count = index_level.sub_index_levels.len() as u32;

        if let Some(estimated_costs_only_with_layer_info) = estimated_costs_only_with_layer_info {
            // On this level we will have a 0 and all the top index paths
            estimated_costs_only_with_layer_info.insert(
                KeyInfoPath::from_known_owned_path(contract_document_type_path.clone()),
                EstimatedLayerInformation {
                    is_sum_tree: false,
                    estimated_layer_count: ApproximateElements(sub_level_index_count + 1),
                    estimated_layer_sizes: AllSubtrees(
                        DEFAULT_HASH_SIZE_U8,
                        NoSumTrees,
                        storage_flags.map(|s| s.serialized_size()),
                    ),
                },
            );
        }

        // next we need to store a reference to the document for each index
        for (name, sub_level) in &index_level.sub_index_levels {
            // at this point the contract path is to the contract documents
            // for each index the top index component will already have been added
            // when the contract itself was created
            let mut index_path: Vec<Vec<u8>> = contract_document_type_path.clone();
            index_path.push(Vec::from(name.as_bytes()));

            // with the example of the dashpay contract's first index
            // the index path is now something like Contracts/ContractID/Documents(1)/$ownerId
            let document_top_field = document_and_contract_info
                .owned_document_info
                .document_info
                .get_raw_for_document_type(
                    name,
                    document_type,
                    document_and_contract_info.owned_document_info.owner_id,
                    Some((sub_level, event_id)),
                    drive_version,
                )?
                .unwrap_or_default();

            if let Some(estimated_costs_only_with_layer_info) = estimated_costs_only_with_layer_info
            {
                let document_top_field_estimated_size = document_and_contract_info
                    .owned_document_info
                    .document_info
                    .get_estimated_size_for_document_type(name, document_type, drive_version)?;

                if document_top_field_estimated_size > u8::MAX as u16 {
                    return Err(Error::Fee(FeeError::Overflow(
                        "document top field is too big for being an index",
                    )));
                }

                // On this level we will have all the user defined values for the paths
                estimated_costs_only_with_layer_info.insert(
                    KeyInfoPath::from_known_owned_path(index_path.clone()),
                    EstimatedLayerInformation {
                        is_sum_tree: false,
                        estimated_layer_count: PotentiallyAtMaxElements,
                        estimated_layer_sizes: AllSubtrees(
                            document_top_field_estimated_size as u8,
                            NoSumTrees,
                            storage_flags.map(|s| s.serialized_size()),
                        ),
                    },
                );
            }

            let any_fields_null = document_top_field.is_empty();

            let mut index_path_info = if document_and_contract_info
                .owned_document_info
                .document_info
                .is_document_size()
            {
                // This is a stateless operation
                PathInfo::PathWithSizes(KeyInfoPath::from_known_owned_path(index_path))
            } else {
                PathInfo::PathIterator::<0>(index_path)
            };

            // we push the actual value of the index path
            index_path_info.push(document_top_field)?;
            // the index path is now something like Contracts/ContractID/Documents(1)/$ownerId/<ownerId>

            self.remove_indices_for_index_level_for_contract_operations(
                document_and_contract_info,
                index_path_info,
                sub_level,
                any_fields_null,
                &storage_flags,
                previous_batch_operations,
                estimated_costs_only_with_layer_info,
                event_id,
                transaction,
                batch_operations,
                drive_version,
            )?;
        }
        Ok(())
    }
}
