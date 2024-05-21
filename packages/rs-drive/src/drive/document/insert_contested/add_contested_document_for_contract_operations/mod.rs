mod v0;

use crate::drive::object_size_info::DocumentAndContractInfo;
use crate::drive::Drive;
use crate::error::drive::DriveError;
use crate::error::Error;
use crate::fee::op::LowLevelDriveOperation;
use dpp::block::block_info::BlockInfo;

use dpp::version::PlatformVersion;
use dpp::voting::vote_polls::contested_document_resource_vote_poll::ContestedDocumentResourceVotePoll;
use grovedb::batch::KeyInfoPath;
use grovedb::{EstimatedLayerInformation, TransactionArg};
use std::collections::HashMap;

impl Drive {
    /// Gathers the operations to add a contested document to a contract.
    pub(crate) fn add_contested_document_for_contract_operations(
        &self,
        document_and_contract_info: DocumentAndContractInfo,
        contested_document_resource_vote_poll: ContestedDocumentResourceVotePoll,
        insert_without_check: bool,
        block_info: &BlockInfo,
        previous_batch_operations: &mut Option<&mut Vec<LowLevelDriveOperation>>,
        estimated_costs_only_with_layer_info: &mut Option<
            HashMap<KeyInfoPath, EstimatedLayerInformation>,
        >,
        transaction: TransactionArg,
        platform_version: &PlatformVersion,
    ) -> Result<Vec<LowLevelDriveOperation>, Error> {
        match platform_version
            .drive
            .methods
            .document
            .insert_contested
            .add_contested_document_for_contract_operations
        {
            0 => self.add_contested_document_for_contract_operations_v0(
                document_and_contract_info,
                contested_document_resource_vote_poll,
                insert_without_check,
                block_info,
                previous_batch_operations,
                estimated_costs_only_with_layer_info,
                transaction,
                platform_version,
            ),
            version => Err(Error::Drive(DriveError::UnknownVersionMismatch {
                method: "add_contested_document_for_contract_operations".to_string(),
                known_versions: vec![0],
                received: version,
            })),
        }
    }
}