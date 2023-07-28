mod v0;

use crate::drive::flags::StorageFlags;
use crate::drive::object_size_info::DocumentInfo::DocumentRefInfo;
use crate::drive::object_size_info::{DocumentAndContractInfo, OwnedDocumentInfo};
use crate::drive::Drive;
use crate::error::drive::DriveError;
use crate::error::Error;
use crate::fee::op::LowLevelDriveOperation;
use dpp::block::block_info::BlockInfo;
use dpp::data_contract::document_type::DocumentTypeRef;
use dpp::data_contract::DataContract;
use dpp::document::Document;
use dpp::fee::fee_result::FeeResult;
use dpp::version::drive_versions::DriveVersion;
use dpp::version::PlatformVersion;
use grovedb::batch::KeyInfoPath;
use grovedb::{EstimatedLayerInformation, TransactionArg};
use std::borrow::Cow;
use std::collections::HashMap;

impl Drive {
    /// Updates a document and returns the associated fee.
    ///
    /// # Parameters
    /// * `document`: The document to be updated.
    /// * `contract`: The contract for the document.
    /// * `document_type`: The document type.
    /// * `owner_id`: The optional owner ID.
    /// * `block_info`: The block info.
    /// * `apply`: Whether to apply the operation.
    /// * `storage_flags`: Optional storage flags.
    /// * `transaction`: The transaction argument.
    /// * `drive_version`: The drive version to select the correct function version to run.
    ///
    /// # Returns
    /// * `Ok(FeeResult)` if the operation was successful.
    /// * `Err(DriveError::UnknownVersionMismatch)` if the drive version does not match known versions.
    pub fn update_document_for_contract(
        &self,
        document: &Document,
        contract: &DataContract,
        document_type: DocumentTypeRef,
        owner_id: Option<[u8; 32]>,
        block_info: BlockInfo,
        apply: bool,
        storage_flags: Option<Cow<StorageFlags>>,
        transaction: TransactionArg,
        platform_version: &PlatformVersion,
    ) -> Result<FeeResult, Error> {
        match platform_version
            .drive
            .methods
            .document
            .update
            .update_document_for_contract
        {
            0 => self.update_document_for_contract_v0(
                document,
                contract,
                document_type,
                owner_id,
                block_info,
                apply,
                storage_flags,
                transaction,
                platform_version,
            ),
            version => Err(Error::Drive(DriveError::UnknownVersionMismatch {
                method: "update_document_for_contract".to_string(),
                known_versions: vec![0],
                received: version,
            })),
        }
    }
}