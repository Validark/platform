#[cfg(feature = "json-object")]
mod json_conversion;
#[cfg(feature = "platform-value")]
mod value_conversion;
mod state_transition_like;
pub(super) mod v0_methods;
mod types;
mod identity_signed;

use crate::identity::SecurityLevel::MASTER;
use crate::identity::{KeyID, SecurityLevel};
use crate::platform_serialization::PlatformSignable;
use crate::prelude::Identifier;
use crate::serialization_traits::{PlatformDeserializable, PlatformSerializable, Signable};
use crate::state_transition::{
    StateTransitionFieldTypes, StateTransitionLike, StateTransitionType,
};
use crate::version::LATEST_VERSION;
use crate::ProtocolError;
use bincode::{config, Decode, Encode};
use platform_serialization::{PlatformDeserialize, PlatformSerialize};
use platform_value::{BinaryData, Value};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::convert::TryInto;


#[derive(
Debug,
Clone,
Serialize,
Deserialize,
Encode,
Decode,
PlatformDeserialize,
PlatformSerialize,
PlatformSignable,
PartialEq,
)]
#[serde(rename_all = "camelCase")]
#[platform_error_type(ProtocolError)]
pub struct IdentityCreditTransferTransitionV0 {
    // Own ST fields
    pub identity_id: Identifier,
    pub recipient_id: Identifier,
    pub amount: u64,
    #[platform_signable(exclude_from_sig_hash)]
    pub signature_public_key_id: KeyID,
    #[platform_signable(exclude_from_sig_hash)]
    pub signature: BinaryData,
}

impl Default for IdentityCreditTransferTransitionV0 {
    fn default() -> Self {
        IdentityCreditTransferTransitionV0 {
            identity_id: Identifier::default(),
            recipient_id: Identifier::default(),
            amount: Default::default(),
            signature_public_key_id: Default::default(),
            signature: Default::default(),
        }
    }
}



#[cfg(test)]
mod test {
    use crate::identity::state_transition::identity_credit_transfer_transition::IdentityCreditTransferTransitionV0;

    use crate::serialization_traits::{PlatformDeserializable, PlatformSerializable};
    use crate::state_transition::StateTransitionType;

    use platform_value::Identifier;
    use rand::Rng;
    use std::fmt::Debug;
    use crate::state_transition::identity_credit_transfer_transition::v0::IdentityCreditTransferTransitionV0;

    fn test_identity_credit_transfer_transition<
        T: PlatformSerializable + PlatformDeserializable + Debug + PartialEq,
    >(
        transition: T,
    ) {
        let serialized = T::serialize(&transition).expect("expected to serialize");
        let deserialized = T::deserialize(serialized.as_slice()).expect("expected to deserialize");
        assert_eq!(transition, deserialized);
    }

    #[test]
    fn test_identity_credit_transfer_transition1() {
        let mut rng = rand::thread_rng();
        let transition = IdentityCreditTransferTransitionV0 {
            identity_id: Identifier::random(),
            recipient_id: Identifier::random(),
            amount: rng.gen(),
            signature_public_key_id: rng.gen(),
            signature: [0; 65].to_vec().into(),
        };

        test_identity_credit_transfer_transition(transition);
    }
}
