use crate::state_transition::identity_public_key_transitions::IdentityPublicKeyInCreation;
use crate::state_transition::identity_update_transition::IdentityUpdateTransitionV0;
use crate::state_transition::identity_update_transition::IdentityUpdateTransition;
use crate::state_transition_action::identity::identity_update::v0::IdentityUpdateTransitionActionV0;


impl From<IdentityUpdateTransitionV0> for IdentityUpdateTransitionActionV0 {
    fn from(value: IdentityUpdateTransitionV0) -> Self {
        let IdentityUpdateTransitionV0 {
            identity_id,
            add_public_keys,
            disable_public_keys,
            public_keys_disabled_at,
            revision,
            ..
        } = value;
        IdentityUpdateTransitionActionV0 {
            add_public_keys: add_public_keys
                .into_iter()
                .map(IdentityPublicKeyInCreation::to_identity_public_key)
                .collect(),
            disable_public_keys,
            public_keys_disabled_at,
            identity_id,
            revision,
        }
    }
}

impl From<&IdentityUpdateTransition> for IdentityUpdateTransitionActionV0 {
    fn from(value: &IdentityUpdateTransition) -> Self {
        let IdentityUpdateTransitionV0 {
            identity_id,
            add_public_keys,
            disable_public_keys,
            public_keys_disabled_at,
            revision,
            ..
        } = value;
        IdentityUpdateTransitionActionV0 {
            add_public_keys: add_public_keys
                .iter()
                .map(|key| key.clone().to_identity_public_key())
                .collect(),
            disable_public_keys: disable_public_keys.clone(),
            public_keys_disabled_at: *public_keys_disabled_at,
            identity_id: *identity_id,
            revision: *revision,
        }
    }
}
