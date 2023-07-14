mod v0;

use crate::drive::object_size_info::{DocumentAndContractInfo, OwnedDocumentInfo};
use crate::drive::Drive;
use crate::error::document::DocumentError;
use crate::error::drive::DriveError;
use crate::error::Error;
use crate::fee::calculate_fee;
use crate::fee::op::LowLevelDriveOperation;
use crate::fee::result::FeeResult;
use dpp::block::block_info::BlockInfo;
use dpp::identifier::Identifier;
use dpp::version::drive_versions::DriveVersion;
use grovedb::TransactionArg;

impl Drive {
    /// Adds a document to a contract.
    ///
    /// # Parameters
    /// * `document_and_contract_info`: Information about the document and contract.
    /// * `override_document`: Whether to override the document.
    /// * `block_info`: The block info.
    /// * `apply`: Whether to apply the operation.
    /// * `transaction`: The transaction argument.
    /// * `drive_version`: The drive version to select the correct function version to run.
    ///
    /// # Returns
    /// * `Ok(FeeResult)` if the operation was successful.
    /// * `Err(DriveError::UnknownVersionMismatch)` if the drive version does not match known versions.
    pub fn add_document_for_contract(
        &self,
        document_and_contract_info: DocumentAndContractInfo,
        override_document: bool,
        block_info: BlockInfo,
        apply: bool,
        transaction: TransactionArg,
        drive_version: &DriveVersion,
    ) -> Result<FeeResult, Error> {
        match drive_version
            .methods
            .document
            .insert
            .add_document_for_contract
        {
            0 => self.add_document_for_contract_v0(
                document_and_contract_info,
                override_document,
                block_info,
                apply,
                transaction,
            ),
            version => Err(Error::Drive(DriveError::UnknownVersionMismatch {
                method: "add_document_for_contract".to_string(),
                known_versions: vec![0],
                received: version,
            })),
        }
    }
}
