use crate::state_transition::identity_public_key_transitions::IdentityPublicKeyInCreation;
use crate::state_transition::identity_update_transition::IdentityUpdateTransition;
use crate::util::deserializer::ProtocolVersion;
use crate::version::{FeatureVersion, PlatformVersion};
use crate::{
    identity::{KeyType, Purpose, SecurityLevel},
    state_transition::StateTransitionType,
    tests::utils::generate_random_identifier_struct,
    version::LATEST_VERSION,
};
use platform_value::string_encoding::Encoding;
use platform_value::BinaryData;

pub fn get_identity_update_transition_fixture(
    platform_version: PlatformVersion,
) -> IdentityUpdateTransition {
    match platform_version
        .dpp
        .state_transition_serialization_versions
        .identity_update_state_transition
        .default_current_version
    {
        0 => IdentityUpdateTransitionV0 {
            protocol_version: LATEST_VERSION,
            transition_type: StateTransitionType::IdentityUpdate,
            signature: BinaryData::new(vec![0; 65]),
            signature_public_key_id: 0,
            identity_id: generate_random_identifier_struct(),
            revision: 0,
            add_public_keys: vec![IdentityPublicKeyInCreation {
                id: 3,
                key_type: KeyType::ECDSA_SECP256K1,
                purpose: Purpose::AUTHENTICATION,
                read_only: false,
                data: BinaryData::from_string(
                    "AkVuTKyF3YgKLAQlLEtaUL2HTditwGILfWUVqjzYnIgH",
                    Encoding::Base64,
                )
                .unwrap(),
                security_level: SecurityLevel::MASTER,
                signature: BinaryData::new(vec![0; 65]),
            }],
            disable_public_keys: vec![0],
            public_keys_disabled_at: Some(1234567),
            ..Default::default()
        }
        .into(),
        _ => unimplemented!("not yet implemented get_identity_update_transition_fixture"),
    }
}
