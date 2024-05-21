use crate::drive::batch::transitions::document::DriveHighLevelDocumentOperationConverter;
use crate::drive::batch::DriveOperation::{DocumentOperation, IdentityOperation};
use crate::drive::batch::{DocumentOperationType, DriveOperation, IdentityOperationType};
use crate::drive::flags::StorageFlags;
use crate::drive::object_size_info::DocumentInfo::DocumentOwnedInfo;
use crate::drive::object_size_info::{DataContractInfo, DocumentTypeInfo, OwnedDocumentInfo};
use crate::error::Error;
use dpp::block::epoch::Epoch;

use dpp::prelude::Identifier;
use std::borrow::Cow;
use crate::state_transition_action::document::documents_batch::document_transition::document_base_transition_action::DocumentBaseTransitionActionAccessorsV0;
use crate::state_transition_action::document::documents_batch::document_transition::document_update_price_transition_action::{DocumentUpdatePriceTransitionAction, DocumentUpdatePriceTransitionActionAccessorsV0};
use dpp::version::PlatformVersion;

impl DriveHighLevelDocumentOperationConverter for DocumentUpdatePriceTransitionAction {
    fn into_high_level_document_drive_operations<'b>(
        self,
        epoch: &Epoch,
        owner_id: Identifier,
        _platform_version: &PlatformVersion,
    ) -> Result<Vec<DriveOperation<'b>>, Error> {
        let data_contract_id = self.base().data_contract_id();
        let document_type_name = self.base().document_type_name().clone();
        let identity_contract_nonce = self.base().identity_contract_nonce();
        let fetch_info = self.base().data_contract_fetch_info();
        let document = self.document_owned();

        let storage_flags = StorageFlags::new_single_epoch(epoch.index, Some(owner_id.to_buffer()));

        Ok(vec![
            IdentityOperation(IdentityOperationType::UpdateIdentityContractNonce {
                identity_id: owner_id.into_buffer(),
                contract_id: data_contract_id.into_buffer(),
                nonce: identity_contract_nonce,
            }),
            DocumentOperation(DocumentOperationType::UpdateDocument {
                owned_document_info: OwnedDocumentInfo {
                    document_info: DocumentOwnedInfo((document, Some(Cow::Owned(storage_flags)))),
                    owner_id: Some(owner_id.into_buffer()),
                },
                contract_info: DataContractInfo::DataContractFetchInfo(fetch_info),
                document_type_info: DocumentTypeInfo::DocumentTypeName(document_type_name),
            }),
        ])
    }
}