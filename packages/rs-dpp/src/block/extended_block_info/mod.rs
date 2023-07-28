use crate::block::block_info::BlockInfo;
use crate::block::extended_block_info::v0::{
    ExtendedBlockInfoV0, ExtendedBlockInfoV0Getters, ExtendedBlockInfoV0Setters,
};
use crate::protocol_error::ProtocolError;
use crate::serialization::PlatformDeserializable;
use crate::serialization::PlatformSerializable;
use crate::version::FeatureVersion;
use bincode::{config, Decode, Encode};
use derive_more::From;
use platform_serialization_derive::{PlatformDeserialize, PlatformSerialize};
use serde::{Deserialize, Serialize};

pub mod v0;

/// Extended Block information
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    PlatformSerialize,
    PlatformDeserialize,
    From,
)]
pub enum ExtendedBlockInfo {
    V0(ExtendedBlockInfoV0),
}

impl ExtendedBlockInfo {
    /// Returns the version of this ExtendedBlockInfo.
    /// Currently, the only available version is 0.
    pub fn version(&self) -> FeatureVersion {
        match self {
            ExtendedBlockInfo::V0(_) => 0,
        }
    }
}

impl ExtendedBlockInfoV0Getters for ExtendedBlockInfo {
    fn basic_info(&self) -> &BlockInfo {
        match self {
            ExtendedBlockInfo::V0(v0) => &v0.basic_info,
        }
    }

    fn basic_info_mut(&mut self) -> &mut BlockInfo {
        match self {
            ExtendedBlockInfo::V0(v0) => &mut v0.basic_info,
        }
    }

    fn basic_info_owned(self) -> BlockInfo {
        match self {
            ExtendedBlockInfo::V0(v0) => v0.basic_info,
        }
    }

    fn app_hash(&self) -> &[u8; 32] {
        match self {
            ExtendedBlockInfo::V0(v0) => &v0.app_hash,
        }
    }

    fn quorum_hash(&self) -> &[u8; 32] {
        match self {
            ExtendedBlockInfo::V0(v0) => &v0.quorum_hash,
        }
    }

    fn signature(&self) -> &[u8; 96] {
        match self {
            ExtendedBlockInfo::V0(v0) => &v0.signature,
        }
    }

    fn round(&self) -> u32 {
        match self {
            ExtendedBlockInfo::V0(v0) => v0.round,
        }
    }
}

impl ExtendedBlockInfoV0Setters for ExtendedBlockInfo {
    fn set_basic_info(&mut self, info: BlockInfo) {
        if let ExtendedBlockInfo::V0(v0) = self {
            v0.set_basic_info(info);
        }
    }

    fn set_app_hash(&mut self, hash: [u8; 32]) {
        if let ExtendedBlockInfo::V0(v0) = self {
            v0.set_app_hash(hash);
        }
    }

    fn set_quorum_hash(&mut self, hash: [u8; 32]) {
        if let ExtendedBlockInfo::V0(v0) = self {
            v0.set_quorum_hash(hash);
        }
    }

    fn set_signature(&mut self, signature: [u8; 96]) {
        if let ExtendedBlockInfo::V0(v0) = self {
            v0.set_signature(signature);
        }
    }

    fn set_round(&mut self, round: u32) {
        if let ExtendedBlockInfo::V0(v0) = self {
            v0.set_round(round);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::block_info::BlockInfo;
    use ciborium::{de::from_reader, ser::into_writer};

    #[test]
    fn test_extended_block_info_serde_ciborium() {
        let block_info: ExtendedBlockInfo = ExtendedBlockInfoV0 {
            basic_info: BlockInfo::default(),
            app_hash: [1; 32],
            quorum_hash: [2; 32],
            signature: [3; 96],
            round: 1,
        }
        .into();

        // Serialize into a vector
        let mut encoded: Vec<u8> = vec![];
        into_writer(&block_info, &mut encoded).unwrap();

        // Deserialize from the vector
        let decoded: ExtendedBlockInfo = from_reader(&encoded[..]).unwrap();

        assert_eq!(block_info, decoded);
    }

    #[test]
    fn test_extended_block_info_bincode() {
        let block_info: ExtendedBlockInfo = ExtendedBlockInfoV0 {
            basic_info: BlockInfo::default(),
            app_hash: [1; 32],
            quorum_hash: [2; 32],
            signature: [3; 96],
            round: 1,
        }
        .into();

        // Serialize into a vector
        let mut encoded: Vec<u8> = block_info.serialize().expect("expected to serialize");

        // Deserialize from the vector
        let decoded: ExtendedBlockInfo = ExtendedBlockInfo::deserialize(encoded);

        assert_eq!(block_info, decoded);
    }
}