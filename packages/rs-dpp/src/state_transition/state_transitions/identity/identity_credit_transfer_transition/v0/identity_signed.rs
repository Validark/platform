use crate::identity::{KeyID, SecurityLevel};
use crate::identity::SecurityLevel::CRITICAL;
use crate::state_transition::identity_credit_transfer_transition::v0::IdentityCreditTransferTransitionV0;
use crate::state_transition::{StateTransitionFieldTypes, StateTransitionIdentitySigned};

impl StateTransitionIdentitySigned for IdentityCreditTransferTransitionV0 {
    fn signature_public_key_id(&self) -> Option<KeyID> {
        Some(self.signature_public_key_id)
    }

    fn set_signature_public_key_id(&mut self, key_id: KeyID) {
        self.signature_public_key_id = key_id
    }

    fn security_level_requirement(&self) -> Vec<SecurityLevel> {
        vec![CRITICAL]
    }
}