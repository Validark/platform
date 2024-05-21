use crate::drive::object_size_info::DocumentAndContractInfo;
use crate::drive::Drive;

use crate::error::Error;
use crate::fee::op::LowLevelDriveOperation;
use dpp::block::block_info::BlockInfo;
use dpp::fee::fee_result::FeeResult;

use dpp::version::PlatformVersion;
use dpp::voting::vote_polls::contested_document_resource_vote_poll::ContestedDocumentResourceVotePoll;
use grovedb::TransactionArg;

impl Drive {
    /// Adds a contested document to a contract.
    #[inline(always)]
    pub(super) fn add_contested_document_for_contract_v0(
        &self,
        document_and_contract_info: DocumentAndContractInfo,
        contested_document_resource_vote_poll: ContestedDocumentResourceVotePoll,
        insert_without_check: bool,
        block_info: BlockInfo,
        apply: bool,
        transaction: TransactionArg,
        platform_version: &PlatformVersion,
    ) -> Result<FeeResult, Error> {
        let mut drive_operations: Vec<LowLevelDriveOperation> = vec![];
        self.add_contested_document_for_contract_apply_and_add_to_operations(
            document_and_contract_info,
            contested_document_resource_vote_poll,
            insert_without_check,
            &block_info,
            true,
            apply,
            transaction,
            &mut drive_operations,
            platform_version,
        )?;
        let fees = Drive::calculate_fee(
            None,
            Some(drive_operations),
            &block_info.epoch,
            self.config.epochs_per_era,
            platform_version,
        )?;
        Ok(fees)
    }
}