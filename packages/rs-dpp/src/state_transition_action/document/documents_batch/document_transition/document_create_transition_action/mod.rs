#[cfg(feature = "state-transition-transformers")]
pub mod transformer;
mod v0;

use crate::identity::TimestampMillis;
use platform_value::{Identifier, Value};
use std::collections::BTreeMap;

use crate::document::{Document, DocumentV0};
use crate::ProtocolError;

use serde::{Deserialize, Serialize};

pub use v0::*;
use crate::data_contract::base::DataContractBaseMethodsV0;
use crate::data_contract::document_type::v0::v0_methods::DocumentTypeV0Methods;
use crate::state_transition_action::document::documents_batch::document_transition::document_base_transition_action::{DocumentBaseTransitionAction, DocumentBaseTransitionActionV0};
use crate::version::{FeatureVersion, PlatformVersion};

#[derive(Debug, Clone)]
pub enum DocumentCreateTransitionAction {
    V0(DocumentCreateTransitionActionV0),
}

impl DocumentCreateTransitionActionAccessorsV0 for DocumentCreateTransitionAction {
    fn base(&self) -> &DocumentBaseTransitionAction {
        match self {
            DocumentCreateTransitionAction::V0(v0) => &v0.base,
        }
    }

    fn base_owned(self) -> DocumentBaseTransitionAction {
        match self {
            DocumentCreateTransitionAction::V0(v0) => v0.base,
        }
    }

    fn created_at(&self) -> Option<TimestampMillis> {
        match self {
            DocumentCreateTransitionAction::V0(v0) => v0.created_at,
        }
    }

    fn updated_at(&self) -> Option<TimestampMillis> {
        match self {
            DocumentCreateTransitionAction::V0(v0) => v0.updated_at,
        }
    }

    fn data(&self) -> &BTreeMap<String, Value> {
        match self {
            DocumentCreateTransitionAction::V0(v0) => &v0.data,
        }
    }

    fn data_owned(self) -> BTreeMap<String, Value> {
        match self {
            DocumentCreateTransitionAction::V0(v0) => v0.data,
        }
    }
}

impl Document {
    /// Attempts to create a new `Document` from the given `DocumentCreateTransition` reference and `owner_id`.
    ///
    /// # Arguments
    ///
    /// * `value` - A reference to the `DocumentCreateTransition` containing information about the document being created.
    /// * `owner_id` - The `Identifier` of the document's owner.
    ///
    /// # Returns
    ///
    /// * `Result<Self, ProtocolError>` - A new `Document` object if successful, otherwise a `ProtocolError`.
    pub fn try_from_create_transition(
        document_create_transition_action: &DocumentCreateTransitionAction,
        owner_id: Identifier,
        platform_version: &PlatformVersion,
    ) -> Result<Self, ProtocolError> {
        match document_create_transition_action {
            DocumentCreateTransitionAction::V0(v0) => {
                Self::try_from_create_transition_v0(v0, owner_id, platform_version)
            }
        }
    }

    /// Attempts to create a new `Document` from the given `DocumentCreateTransition` instance and `owner_id`.
    ///
    /// # Arguments
    ///
    /// * `value` - A `DocumentCreateTransition` instance containing information about the document being created.
    /// * `owner_id` - The `Identifier` of the document's owner.
    ///
    /// # Returns
    ///
    /// * `Result<Self, ProtocolError>` - A new `Document` object if successful, otherwise a `ProtocolError`.
    pub fn try_from_owned_create_transition(
        document_create_transition_action: DocumentCreateTransitionAction,
        owner_id: Identifier,
        platform_version: &PlatformVersion,
    ) -> Result<Self, ProtocolError> {
        match document_create_transition_action {
            DocumentCreateTransitionAction::V0(v0) => {
                Self::try_from_owned_create_transition_v0(v0, owner_id, platform_version)
            }
        }
    }
}
