use crate::consensus::basic::BasicError;
use crate::consensus::ConsensusError;
use crate::ProtocolError;
use bincode::{Decode, Encode};
use platform_serialization_derive::{PlatformDeserialize, PlatformSerialize};
use thiserror::Error;

#[derive(
    Error, Debug, Clone, PartialEq, Eq, Encode, Decode, PlatformSerialize, PlatformDeserialize,
)]
#[error(
    "Unrecognized trade mode: allowed {:?}, got {}",
    allowed_values,
    received
)]
#[platform_serialize(unversioned)]
pub struct UnknownTradeModeError {
    /*

    DO NOT CHANGE ORDER OF FIELDS WITHOUT INTRODUCING OF NEW VERSION

    */
    allowed_values: Vec<u8>,
    received: u8,
}

impl UnknownTradeModeError {
    pub fn new(allowed_values: Vec<u8>, received: u8) -> Self {
        Self {
            allowed_values,
            received,
        }
    }

    pub fn allowed_values(&self) -> Vec<u8> {
        self.allowed_values.clone()
    }
    pub fn received(&self) -> u8 {
        self.received
    }
}

impl From<UnknownTradeModeError> for ConsensusError {
    fn from(err: UnknownTradeModeError) -> Self {
        Self::BasicError(BasicError::UnknownTradeModeError(err))
    }
}
