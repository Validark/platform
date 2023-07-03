use crate::platform_serialization::PlatformSignable;
use crate::serialization_traits::{PlatformDeserializable, Signable};
use bincode::{config, Decode, Encode};
use platform_value::{BinaryData, ReplacementType, Value};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::convert::{TryFrom, TryInto};

use crate::consensus::signature::{
    InvalidSignaturePublicKeySecurityLevelError, MissingPublicKeyError, SignatureError,
};
use crate::consensus::ConsensusError;
use crate::identity::signer::Signer;
use crate::identity::{Identity, IdentityPublicKey};

use crate::identity::SecurityLevel::{CRITICAL, MASTER};
use crate::state_transition::identity_credit_withdrawal_transition::v0::IdentityCreditWithdrawalTransitionV0;
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

pub trait IdentityCreditWithdrawalTransitionV0Methods {
    /// Get State Transition Type
    fn get_type() -> StateTransitionType {
        StateTransitionType::IdentityCreditWithdrawal
    }

    /// Get owner ID
    fn owner_id(&self) -> Identifier;
    fn set_revision(&mut self, revision: Revision);
    fn revision(&self) -> Revision;
}

impl IdentityCreditWithdrawalTransitionV0Methods for IdentityCreditWithdrawalTransitionV0 {
    fn set_revision(&mut self, revision: Revision) {
        self.revision = revision;
    }

    fn revision(&self) -> Revision {
        self.revision
    }

    /// Get owner ID
    fn owner_id(&self) -> Identifier {
        self.identity_id
    }
}
