use std::collections::HashMap;
use grovedb::batch::KeyInfoPath;
use grovedb::{EstimatedLayerInformation, TransactionArg};
use dpp::block::block_info::BlockInfo;
use dpp::version::drive_versions::DriveVersion;
use crate::drive::document::contract_documents_primary_key_path;
use crate::drive::Drive;
use crate::drive::grove_operations::DirectQueryType::{StatefulDirectQuery, StatelessDirectQuery};
use crate::drive::grove_operations::QueryTarget::QueryTargetValue;
use crate::drive::object_size_info::DocumentAndContractInfo;
use crate::error::Error;
use crate::fee::op::LowLevelDriveOperation;

impl Drive {
    /// Gathers the operations to add a document to a contract.
    pub(super) fn add_document_for_contract_operations_v0(
        &self,
        document_and_contract_info: DocumentAndContractInfo,
        override_document: bool,
        block_info: &BlockInfo,
        previous_batch_operations: &mut Option<&mut Vec<LowLevelDriveOperation>>,
        estimated_costs_only_with_layer_info: &mut Option<
            HashMap<KeyInfoPath, EstimatedLayerInformation>,
        >,
        transaction: TransactionArg,
        drive_version: &DriveVersion,
    ) -> Result<Vec<LowLevelDriveOperation>, Error> {
        let mut batch_operations: Vec<LowLevelDriveOperation> = vec![];
        let primary_key_path = contract_documents_primary_key_path(
            document_and_contract_info.contract.id().as_bytes(),
            document_and_contract_info.document_type.name.as_str(),
        );

        // Apply means stateful query
        let query_type = if estimated_costs_only_with_layer_info.is_none() {
            StatefulDirectQuery
        } else {
            StatelessDirectQuery {
                in_tree_using_sums: false,
                query_target: QueryTargetValue(
                    document_and_contract_info.document_type.estimated_size() as u32,
                ),
            }
        };

        if override_document
            && !document_and_contract_info
            .owned_document_info
            .document_info
            .is_document_size()
            && self.grove_has_raw(
            primary_key_path.as_ref().into(),
            document_and_contract_info
                .owned_document_info
                .document_info
                .id_key_value_info()
                .as_key_ref_request()?,
            query_type,
            transaction,
            &mut batch_operations,
            drive_version,
        )?
        {
            let update_operations = self.update_document_for_contract_operations(
                document_and_contract_info,
                block_info,
                previous_batch_operations,
                estimated_costs_only_with_layer_info,
                transaction,
                drive_version,
            )?;
            batch_operations.extend(update_operations);
            return Ok(batch_operations);
        } else {
            // if we are trying to get estimated costs we need to add the upper levels
            if let Some(estimated_costs_only_with_layer_info) = estimated_costs_only_with_layer_info
            {
                Self::add_estimation_costs_for_levels_up_to_contract_document_type_excluded(
                    document_and_contract_info.contract,
                    estimated_costs_only_with_layer_info,
                );
            }
            // if we have override_document set that means we already checked if it exists
            self.add_document_to_primary_storage(
                &document_and_contract_info,
                block_info,
                override_document,
                estimated_costs_only_with_layer_info,
                transaction,
                &mut batch_operations,
                drive_version,
            )?;
        }

        self.add_indices_for_top_index_level_for_contract_operations(
            &document_and_contract_info,
            previous_batch_operations,
            estimated_costs_only_with_layer_info,
            transaction,
            &mut batch_operations,
            drive_version,
        )?;
        Ok(batch_operations)
    }
}