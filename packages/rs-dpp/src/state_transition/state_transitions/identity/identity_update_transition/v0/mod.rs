mod identity_signed;
#[cfg(feature = "json-object")]
mod json_conversion;
mod state_transition_like;
mod types;
pub(super) mod v0_methods;
#[cfg(feature = "platform-value")]
mod value_conversion;

use crate::platform_serialization::PlatformSignable;
use crate::serialization_traits::{PlatformDeserializable, Signable};
use bincode::{config, Decode, Encode};
use platform_value::{BinaryData, ReplacementType, Value};
use serde::{Deserialize, Serialize};

use std::convert::{TryFrom, TryInto};

use crate::consensus::signature::{
    InvalidSignaturePublicKeySecurityLevelError, MissingPublicKeyError, SignatureError,
};
use crate::consensus::ConsensusError;
use crate::identity::signer::Signer;
use crate::identity::{Identity, IdentityPublicKey};

use crate::serialization_traits::PlatformSerializable;
use crate::state_transition::identity_public_key_transitions::IdentityPublicKeyInCreation;
use crate::version::FeatureVersion;
use crate::{
    identity::{KeyID, SecurityLevel},
    prelude::{Identifier, Revision, TimestampMillis},
    state_transition::{StateTransitionFieldTypes, StateTransitionLike, StateTransitionType},
    version::LATEST_VERSION,
    ProtocolError,
};
use platform_serialization::{PlatformDeserialize, PlatformSerialize};

#[derive(
    Serialize,
    Deserialize,
    Encode,
    Decode,
    PlatformDeserialize,
    PlatformSerialize,
    PlatformSignable,
    Debug,
    Clone,
    PartialEq,
)]
#[serde(rename_all = "camelCase")]
#[platform_error_type(ProtocolError)]
pub struct IdentityUpdateTransitionV0 {
    /// Unique identifier of the identity to be updated
    pub identity_id: Identifier,

    /// Identity Update revision number
    pub revision: Revision,

    /// Public Keys to add to the Identity
    /// we want to skip serialization of transitions, as we does it manually in `to_object()`  and `to_json()`
    #[serde(default)]
    #[platform_signable(into = "Vec<IdentityPublicKeyInCreationSignable>")]
    pub add_public_keys: Vec<IdentityPublicKeyInCreation>,

    /// Identity Public Keys ID's to disable for the Identity
    #[serde(default)]
    pub disable_public_keys: Vec<KeyID>,

    /// Timestamp when keys were disabled
    pub public_keys_disabled_at: Option<TimestampMillis>,

    /// The ID of the public key used to sing the State Transition
    #[platform_signable(exclude_from_sig_hash)]
    pub signature_public_key_id: KeyID,
    /// Cryptographic signature of the State Transition
    #[platform_signable(exclude_from_sig_hash)]
    pub signature: BinaryData,
}

impl Default for IdentityUpdateTransitionV0 {
    fn default() -> Self {
        Self {
            transition_type: StateTransitionType::IdentityUpdate,
            signature: Default::default(),
            signature_public_key_id: Default::default(),
            identity_id: Default::default(),
            revision: Default::default(),
            add_public_keys: Default::default(),
            disable_public_keys: Default::default(),
            public_keys_disabled_at: Default::default(),
        }
    }
}

/// if the property isn't present the empty list is returned. If property is defined, the function
/// might return some serialization-related errors
fn get_list<T: TryFrom<Value, Error = platform_value::Error>>(
    value: &mut Value,
    property_name: &str,
) -> Result<Vec<T>, ProtocolError> {
    value
        .remove_optional_array(property_name)
        .map_err(ProtocolError::ValueError)?
        .unwrap_or_default()
        .into_iter()
        .map(|value| value.try_into().map_err(ProtocolError::ValueError))
        .collect()
}

/// if the property isn't present the empty list is returned. If property is defined, the function
/// might return some serialization-related errors
fn remove_integer_list_or_default<T>(
    value: &mut Value,
    property_name: &str,
) -> Result<Vec<T>, ProtocolError>
where
    T: TryFrom<i128>
        + TryFrom<u128>
        + TryFrom<u64>
        + TryFrom<i64>
        + TryFrom<u32>
        + TryFrom<i32>
        + TryFrom<u16>
        + TryFrom<i16>
        + TryFrom<u8>
        + TryFrom<i8>,
{
    value
        .remove_optional_array(property_name)
        .map_err(ProtocolError::ValueError)?
        .unwrap_or_default()
        .into_iter()
        .map(|value| value.to_integer().map_err(ProtocolError::ValueError))
        .collect()
}

#[cfg(test)]
mod test {
    use crate::state_transition::{JsonSerializationOptions, StateTransitionJsonConvert};
    use crate::tests::{fixtures::identity_fixture, utils::generate_random_identifier_struct};
    use getrandom::getrandom;

    use super::*;

    #[test]
    fn conversion_to_raw_object() {
        let public_key = identity_fixture().get_public_keys()[&0].to_owned();
        let mut buffer = [0u8; 33];
        let _ = getrandom(&mut buffer);
        let transition = IdentityUpdateTransitionV0 {
            identity_id: generate_random_identifier_struct(),
            add_public_keys: vec![(&public_key).into()],
            signature: BinaryData::new(buffer.to_vec()),

            ..Default::default()
        };

        let result = transition
            .to_object(false)
            .expect("conversion to raw object shouldn't fail");

        assert!(matches!(result[IDENTITY_ID], Value::Identifier(_)));
        assert!(matches!(result[SIGNATURE], Value::Bytes(_)));
        assert!(matches!(
            result[ADD_PUBLIC_KEYS][0]["data"],
            Value::Bytes(_)
        ));
    }
}
